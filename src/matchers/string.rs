use super::{Matcher, MatcherMeta};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::rc::Rc;

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
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        let mut zip = self
            .to_match
            .iter()
            .zip(&source[pos..])
            .take(self.to_match.len());

        if zip.len() == self.to_match.len() && zip.all(|(a, b)| self.char_matches(a, b)) {
            Ok(Token {
                matcher_name: self.get_name().clone(),
                children: vec![],
                source,
                range: pos..pos + self.to_match.len(),
                matcher_id: self.id(),
            })
        } else {
            Err(FluxError::new_matcher("expected", pos, self.get_name().clone()))
        }
    }

    fn min_length(&self) -> usize {
        self.to_match.len()
    }
}
