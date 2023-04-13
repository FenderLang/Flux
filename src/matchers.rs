use std::ops::Range;
use std::{ops::RangeInclusive, sync::Arc};

use bumpalo::Bump;

use crate::error::FluxError;
use crate::lexer::CullStrategy;
use crate::tokens::Token;

pub type MatcherName = Arc<Option<String>>;
pub type TokenResult = Option<Range<usize>>;

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct MatcherContext<'a, 'b: 'a> {
    pub source: Arc<[char]>,
    pub pos: usize,
    pub depth: usize,
    pub output: &'a mut TokenOutput<'b>,
    pub alloc: &'b Bump,
}

impl Matcher {
    pub fn apply<'a>(&self, context: MatcherContext, matchers: &[Matcher]) -> TokenResult {
        match &self.matcher_type {
            MatcherType::String(to_match, case_sensitive) => {
                self.apply_string(context, to_match, *case_sensitive)
            }
            MatcherType::CharSet(chars, inverted) => self.apply_char_set(context, chars, *inverted),
            MatcherType::CharRange(range, inverted) => {
                self.apply_char_range(context, range, *inverted)
            }
            MatcherType::List(children) => self.apply_list(context, children, matchers),
            MatcherType::Choice(children, cache) => {
                let choice_children = context
                    .source
                    .get(context.pos)
                    .filter(|c| c.is_ascii())
                    .map(|c| *c as u32 as usize)
                    .and_then(|c| cache.as_ref().map(|cache| &cache[c]))
                    .unwrap_or(children);
                self.apply_choice(context, choice_children, matchers)
            }
            MatcherType::Repeating(child, range, cache) => {
                self.apply_repeating(context, *child, range, cache, matchers)
            }
            MatcherType::Inverted(child) => self.apply_inverted(context, *child, matchers),
            MatcherType::Wrapper(child) => Self::apply_wrapper(context, *child, matchers),
            MatcherType::Eof => Self::apply_eof(context.source, context.pos),
            MatcherType::Newline => self.apply_newline(context),
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

impl Matcher {
    fn apply_string<'a>(
        &self,
        context: MatcherContext<'_, 'a>,
        to_match: &[char],
        case_sensitive: bool,
    ) -> TokenResult {
        let matched_chars = to_match
            .iter()
            .zip(&context.source[context.pos..])
            .take_while(|(a, b)| char_matches(a, b, case_sensitive))
            .count();

        if matched_chars > 0 {
            context.output.mark_success(
                context.pos,
                context.pos + matched_chars,
                context.depth,
                self,
            );
        }
        if matched_chars == to_match.len() {
            let range = context.pos..context.pos + to_match.len();
            self.push_token(
                context.output,
                self.create_token(context.source.clone(), range.clone(), &context.alloc),
            );
            Some(range)
        } else {
            None
        }
    }

    fn apply_char_set<'a>(
        &self,
        context: MatcherContext<'_, 'a>,
        to_match: &[char],
        inverted: bool,
    ) -> TokenResult {
        match context.source.get(context.pos) {
            Some(c) if to_match.contains(c) ^ inverted => {
                let range = context.pos..context.pos + 1;
                self.push_token(
                    context.output,
                    self.create_token(context.source, range.clone(), context.alloc),
                );
                Some(range)
            }
            _ => None,
        }
    }

