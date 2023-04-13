use std::ops::Range;
use std::{ops::RangeInclusive, sync::Arc};

use bumpalo::Bump;

use crate::error::FluxError;
use crate::lexer::CullStrategy;
use crate::tokens::Token;

pub type MatcherName = Arc<Option<String>>;
pub type TokenResult = Option<Range<usize>>;

pub struct SuccessMark {
    pub(crate) begin: usize,
    pub(crate) end: usize,
    pub(crate) depth: usize,
    pub(crate) matcher: Option<usize>,
}

impl Default for SuccessMark {
    fn default() -> Self {
        Self {
            begin: 0,
            end: 0,
            depth: usize::MAX,
            matcher: None,
        }
    }
}

pub struct TokenOutput<'a> {
    pub(crate) tokens: bumpalo::collections::Vec<'a, Token<'a>>,
    pub(crate) last_success: SuccessMark,
}

impl<'a> TokenOutput<'a> {
    fn len(&self) -> usize {
        self.tokens.len()
    }

    fn mark_success(&mut self, begin: usize, end: usize, depth: usize, matcher: &Matcher) {
        if end < self.last_success.end {
            return;
        }
        if end > self.last_success.end {
            self.last_success.depth = depth;
        }
        self.last_success.begin = begin;
        self.last_success.end = end;
        if matcher.show_in_errors && depth <= self.last_success.depth {
            self.last_success.matcher.replace(matcher.id);
            self.last_success.depth = depth;
        }
    }

    pub(crate) fn create_error(&self, source: Arc<[char]>, matchers: &[Matcher]) -> FluxError {
        if let Some(matcher) = self.last_success.matcher {
            let name = matchers[matcher].name.clone();
            FluxError::new_matcher("expected", self.last_success.end, 0, name, Some(source))
        } else {
            FluxError::new("unexpected token", self.last_success.end, Some(source))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Matcher {
    pub(crate) cull_strategy: CullStrategy,
    pub(crate) name: MatcherName,
    pub(crate) id: usize,
    pub(crate) matcher_type: MatcherType,
    pub(crate) show_in_errors: bool,
}

#[derive(Debug, Clone)]
pub enum MatcherType {
    String(Vec<char>, bool),
    CharSet(Vec<char>, bool),
    CharRange(RangeInclusive<char>, bool),
    List(Vec<usize>),
    Choice(Vec<usize>, Option<[Vec<usize>; 256]>),
    Repeating(usize, RangeInclusive<usize>, Option<[bool; 256]>),
    Inverted(usize),
    Wrapper(usize),
    Eof,
    Newline,
    Placeholder,
}

impl Matcher {
    pub fn apply<'a>(
        &self,
        source: Arc<[char]>,
        output: &mut TokenOutput<'a>,
        matchers: &[Matcher],
        pos: usize,
        depth: usize,
        alloc: &'a Bump,
    ) -> TokenResult {
        match &self.matcher_type {
            MatcherType::String(to_match, case_sensitive) => apply_string(
                self,
                source,
                output,
                pos,
                depth,
                to_match,
                *case_sensitive,
                alloc,
            ),
            MatcherType::CharSet(chars, inverted) => {
                apply_char_set(self, source, output, pos, chars, *inverted, alloc)
            }
            MatcherType::CharRange(range, inverted) => {
                apply_char_range(self, source, output, pos, range, *inverted, alloc)
            }
            MatcherType::List(children) => {
                apply_list(self, source, output, pos, depth, children, matchers, alloc)
            }
            MatcherType::Choice(children, cache) => {
                let choice_children = source
                    .get(pos)
                    .filter(|c| c.is_ascii())
                    .map(|c| *c as u32 as usize)
                    .and_then(|c| cache.as_ref().map(|cache| &cache[c]))
                    .unwrap_or(children);
                apply_choice(
                    self,
                    source,
                    output,
                    pos,
                    depth,
                    choice_children,
                    matchers,
                    alloc,
                )
            }
            MatcherType::Repeating(child, range, cache) => apply_repeating(
                self, source, output, pos, depth, *child, range, cache, matchers, alloc,
            ),
            MatcherType::Inverted(child) => {
                apply_inverted(self, source, output, pos, depth, *child, matchers, alloc)
            }
            MatcherType::Wrapper(child) => {
                apply_wrapper(source, output, pos, depth, *child, matchers, alloc)
            }
            MatcherType::Eof => apply_eof(source, pos),
            MatcherType::Newline => apply_newline(self, output, source, pos, alloc),
            MatcherType::Placeholder => unreachable!(),
        }
    }

    pub fn children(&mut self) -> Option<Vec<&mut usize>> {
        match &mut self.matcher_type {
            MatcherType::String(_, _) => None,
            MatcherType::CharSet(_, _) => None,
            MatcherType::CharRange(_, _) => None,
            MatcherType::List(children) => Some(children.iter_mut().collect()),
            MatcherType::Choice(children, _) => Some(children.iter_mut().collect()),
            MatcherType::Repeating(child, _, _) => Some(vec![child]),
            MatcherType::Inverted(child) => Some(vec![child]),
            MatcherType::Wrapper(child) => Some(vec![child]),
            MatcherType::Eof => None,
            MatcherType::Placeholder => None,
            MatcherType::Newline => None,
        }
    }

    pub fn can_start_with(&self, c: char, matchers: &[Matcher]) -> bool {
        match &self.matcher_type {
            MatcherType::String(s, case_sensitive) => {
                if s.is_empty() {
                    return true;
                }
                if *case_sensitive {
                    s[0] == c
                } else {
                    s[0].eq_ignore_ascii_case(&c)
                }
            }
            MatcherType::CharSet(chars, inverted) => chars.contains(&c) ^ inverted,
            MatcherType::CharRange(range, inverted) => range.contains(&c) ^ inverted,
            MatcherType::List(children) => {
                for child in children {
                    let child = &matchers[*child];
                    match &child.matcher_type {
                        MatcherType::Repeating(_, r, _) => {
                            let matches = child.can_start_with(c, matchers);
                            if *r.start() == 0 && !matches {
                                continue;
                            }
                            return matches;
                        }
                        _ => return child.can_start_with(c, matchers),
                    }
                }
                false
            }
            MatcherType::Choice(children, cache) => cache
                .as_ref()
                .filter(|_| c.is_ascii())
                .map(|cache| !cache[c as u32 as usize].is_empty())
                .unwrap_or_else(|| {
                    children
                        .iter()
                        .map(|c| &matchers[*c])
                        .any(|m| m.can_start_with(c, matchers))
                }),
            MatcherType::Repeating(child, _, _) | MatcherType::Wrapper(child) => {
                matchers[*child].can_start_with(c, matchers)
            }
            MatcherType::Inverted(child) => !matchers[*child].can_start_with(c, matchers),
            MatcherType::Eof => false,
            MatcherType::Newline => c == '\n' || c == '\r',
            MatcherType::Placeholder => unreachable!(),
        }
    }

    fn push_token<'a>(&self, output: &mut TokenOutput<'a>, token: Token<'a>) {
        match self.cull_strategy {
            CullStrategy::DeleteAll | CullStrategy::LiftChildren => (),
            _ => output.tokens.push(token),
        }
    }

    fn process_children<'a>(
        &self,
        source: Arc<[char]>,
        range: Range<usize>,
        output: &mut TokenOutput<'a>,
        start: usize,
        alloc: &'a Bump,
    ) {
        match self.cull_strategy {
            CullStrategy::None => {
                self.create_parent(source, range, &mut output.tokens, start, alloc)
            }
            CullStrategy::DeleteAll => (),
            CullStrategy::LiftChildren => (),
            CullStrategy::LiftAtMost(n) => {
                if output.len() - start > n {
                    self.create_parent(source, range, &mut output.tokens, start, alloc);
                }
            }
            CullStrategy::DeleteChildren => {
                output.tokens.truncate(start);
                output.tokens.push(self.create_token(source, range, alloc));
            }
        }
    }

