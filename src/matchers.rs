use std::ops::Range;
use std::{ops::RangeInclusive, sync::Arc};

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

pub struct TokenOutput {
    pub(crate) tokens: Vec<Token>,
    pub(crate) last_success: SuccessMark,
}

impl TokenOutput {
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
    Choice(Vec<usize>),
    Repeating(usize, RangeInclusive<usize>),
    Inverted(usize),
    Wrapper(usize),
    Eof,
    Newline,
    Placeholder,
}

impl Matcher {
    pub fn apply(
        &self,
        source: Arc<[char]>,
        output: &mut TokenOutput,
        matchers: &[Matcher],
        pos: usize,
        depth: usize,
    ) -> TokenResult {
        match &self.matcher_type {
            MatcherType::String(to_match, case_sensitive) => {
                apply_string(self, source, output, pos, depth, to_match, *case_sensitive)
            }
            MatcherType::CharSet(chars, inverted) => {
                apply_char_set(self, source, output, pos, chars, *inverted)
            }
            MatcherType::CharRange(range, inverted) => {
                apply_char_range(self, source, output, pos, range, *inverted)
            }
            MatcherType::List(children) => {
                apply_list(self, source, output, pos, depth, children, matchers)
            }
            MatcherType::Choice(children) => {
                apply_choice(self, source, output, pos, depth, children, matchers)
            }
            MatcherType::Repeating(child, range) => {
                apply_repeating(self, source, output, pos, depth, *child, range, matchers)
            }
            MatcherType::Inverted(child) => {
                apply_inverted(self, source, output, pos, depth, *child, matchers)
            }
            MatcherType::Wrapper(child) => {
                apply_wrapper(source, output, pos, depth, *child, matchers)
            }
            MatcherType::Eof => apply_eof(source, pos),
            MatcherType::Newline => apply_newline(self, output, source, pos),
            MatcherType::Placeholder => unreachable!(),
        }
    }

    pub fn children(&mut self) -> Option<Vec<&mut usize>> {
        match &mut self.matcher_type {
            MatcherType::String(_, _) => None,
            MatcherType::CharSet(_, _) => None,
            MatcherType::CharRange(_, _) => None,
            MatcherType::List(children) => Some(children.iter_mut().collect()),
            MatcherType::Choice(children) => Some(children.iter_mut().collect()),
            MatcherType::Repeating(child, _) => Some(vec![child]),
            MatcherType::Inverted(child) => Some(vec![child]),
            MatcherType::Wrapper(child) => Some(vec![child]),
            MatcherType::Eof => None,
            MatcherType::Placeholder => None,
            MatcherType::Newline => None,
        }
    }

    fn push_token(&self, output: &mut TokenOutput, token: Token) {
        match self.cull_strategy {
            CullStrategy::DeleteAll | CullStrategy::LiftChildren => (),
            _ => output.tokens.push(token),
        }
    }

    fn process_children(
        &self,
        source: Arc<[char]>,
        range: Range<usize>,
        output: &mut TokenOutput,
        start: usize,
    ) {
        match self.cull_strategy {
            CullStrategy::None => self.create_parent(source, range, &mut output.tokens, start),
            CullStrategy::DeleteAll => (),
            CullStrategy::LiftChildren => (),
            CullStrategy::LiftAtMost(n) => {
                if output.len() - start > n {
                    self.create_parent(source, range, &mut output.tokens, start);
                }
            }
            CullStrategy::DeleteChildren => {
                output.tokens.drain(start..);
                output.tokens.push(self.create_token(source, range));
            }
        }
    }

    fn create_parent(
        &self,
        source: Arc<[char]>,
        range: Range<usize>,
        output: &mut Vec<Token>,
        start: usize,
    ) {
        let children: Vec<_> = output.drain(start..).collect();
        let mut token = self.create_token(source, range);
        token.children = children;
        output.push(token);
    }

