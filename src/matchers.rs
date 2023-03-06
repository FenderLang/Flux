use std::{ops::RangeInclusive, sync::Arc};

use crate::error::{FluxError, Result};
use crate::tokens::Token;

pub type MatcherName = Arc<Option<String>>;

#[derive(Debug, Clone)]
pub struct Matcher {
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
        matchers: &[Matcher],
        pos: usize,
        depth: usize,
    ) -> Result<Token> {
        match &self.matcher_type {
            MatcherType::String(to_match, case_sensitive) => {
                apply_string(self, source, pos, depth, to_match, *case_sensitive)
            }
            MatcherType::CharSet(chars, inverted) => {
                apply_char_set(self, source, pos, depth, chars, *inverted)
            }
            MatcherType::CharRange(range, inverted) => {
                apply_char_range(self, source, pos, depth, range, *inverted)
            }
            MatcherType::List(children) => apply_list(self, source, pos, depth, children, matchers),
            MatcherType::Choice(children) => {
                apply_choice(self, source, pos, depth, children, matchers)
            }
            MatcherType::Repeating(child, range) => {
                apply_repeating(self, source, pos, depth, *child, range, matchers)
            }
            MatcherType::Inverted(child) => {
                apply_inverted(self, source, pos, depth, *child, matchers)
            }
            MatcherType::Wrapper(child) => {
                apply_wrapper(source, pos, depth, *child, matchers)
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
    pos: usize,
    depth: usize,
    to_match: &[char],
    case_sensitive: bool,
) -> Result<Token> {
    let mut compared_strings_zipped = to_match.iter().zip(&source[pos..]);

    if compared_strings_zipped.len() == to_match.len()
       && compared_strings_zipped.all(|(a, b)| char_matches(a, b, case_sensitive))
    {
        Ok(Token {
            matcher_name: matcher.name.clone(),
            children: Vec::with_capacity(0),
            source,
            range: pos..pos + to_match.len(),
            matcher_id: matcher.id,
            failure: None,
        })
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
    pos: usize,
    depth: usize,
    to_match: &[char],
    inverted: bool,
) -> Result<Token> {
    match source.get(pos) {
        Some(c) if to_match.contains(c) ^ inverted => Ok(Token {
            children: Vec::with_capacity(0),
            matcher_name: matcher.name.clone(),
            range: pos..pos + 1,
            source,
            matcher_id: matcher.id,
            failure: None,
        }),
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
    pos: usize,
    depth: usize,
    range: &RangeInclusive<char>,
    inverted: bool,
) -> Result<Token> {
    match source.get(pos) {
        Some(c) if range.contains(c) ^ inverted => Ok(Token {
            matcher_name: matcher.name.clone(),
            children: Vec::with_capacity(0),
            source,
            range: pos..pos + 1,
            matcher_id: matcher.id,
            failure: None,
        }),
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
    pos: usize,
    depth: usize,
    children: &[usize],
    matchers: &[Matcher],
) -> Result<Token> {
    let mut child_matchers: Vec<Token> = Vec::with_capacity(children.len());
    let mut cursor = pos;
    let mut failures = Vec::new();
    for child in children {
        let child = &matchers[*child];
        match child.apply(source.clone(), matchers, cursor, next_depth(matcher, depth)) {
            Ok(mut child_token) => {
                cursor = child_token.range.end;
                let failure = std::mem::replace(&mut child_token.failure, None);
                failures.extend(failure);
                child_token.failure = None;
                child_matchers.push(child_token);
            }
            Err(mut err) => {
                if err.matcher_name.is_none() {
                    err.matcher_name = matcher.name.clone()
                }
                failures.push(err);
                let reduced = failures
                    .into_iter()
                    .reduce(FluxError::max)
                    .expect("error always present");
                return Err(reduced);
            }
        }
    }

    Ok(Token {
        range: (pos..child_matchers.iter().last().unwrap().range.end),
        children: child_matchers,
        matcher_name: matcher.name.clone(),
        source,
        matcher_id: matcher.id,
        failure: failures.into_iter().reduce(FluxError::max),
    })
}

fn apply_choice(
    matcher: &Matcher,
    source: Arc<Vec<char>>,
    pos: usize,
    depth: usize,
    children: &[usize],
    matchers: &[Matcher],
) -> Result<Token> {
    let mut errors: Vec<FluxError> = vec![];
    for child in children {
        let child = &matchers[*child];
        let matched = child.apply(source.clone(), matchers, pos, next_depth(matcher, depth));
        match matched {
            Ok(mut token) => {
                return Ok(Token {
                    matcher_name: matcher.name.clone(),
                    range: token.range.clone(),
                    failure: {
                        let failure = std::mem::replace(&mut token.failure, None);
                        errors.extend(failure);
                        errors.into_iter().reduce(FluxError::max)
                    },
                    children: vec![token],
                    source,
                    matcher_id: matcher.id,
                });
            }
            Err(mut err) => {
                if err.matcher_name.is_none() {
                    err.matcher_name = matcher.name.clone();
                }
                errors.push(err)
            }
        }
    }
    errors.push(FluxError::new_matcher(
        "expected",
        pos,
        depth,
        matcher.name.clone(),
        Some(source),
    ));
    Err(errors
        .into_iter()
        .reduce(FluxError::max)
        .expect("error always present"))
}

fn apply_repeating(
    matcher: &Matcher,
    source: Arc<Vec<char>>,
    pos: usize,
    depth: usize,
    child: usize,
    range: &RangeInclusive<usize>,
    matchers: &[Matcher],
) -> Result<Token> {
    let mut children: Vec<Token> = Vec::with_capacity(*range.start());

    let child = &matchers[child];
    let mut cursor = pos;
    let mut child_error = None;
    while children.len() < *range.end() {
        match child.apply(source.clone(), matchers, cursor, next_depth(matcher, depth)) {
            Ok(mut child_token) => {
                cursor = child_token.range.end;
                let err = std::mem::replace(&mut child_token.failure, None);
                child_error = err.or(child_error);
                children.push(child_token);
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

    if children.len() < *range.start() {
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
        Ok(Token {
            range: (pos..cursor),
            children,
            matcher_name: matcher.name.clone(),
            source,
            matcher_id: matcher.id,
            failure: child_error,
        })
    }
}

fn apply_eof(matcher: &Matcher, source: Arc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
    if pos == source.len() {
        Ok(Token {
            matcher_name: matcher.name.clone(),
            children: Vec::with_capacity(0),
            source,
            range: (pos..pos),
            matcher_id: matcher.id,
            failure: None,
        })
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
    pos: usize,
    depth: usize,
    child: usize,
    matchers: &[Matcher],
) -> Result<Token> {
    let child = &matchers[child];
    match child.apply(source.clone(), matchers, pos, next_depth(matcher, depth)) {
        Ok(_) => Err(FluxError::new_matcher(
            "unexpected",
            pos,
            depth,
            matcher.name.clone(),
            Some(source),
        )),
        Err(err) => Ok(Token {
            children: Vec::with_capacity(0),
            matcher_name: matcher.name.clone(),
            source,
            range: pos..pos,
            matcher_id: matcher.id,
            failure: Some(err),
        }),
    }
}

fn apply_wrapper(
    source: Arc<Vec<char>>,
    pos: usize,
    depth: usize,
    child: usize,
    matchers: &[Matcher],
) -> Result<Token> {
    let child = &matchers[child];
    child.apply(source, matchers, pos, depth)
}