    fn create_parent<'a>(
        &self,
        source: Arc<[char]>,
        range: Range<usize>,
        output: &mut bumpalo::collections::Vec<'a, Token<'a>>,
        start: usize,
        alloc: &'a Bump,
    ) {
        let mut children = bumpalo::collections::Vec::with_capacity_in(output.len() - start, alloc);
        children.extend(output.drain(start..));
        let mut token = self.create_token(source, range, alloc);
        token.children = children;
        output.push(token);
    }

    fn create_token<'a>(
        &self,
        source: Arc<[char]>,
        range: Range<usize>,
        alloc: &'a Bump,
    ) -> Token<'a> {
        Token {
            matcher_name: self.name.clone(),
            matcher_id: self.id,
            children: bumpalo::collections::Vec::new_in(alloc),
            source,
            range,
        }
    }
}

fn char_matches(first: &char, second: &char, case_sensitive: bool) -> bool {
    if case_sensitive {
        first == second
    } else {
        first == second || first.eq_ignore_ascii_case(second)
    }
}

fn next_depth(matcher: &Matcher, depth: usize) -> usize {
    match &*matcher.name {
        Some(_) => depth + 1,
        None => depth,
    }
}

fn apply_string<'a>(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput<'a>,
    pos: usize,
    depth: usize,
    to_match: &[char],
    case_sensitive: bool,
    alloc: &'a Bump,
) -> TokenResult {
    let matched_chars = to_match
        .iter()
        .zip(&source[pos..])
        .take_while(|(a, b)| char_matches(a, b, case_sensitive))
        .count();

    if matched_chars > 0 {
        output.mark_success(pos, pos + matched_chars, depth, matcher);
    }
    if matched_chars == to_match.len() {
        let range = pos..pos + to_match.len();
        matcher.push_token(output, matcher.create_token(source, range.clone(), alloc));
        Some(range)
    } else {
        None
    }
}

fn apply_char_set<'a>(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput<'a>,
    pos: usize,
    to_match: &[char],
    inverted: bool,
    alloc: &'a Bump,
) -> TokenResult {
    match source.get(pos) {
        Some(c) if to_match.contains(c) ^ inverted => {
            let range = pos..pos + 1;
            matcher.push_token(output, matcher.create_token(source, range.clone(), alloc));
            Some(range)
        }
        _ => None,
    }
}