    fn apply_char_range<'a>(
        &self,
        context: MatcherContext<'_, 'a>,
        range: &RangeInclusive<char>,
        inverted: bool,
    ) -> TokenResult {
        match context.source.get(context.pos) {
            Some(c) if range.contains(c) ^ inverted => {
                let range = context.pos..context.pos + 1;
                self.push_token(
                    context.output,
                    self.create_token(context.source, range.clone(), context.alloc),
                );
                Some(range)
            }
            _ => None,
        }
    }

    fn apply_list<'a>(
        &self,
        mut context: MatcherContext<'_, 'a>,
        children: &[usize],
        matchers: &[Matcher],
    ) -> TokenResult {
        let mut cursor = context.pos;
        let output_start = context.output.len();
        for child in children {
            let child = &matchers[*child];
            match child.apply(
                MatcherContext {
                    source: context.source.clone(),
                    pos: cursor,
                    depth: next_depth(self, context.depth),
                    output: context.output,
                    alloc: &context.alloc,
                },
                matchers,
            ) {
                Some(child_token) => {
                    cursor = child_token.end;
                    context
                        .output
                        .mark_success(context.pos, cursor, context.depth, self);
                }
                None => {
                    context.output.tokens.truncate(output_start);
                    return None;
                }
            }
        }
        let range = context.pos..cursor;
        self.process_children(
            context.source.clone(),
            range.clone(),
            &mut context.output,
            output_start,
            context.alloc,
        );
        Some(range)
    }

    fn apply_choice<'a>(
        &self,
        mut context: MatcherContext<'_, 'a>,
        children: &[usize],
        matchers: &[Matcher],
    ) -> TokenResult {
        let output_start = context.output.len();
        context
            .output
            .mark_success(context.pos, context.pos, context.depth, self);
        for child in children {
            let child = &matchers[*child];
            let matched = child.apply(
                MatcherContext {
                    source: context.source.clone(),
                    pos: context.pos,
                    depth: next_depth(self, context.depth),
                    output: &mut context.output,
                    alloc: &context.alloc,
                },
                matchers,
            );
            match matched {
                Some(range) => {
                    self.process_children(
                        context.source,
                        range.clone(),
                        context.output,
                        output_start,
                        context.alloc,
                    );
                    return Some(range);
                }
                None => (),
            }
        }
        None
    }

    fn apply_repeating<'a>(
        &self,
        mut context: MatcherContext<'_, 'a>,
        child: usize,
        range: &RangeInclusive<usize>,
        cache: &Option<[bool; 256]>,
        matchers: &[Matcher],
    ) -> TokenResult {
        let output_start = context.output.len();
        let child = &matchers[child];
        let mut cursor = context.pos;
        let mut child_count = 0;
        while child_count < *range.end() {
            if context
                .source
                .get(context.pos)
                .filter(|c| c.is_ascii())
                .and_then(|c| cache.as_ref().map(|cache| cache[*c as u32 as usize]))
                == Some(false)
            {
                break;
            }
            let matched = child.apply(
                MatcherContext {
                    source: context.source.clone(),
                    pos: cursor,
                    depth: context.depth,
                    output: &mut context.output,
                    alloc: context.alloc,
                },
                matchers,
            );
            match matched {
                Some(child_token) => {
                    cursor = child_token.end;
                    child_count += 1;
                    context
                        .output
                        .mark_success(context.pos, cursor, context.depth, self);
                    if child_token.is_empty() {
                        break;
                    }
                }
                None => break,
            }
        }

        if child_count < *range.start() {
            context.output.tokens.truncate(output_start);
            None
        } else {
            let range = context.pos..cursor;
            self.process_children(
                context.source,
                range.clone(),
                &mut context.output,
                output_start,
                context.alloc,
            );
            Some(range)
        }
    }

    fn apply_eof(source: Arc<[char]>, pos: usize) -> TokenResult {
        (pos == source.len()).then(|| pos..pos)
    }

    fn apply_inverted<'a>(
        &self,
        mut context: MatcherContext<'_, 'a>,
        child: usize,
        matchers: &[Matcher],
    ) -> TokenResult {
        let child = &matchers[child];
        let matched = child.apply(
            MatcherContext {
                source: context.source.clone(),
                pos: context.pos,
                depth: context.depth,
                output: &mut context.output,
                alloc: context.alloc,
            },
            matchers,
        );
        let output_start = context.output.len();
        match matched {
            Some(_) => {
                context.output.tokens.truncate(output_start);
                None
            }
            None => {
                let range = context.pos..context.pos;
                self.push_token(
                    context.output,
                    self.create_token(context.source, range.clone(), context.alloc),
                );
                Some(range)
            }
        }
    }

    fn apply_newline<'a>(&self, context: MatcherContext<'_, 'a>) -> TokenResult {
        match context.source.get(context.pos) {
            Some('\r') => {
                let range = if let Some('\n') = context.source.get(context.pos + 1) {
                    context.pos..context.pos + 2
                } else {
                    context.pos..context.pos + 1
                };
                self.push_token(
                    context.output,
                    self.create_token(context.source, range.clone(), context.alloc),
                );
                Some(range)
            }
            Some('\n') => {
                let range = context.pos..context.pos + 1;
                self.push_token(
                    context.output,
                    self.create_token(context.source, range.clone(), context.alloc),
                );
                Some(range)
            }
            _ => None,
        }
    }

    fn apply_wrapper<'a>(
        context: MatcherContext<'_, 'a>,
        child: usize,
        matchers: &[Matcher],
    ) -> TokenResult {
        let child = &matchers[child];
        child.apply(context, matchers)
    }
}
