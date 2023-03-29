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
pub struct MatcherFuncArgs<'a> {
    pub source: Arc<[char]>,
    pub pos: usize,
    pub depth: usize,
    pub output: &'a mut TokenOutput,
}

impl Matcher {
    pub fn apply(&self, func_args: MatcherFuncArgs, matchers: &[Matcher]) -> TokenResult {
        match &self.matcher_type {
            MatcherType::String(to_match, case_sensitive) => {
                self.apply_string(func_args, to_match, *case_sensitive)
            }
            MatcherType::CharSet(chars, inverted) => {
                self.apply_char_set(func_args, chars, *inverted)
            }
            MatcherType::CharRange(range, inverted) => {
                self.apply_char_range(func_args, range, *inverted)
            }
            MatcherType::List(children) => self.apply_list(func_args, children, matchers),
            MatcherType::Choice(children) => self.apply_choice(func_args, children, matchers),
            MatcherType::Repeating(child, range) => {
                self.apply_repeating(func_args, *child, range, matchers)
            }
            MatcherType::Inverted(child) => self.apply_inverted(func_args, *child, matchers),
            MatcherType::Wrapper(child) => Matcher::apply_wrapper(func_args, *child, matchers),
            MatcherType::Eof => Matcher::apply_eof(func_args.source, func_args.pos),
            MatcherType::Newline => self.apply_newline(func_args),
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
        func_args: MatcherFuncArgs,

        to_match: &[char],
        case_sensitive: bool,
    ) -> TokenResult {
        let matched_chars = to_match
            .iter()
            .zip(&func_args.source[func_args.pos..])
            .take_while(|(a, b)| char_matches(a, b, case_sensitive))
            .count();

        if matched_chars > 0 {
            func_args.output.mark_success(
                func_args.pos,
                func_args.pos + matched_chars,
                func_args.depth,
                self,
            );
        }
        if matched_chars == to_match.len() {
            let range = func_args.pos..func_args.pos + to_match.len();
            self.push_token(
                func_args.output,
                self.create_token(func_args.source, range.clone()),
            );
            Some(range)
        } else {
            None
        }
    }

    fn apply_char_set(
        &self,
        func_args: MatcherFuncArgs,

        to_match: &[char],
        inverted: bool,
    ) -> TokenResult {
        match func_args.source.get(func_args.pos) {
            Some(c) if to_match.contains(c) ^ inverted => {
                let range = func_args.pos..func_args.pos + 1;
                self.push_token(
                    func_args.output,
                    self.create_token(func_args.source, range.clone()),
                );
                Some(range)
            }
            _ => None,
        }
    }

    fn apply_char_range(
        &self,
        func_args: MatcherFuncArgs,
        range: &RangeInclusive<char>,
        inverted: bool,
    ) -> TokenResult {
        match func_args.source.get(func_args.pos) {
            Some(c) if range.contains(c) ^ inverted => {
                let range = func_args.pos..func_args.pos + 1;
                self.push_token(
                    func_args.output,
                    self.create_token(func_args.source, range.clone()),
                );
                Some(range)
            }
            _ => None,
        }
    }

    fn apply_list(
        &self,
        func_args: MatcherFuncArgs,
        children: &[usize],
        matchers: &[Matcher],
    ) -> TokenResult {
        let mut cursor = func_args.pos;
        let output_start = func_args.output.len();
        for child in children {
            let child = &matchers[*child];
            match child.apply(
                MatcherFuncArgs {
                    source: func_args.source.clone(),
                    pos: cursor,
                    depth: self.next_depth(func_args.depth),
                    output: func_args.output,
                },
                matchers,
            ) {
                Some(child_token) => {
                    cursor = child_token.end;
                    func_args
                        .output
                        .mark_success(func_args.pos, cursor, func_args.depth, self);
                }
                None => {
                    func_args.output.tokens.truncate(output_start);
                    return None;
                }
            }
        }
        let range = func_args.pos..cursor;
        self.process_children(
            func_args.source,
            range.clone(),
            func_args.output,
            output_start,
        );
        Some(range)
    }

    fn apply_choice(
        &self,
        func_args: MatcherFuncArgs,
        children: &[usize],
        matchers: &[Matcher],
    ) -> TokenResult {
        let output_start = func_args.output.len();
        func_args
            .output
            .mark_success(func_args.pos, func_args.pos, func_args.depth, self);
        for child in children {
            let child = &matchers[*child];
            let matched = child.apply(
                MatcherFuncArgs {
                    source: func_args.source.clone(),
                    depth: self.next_depth(func_args.depth),
                    output: func_args.output,
                    ..func_args
                },
                matchers,
            );
            if let Some(range) = matched {
                self.process_children(
                    func_args.source,
                    range.clone(),
                    func_args.output,
                    output_start,
                );
                return Some(range);
            }
        }
        None
    }

    fn apply_repeating(
        &self,
        func_args: MatcherFuncArgs,
        child: usize,
        range: &RangeInclusive<usize>,
        matchers: &[Matcher],
    ) -> TokenResult {
        let output_start = func_args.output.len();
        let child = &matchers[child];
        let mut cursor = func_args.pos;
        let mut child_count = 0;
        while child_count < *range.end() {
            let matched = child.apply(
                MatcherFuncArgs {
                    source: func_args.source.clone(),
                    pos: cursor,
                    depth: self.next_depth(func_args.depth),
                    output: func_args.output,
                },
                matchers,
            );
            match matched {
                Some(child_token) => {
                    cursor = child_token.end;
                    child_count += 1;
                    func_args
                        .output
                        .mark_success(func_args.pos, cursor, func_args.depth, self);
                    if child_token.is_empty() {
                        break;
                    }
                }
                None => break,
            }
        }

        if child_count < *range.start() {
            func_args.output.tokens.truncate(output_start);
            None
        } else {
            let range = func_args.pos..cursor;
            self.process_children(
                func_args.source,
                range.clone(),
                func_args.output,
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
        func_args: MatcherFuncArgs,
        child: usize,
        matchers: &[Matcher],
    ) -> TokenResult {
        let child = &matchers[child];
        let matched = child.apply(
            MatcherFuncArgs {
                source: func_args.source.clone(),
                depth: self.next_depth(func_args.depth),
                output: func_args.output,
                ..func_args
            },
            matchers,
        );
        let output_start = func_args.output.len();
        match matched {
            Some(_) => {
                func_args.output.tokens.truncate(output_start);
                None
            }
            None => {
                let range = func_args.pos..func_args.pos;
                self.push_token(
                    func_args.output,
                    self.create_token(func_args.source, range.clone()),
                );
                Some(range)
            }
        }
    }

    fn apply_newline(&self, func_args: MatcherFuncArgs) -> TokenResult {
        match func_args.source.get(func_args.pos) {
            Some('\r') => {
                let range = if let Some('\n') = func_args.source.get(func_args.pos + 1) {
                    func_args.pos..func_args.pos + 2
                } else {
                    func_args.pos..func_args.pos + 1
                };
                self.push_token(
                    func_args.output,
                    self.create_token(func_args.source, range.clone()),
                );
                Some(range)
            }
            Some('\n') => {
                let range = func_args.pos..func_args.pos + 1;
                self.push_token(
                    func_args.output,
                    self.create_token(func_args.source, range.clone()),
                );
                Some(range)
            }
            _ => None,
        }
    }

    fn apply_wrapper(
        func_args: MatcherFuncArgs,
        child: usize,
        matchers: &[Matcher],
    ) -> TokenResult {
        let child = &matchers[child];
        child.apply(
            MatcherFuncArgs {
                source: func_args.source.clone(),
                depth: func_args.pos,
                ..func_args
            },
            matchers,
        )
    }
}
