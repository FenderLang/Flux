use super::{Matcher, MatcherMeta};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StringMatcher {
    meta: MatcherMeta,
    to_match: Vec<char>,
    case_sensitive: bool,
}

impl StringMatcher {
    pub fn new(meta: MatcherMeta, to_match: String, case_sensitive: bool) -> Self {
        Self {
            meta,
            to_match: to_match.chars().collect(),
            case_sensitive,
        }
    }

    fn char_matches(&self, first: &char, second: &char) -> bool {
        if self.case_sensitive {
            first == second
        } else {
            first == second || first.eq_ignore_ascii_case(second)
        }
    }
}

impl Matcher for StringMatcher {
    impl_meta!();
    fn apply(&self, source: Arc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        let mut compared_strings_zipped = self.to_match.iter().zip(&source[pos..]);

        if compared_strings_zipped.len() == self.to_match.len()
            && compared_strings_zipped.all(|(a, b)| self.char_matches(a, b))
        {
            Ok(Token {
                matcher_name: self.name().clone(),
                children: Vec::with_capacity(0),
                source,
                range: pos..pos + self.to_match.len(),
                matcher_id: self.id(),
                failure: None,
            })
        } else {
            let found_string_size = compared_strings_zipped.len();

            let mismatched_character = compared_strings_zipped
                .enumerate()
                .find(|(_, (a, b))| !self.char_matches(a, b))
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
                self.name().clone(),
                Some(source.clone()),
            ))
        }
    }
}
