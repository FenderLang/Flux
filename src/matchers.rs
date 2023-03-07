use std::ops::Range;
use std::{ops::RangeInclusive, sync::Arc};

use crate::error::{FluxError, Result};
use crate::lexer::CullStrategy;
use crate::tokens::Token;

pub type MatcherName = Arc<Option<String>>;
pub type TokenResult = Result<Range<usize>>;
type TokenOutput = Vec<Token>;

#[derive(Debug, Clone)]
pub struct Matcher {
    pub(crate) cull_strategy: CullStrategy,
    pub(crate) name: MatcherName,
    pub(crate) id: usize,
    pub(crate) matcher_type: MatcherType,
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
    Placeholder,
}

impl Matcher {
    pub fn apply(
        &self,
        source: Arc<Vec<char>>,
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
                apply_char_set(self, source, output, pos, depth, chars, *inverted)
            }
            MatcherType::CharRange(range, inverted) => {
                apply_char_range(self, source, output, pos, depth, range, *inverted)
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
            MatcherType::Eof => apply_eof(self, source, pos, depth),
            MatcherType::Placeholder => unreachable!(),
        }
    }

    pub fn children(&self) -> Option<Vec<usize>> {
        match &self.matcher_type {
            MatcherType::String(_, _) => None,
            MatcherType::CharSet(_, _) => None,
            MatcherType::CharRange(_, _) => None,
            MatcherType::List(children) => Some(children.clone()),
            MatcherType::Choice(children) => Some(children.clone()),
            MatcherType::Repeating(child, _) => Some(vec![*child]),
            MatcherType::Inverted(child) => Some(vec![*child]),
            MatcherType::Wrapper(child) => Some(vec![*child]),
            MatcherType::Eof => None,
            MatcherType::Placeholder => None,
        }
    }

    fn push_token(&self, output: &mut TokenOutput, token: Token) {
        match self.cull_strategy {
            CullStrategy::DeleteAll | CullStrategy::LiftChildren => (),
            _ => output.push(token),
        }
    }

    fn process_children(
        &self,
        source: Arc<Vec<char>>,
        range: Range<usize>,
        output: &mut TokenOutput,
        start: usize,
    ) {
        // TODO: Error handling
        match self.cull_strategy {
            CullStrategy::None => self.create_parent(source, range, output, start),
            CullStrategy::DeleteAll => (),
            CullStrategy::LiftChildren => (),
            CullStrategy::LiftAtMost(n) => {
                if output.len() - start > n {
                    self.create_parent(source, range, output, start);
                }
            }
            CullStrategy::DeleteChildren => {
                output.drain(start..);
                output.push(self.create_token(source, range));
            }
        }
    }

    fn create_parent(
        &self,
        source: Arc<Vec<char>>,
        range: Range<usize>,
        output: &mut Vec<Token>,
        start: usize,
    ) {
        let children: Vec<_> = output.drain(start + 0..).collect();
        let mut token = self.create_token(source, range);
        token.children = children;
        output.push(token);
    }

    fn create_token(&self, source: Arc<Vec<char>>, range: Range<usize>) -> Token {
        Token {
            matcher_name: self.name.clone(),
            matcher_id: self.id,
            children: Vec::with_capacity(0),
            source,
            range,
            failure: None,
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
    source: Arc<Vec<char>>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    to_match: &[char],
    case_sensitive: bool,
) -> TokenResult {
    let mut compared_strings_zipped = to_match.iter().zip(&source[pos..]);

    if compared_strings_zipped.len() == to_match.len()
        && compared_strings_zipped.all(|(a, b)| char_matches(a, b, case_sensitive))
    {
        let range = pos..pos + to_match.len();
        matcher.push_token(output, matcher.create_token(source, range.clone()));
        Ok(range)
    } else {
        let found_string_size = compared_strings_zipped.len();

        let mismatched_character = compared_strings_zipped
            .enumerate()
            .find(|(_, (a, b))| !char_matches(a, b, case_sensitive))
            .map(|(index, _)| index);

        let (error_position, description) = match mismatched_character {
            Some(char_index) => (pos + char_index, "found text did not match expected string"),
            None => (
                pos + found_string_size,
                "too short, string began to match but ended early",
            ),
        };

        Err(FluxError::new_matcher(
            description,
            error_position,
            depth,
            matcher.name.clone(),
            Some(source.clone()),
        ))
    }
}

fn apply_char_set(
    matcher: &Matcher,
    source: Arc<Vec<char>>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    to_match: &[char],
    inverted: bool,
) -> TokenResult {
    match source.get(pos) {
        Some(c) if to_match.contains(c) ^ inverted => {
            let range = pos..pos + 1;
            matcher.push_token(output, matcher.create_token(source, range.clone()));
            Ok(range)
        }
        _ => Err(FluxError::new_matcher(
            "expected",
            pos,
            depth,
            matcher.name.clone(),
            Some(source.clone()),
        )),
    }
}

fn apply_char_range(
    matcher: &Matcher,
    source: Arc<Vec<char>>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    range: &RangeInclusive<char>,
    inverted: bool,
) -> TokenResult {
    match source.get(pos) {
        Some(c) if range.contains(c) ^ inverted => {
            let range = pos..pos + 1;
            matcher.push_token(output, matcher.create_token(source, range.clone()));
            Ok(range)
        }
        _ => Err(FluxError::new_matcher(
            "expected",
            pos,
            depth,
            matcher.name.clone(),
            Some(source.clone()),
        )),
    }
}

fn apply_list(
    matcher: &Matcher,
    source: Arc<Vec<char>>,
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
            Ok(child_token) => cursor = child_token.end,

            Err(mut err) => {
                if err.matcher_name.is_none() {
                    err.matcher_name = matcher.name.clone()
                }
                return Err(err);
            }
        }
    }
    let range = pos..cursor;
    matcher.process_children(source, range.clone(), output, output_start);
    Ok(range)
}

