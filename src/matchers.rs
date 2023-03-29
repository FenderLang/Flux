use std::ops::Range;
use std::{ops::RangeInclusive, sync::Arc};

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

#[derive(Debug)]
pub struct MatcherContext<'a> {
    pub source: Arc<[char]>,
    pub pos: usize,
    pub depth: usize,
    pub output: &'a mut TokenOutput,
}

impl Matcher {
    pub fn apply(&self, context: MatcherContext, matchers: &[Matcher]) -> TokenResult {
        match &self.matcher_type {
            MatcherType::String(to_match, case_sensitive) => {
                self.apply_string(context, to_match, *case_sensitive)
            }
            MatcherType::CharSet(chars, inverted) => {
                self.apply_char_set(context, chars, *inverted)
            }
            MatcherType::CharRange(range, inverted) => {
                self.apply_char_range(context, range, *inverted)
            }
            MatcherType::List(children) => self.apply_list(context, children, matchers),
            MatcherType::Choice(children) => self.apply_choice(context, children, matchers),
            MatcherType::Repeating(child, range) => {
                self.apply_repeating(context, *child, range, matchers)
            }
            MatcherType::Inverted(child) => self.apply_inverted(context, *child, matchers),
            MatcherType::Wrapper(child) => Matcher::apply_wrapper(context, *child, matchers),
            MatcherType::Eof => Matcher::apply_eof(context.source, context.pos),
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
                output.tokens.truncate(start);
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

impl Matcher {
    fn next_depth(&self, depth: usize) -> usize {
        match &*self.name {
            Some(_) => depth + 1,
            None => depth,
        }
    }

    fn apply_string(
        &self,
        context: MatcherContext,

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
                self.create_token(context.source, range.clone()),
            );
            Some(range)
        } else {
            None
        }
    }

    fn apply_char_set(
        &self,
        context: MatcherContext,

        to_match: &[char],
        inverted: bool,
    ) -> TokenResult {
        match context.source.get(context.pos) {
            Some(c) if to_match.contains(c) ^ inverted => {
                let range = context.pos..context.pos + 1;
                self.push_token(
                    context.output,
                    self.create_token(context.source, range.clone()),
                );
                Some(range)
            }
            _ => None,
        }
    }

    fn apply_char_range(
        &self,
        context: MatcherContext,
        range: &RangeInclusive<char>,
        inverted: bool,
    ) -> TokenResult {
        match context.source.get(context.pos) {
            Some(c) if range.contains(c) ^ inverted => {
                let range = context.pos..context.pos + 1;
                self.push_token(
                    context.output,
                    self.create_token(context.source, range.clone()),
                );
                Some(range)
            }
            _ => None,
        }
    }

    fn apply_list(
        &self,
        context: MatcherContext,
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
                    depth: self.next_depth(context.depth),
                    output: context.output,
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
            context.source,
            range.clone(),
            context.output,
            output_start,
        );
        Some(range)
    }

    fn apply_choice(
        &self,
        context: MatcherContext,
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
                    depth: self.next_depth(context.depth),
                    output: context.output,
                    ..context
                },
                matchers,
            );
            if let Some(range) = matched {
                self.process_children(
                    context.source,
                    range.clone(),
                    context.output,
                    output_start,
                );
                return Some(range);
            }
        }
        None
    }

    fn apply_repeating(
        &self,
        context: MatcherContext,
        child: usize,
        range: &RangeInclusive<usize>,
        matchers: &[Matcher],
    ) -> TokenResult {
        let output_start = context.output.len();
        let child = &matchers[child];
        let mut cursor = context.pos;
        let mut child_count = 0;
        while child_count < *range.end() {
            let matched = child.apply(
                MatcherContext {
                    source: context.source.clone(),
                    pos: cursor,
                    depth: self.next_depth(context.depth),
                    output: context.output,
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
                context.output,
                output_start,
            );
            Some(range)
        }
    }

    fn apply_eof(source: Arc<[char]>, pos: usize) -> TokenResult {
        (pos == source.len()).then(|| pos..pos)
    }

    fn apply_inverted(
        &self,
        context: MatcherContext,
        child: usize,
        matchers: &[Matcher],
    ) -> TokenResult {
        let child = &matchers[child];
        let matched = child.apply(
            MatcherContext {
                source: context.source.clone(),
                depth: self.next_depth(context.depth),
                output: context.output,
                ..context
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
                    self.create_token(context.source, range.clone()),
                );
                Some(range)
            }
        }
    }

    fn apply_newline(&self, context: MatcherContext) -> TokenResult {
        match context.source.get(context.pos) {
            Some('\r') => {
                let range = if let Some('\n') = context.source.get(context.pos + 1) {
                    context.pos..context.pos + 2
                } else {
                    context.pos..context.pos + 1
                };
                self.push_token(
                    context.output,
                    self.create_token(context.source, range.clone()),
                );
                Some(range)
            }
            Some('\n') => {
                let range = context.pos..context.pos + 1;
                self.push_token(
                    context.output,
                    self.create_token(context.source, range.clone()),
                );
                Some(range)
            }
            _ => None,
        }
    }

    fn apply_wrapper(
        context: MatcherContext,
        child: usize,
        matchers: &[Matcher],
    ) -> TokenResult {
        let child = &matchers[child];
        child.apply(
            MatcherContext {
                source: context.source.clone(),
                depth: context.pos,
                ..context
            },
            matchers,
        )
    }
}