    fn create_token(&self, source: Arc<[char]>, range: Range<usize>) -> Token {
        Token {
            matcher_name: self.name.clone(),
            matcher_id: self.id,
            children: Vec::with_capacity(0),
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

fn apply_string(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    to_match: &[char],
    case_sensitive: bool,
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
        matcher.push_token(output, matcher.create_token(source, range.clone()));
        Some(range)
    } else {
        None
    }
}

fn apply_char_set(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput,
    pos: usize,
    to_match: &[char],
    inverted: bool,
) -> TokenResult {
    match source.get(pos) {
        Some(c) if to_match.contains(c) ^ inverted => {
            let range = pos..pos + 1;
            matcher.push_token(output, matcher.create_token(source, range.clone()));
            Some(range)
        }
        _ => None,
    }
}

fn apply_char_range(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput,
    pos: usize,
    range: &RangeInclusive<char>,
    inverted: bool,
) -> TokenResult {
    match source.get(pos) {
        Some(c) if range.contains(c) ^ inverted => {
            let range = pos..pos + 1;
            matcher.push_token(output, matcher.create_token(source, range.clone()));
            Some(range)
        }
        _ => None,
    }
}

fn apply_list(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    children: &[usize],
    matchers: &[Matcher],
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
        ) {
            Some(child_token) => {
                cursor = child_token.end;
                output.mark_success(pos, cursor, depth, matcher);
            }
            None => {
                output.tokens.drain(output_start..);
                return None;
            }
        }
    }
    let range = pos..cursor;
    matcher.process_children(source, range.clone(), output, output_start);
    Some(range)
}

fn apply_choice(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    children: &[usize],
    matchers: &[Matcher],
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
        );
        match matched {
            Some(range) => {
                matcher.process_children(source, range.clone(), output, output_start);
                return Some(range);
            }
            None => (),
        }
    }
    None
}

fn apply_repeating(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    child: usize,
    range: &RangeInclusive<usize>,
    matchers: &[Matcher],
) -> TokenResult {
    let output_start = output.len();
    let child = &matchers[child];
    let mut cursor = pos;
    let mut child_count = 0;
    while child_count < *range.end() {
        let matched = child.apply(
            source.clone(),
            output,
            matchers,
            cursor,
            next_depth(matcher, depth),
        );
        match matched {
            Some(child_token) => {
                cursor = child_token.end;
                child_count += 1;
                output.mark_success(pos, cursor, depth, matcher);
            }
            None => break,
        }
    }

    if child_count < *range.start() {
        output.tokens.drain(output_start..);
        None
    } else {
        let range = pos..cursor;
        matcher.process_children(source, range.clone(), output, output_start);
        Some(range)
    }
}

fn apply_eof(source: Arc<[char]>, pos: usize) -> TokenResult {
    (pos == source.len()).then(|| pos..pos)
}

fn apply_inverted(
    matcher: &Matcher,
    source: Arc<[char]>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    child: usize,
    matchers: &[Matcher],
) -> TokenResult {
    let child = &matchers[child];
    let matched = child.apply(
        source.clone(),
        output,
        matchers,
        pos,
        next_depth(matcher, depth),
    );
    let output_start = output.len();
    match matched {
        Some(_) => {
            output.tokens.drain(output_start..);
            None
        }
        None => {
            let range = pos..pos;
            matcher.push_token(output, matcher.create_token(source, range.clone()));
            Some(range)
        }
    }
}

fn apply_newline(
    matcher: &Matcher,
    output: &mut TokenOutput,
    source: Arc<[char]>,
    pos: usize,
) -> TokenResult {
    match source.get(pos) {
        Some('\r') => {
            let range = if let Some('\n') = source.get(pos + 1) {
                pos..pos + 2
            } else {
                pos..pos + 1
            };
            matcher.push_token(output, matcher.create_token(source, range.clone()));
            Some(range)
        }
        Some('\n') => {
            let range = pos..pos + 1;
            matcher.push_token(output, matcher.create_token(source, range.clone()));
            Some(range)
        }
        _ => None,
    }
}

fn apply_wrapper(
    source: Arc<[char]>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    child: usize,
    matchers: &[Matcher],
) -> TokenResult {
    let child = &matchers[child];
    child.apply(source, output, matchers, pos, depth)
}