fn apply_choice(
    matcher: &Matcher,
    source: Arc<Vec<char>>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    children: &[usize],
    matchers: &[Matcher],
) -> TokenResult {
    let output_start = output.len();
    let mut best_error: Option<FluxError> = None;
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
            Ok(range) => {
                matcher.process_children(source, range.clone(), output, output_start);
                return Ok(range);
            }
            Err(mut err) => {
                if err.matcher_name.is_none() {
                    err.matcher_name = matcher.name.clone();
                }
                best_error = Some(match best_error {
                    Some(e) => e.max(err),
                    _ => err,
                });
            }
        }
    }

    Err(best_error.unwrap_or(FluxError::new_matcher(
        "expected",
        pos,
        depth,
        matcher.name.clone(),
        Some(source),
    )))
}

fn apply_repeating(
    matcher: &Matcher,
    source: Arc<Vec<char>>,
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
    let mut child_error = None;
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
            Ok(child_token) => {
                cursor = child_token.end;
                child_count += 1;
            }
            Err(mut err) => {
                if *range.start() == 0 && err.location == pos {
                    break;
                }
                if err.matcher_name.is_none() {
                    err.matcher_name = matcher.name.clone();
                }
                child_error = Some(err);
                break;
            }
        }
    }

    if child_count < *range.start() {
        let mut error = FluxError::new_matcher(
            "expected",
            cursor,
            depth,
            matcher.name.clone(),
            Some(source),
        );
        if let Some(e) = child_error {
            error = error.max(e);
        }
        Err(error)
    } else {
        let range = pos..cursor;
        matcher.process_children(source, range.clone(), output, output_start);
        Ok(range)
    }
}

fn apply_eof(matcher: &Matcher, source: Arc<Vec<char>>, pos: usize, depth: usize) -> TokenResult {
    if pos == source.len() {
        Ok(pos..pos)
    } else {
        Err(FluxError::new_matcher(
            "expected end of file",
            pos,
            depth,
            matcher.name.clone(),
            Some(source),
        ))
    }
}

fn apply_inverted(
    matcher: &Matcher,
    source: Arc<Vec<char>>,
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
    match matched {
        Ok(_) => Err(FluxError::new_matcher(
            "unexpected",
            pos,
            depth,
            matcher.name.clone(),
            Some(source),
        )),
        Err(_) => {
            let range = pos..pos;
            matcher.push_token(output, matcher.create_token(source, range.clone()));
            Ok(range)
        }
    }
}

fn apply_wrapper(
    source: Arc<Vec<char>>,
    output: &mut TokenOutput,
    pos: usize,
    depth: usize,
    child: usize,
    matchers: &[Matcher],
) -> TokenResult {
    let child = &matchers[child];
    child.apply(source, output, matchers, pos, depth)
}