fn apply_char_range<'a>(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput<'a>,
    pos: usize,
    range: &RangeInclusive<char>,
    inverted: bool,
    alloc: &'a Bump,
) -> TokenResult {
    match source.get(pos) {
        Some(c) if range.contains(c) ^ inverted => {
            let range = pos..pos + 1;
            matcher.push_token(output, matcher.create_token(source, range.clone(), alloc));
            Some(range)
        }
        _ => None,
    }
}

fn apply_list<'a>(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput<'a>,
    pos: usize,
    depth: usize,
    children: &[usize],
    matchers: &[Matcher],
    alloc: &'a Bump,
) -> TokenResult {
    let mut cursor = pos;
    let output_start = output.len();
    for child in children {
        let child = &matchers[*child];
        match child.apply(
            source.clone(),
            output,
            matchers,
            cursor,
            next_depth(matcher, depth),
            alloc,
        ) {
            Some(child_token) => {
                cursor = child_token.end;
                output.mark_success(pos, cursor, depth, matcher);
            }
            None => {
                output.tokens.truncate(output_start);
                return None;
            }
        }
    }
    let range = pos..cursor;
    matcher.process_children(source, range.clone(), output, output_start, alloc);
    Some(range)
}

fn apply_choice<'a>(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput<'a>,
    pos: usize,
    depth: usize,
    children: &[usize],
    matchers: &[Matcher],
    alloc: &'a Bump,
) -> TokenResult {
    let output_start = output.len();
    output.mark_success(pos, pos, depth, matcher);
    for child in children {
        let child = &matchers[*child];
        let matched = child.apply(
            source.clone(),
            output,
            matchers,
            pos,
            next_depth(matcher, depth),
            alloc,
        );
        match matched {
            Some(range) => {
                matcher.process_children(source, range.clone(), output, output_start, alloc);
                return Some(range);
            }
            None => (),
        }
    }
    None
}

fn apply_repeating<'a>(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput<'a>,
    pos: usize,
    depth: usize,
    child: usize,
    range: &RangeInclusive<usize>,
    cache: &Option<[bool; 256]>,
    matchers: &[Matcher],
    alloc: &'a Bump,
) -> TokenResult {
    let output_start = output.len();
    let child = &matchers[child];
    let mut cursor = pos;
    let mut child_count = 0;
    while child_count < *range.end() {
        if source
            .get(pos)
            .filter(|c| c.is_ascii())
            .and_then(|c| cache.as_ref().map(|cache| cache[*c as u32 as usize]))
            == Some(false)
        {
            break;
        }
        let matched = child.apply(
            source.clone(),
            output,
            matchers,
            cursor,
            next_depth(matcher, depth),
            alloc,
        );
        match matched {
            Some(child_token) => {
                cursor = child_token.end;
                child_count += 1;
                output.mark_success(pos, cursor, depth, matcher);
                if child_token.is_empty() {
                    break;
                }
            }
            None => break,
        }
    }

    if child_count < *range.start() {
        output.tokens.truncate(output_start);
        None
    } else {
        let range = pos..cursor;
        matcher.process_children(source, range.clone(), output, output_start, alloc);
        Some(range)
    }
}

fn apply_eof(source: Arc<[char]>, pos: usize) -> TokenResult {
    (pos == source.len()).then(|| pos..pos)
}

fn apply_inverted<'a>(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput<'a>,
    pos: usize,
    depth: usize,
    child: usize,
    matchers: &[Matcher],
    alloc: &'a Bump,
) -> TokenResult {
    let child = &matchers[child];
    let matched = child.apply(
        source.clone(),
        output,
        matchers,
        pos,
        next_depth(matcher, depth),
        alloc,
    );
    let output_start = output.len();
    match matched {
        Some(_) => {
            output.tokens.truncate(output_start);
            None
        }
        None => {
            let range = pos..pos;
            matcher.push_token(output, matcher.create_token(source, range.clone(), alloc));
            Some(range)
        }
    }
}

fn apply_newline<'a>(
    matcher: &Matcher,
    output: &mut TokenOutput<'a>,
    source: Arc<[char]>,
    pos: usize,
    alloc: &'a Bump,
) -> TokenResult {
    match source.get(pos) {
        Some('\r') => {
            let range = if let Some('\n') = source.get(pos + 1) {
                pos..pos + 2
            } else {
                pos..pos + 1
            };
            matcher.push_token(output, matcher.create_token(source, range.clone(), alloc));
            Some(range)
        }
        Some('\n') => {
            let range = pos..pos + 1;
            matcher.push_token(output, matcher.create_token(source, range.clone(), alloc));
            Some(range)
        }
        _ => None,
    }
}

fn apply_wrapper<'a>(
    source: Arc<[char]>,
    output: &mut TokenOutput<'a>,
    pos: usize,
    depth: usize,
    child: usize,
    matchers: &[Matcher],
    alloc: &'a Bump,
) -> TokenResult {
    let child = &matchers[child];
    child.apply(source, output, matchers, pos, depth, alloc)
}
