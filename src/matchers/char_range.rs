use super::{Matcher, MatcherMeta};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{rc::Rc, vec};

#[derive(Clone, Debug)]
pub struct CharRangeMatcher {
    meta: MatcherMeta,
    min: char,
    max: char,
    inverted: bool,
}

impl CharRangeMatcher {
    pub fn new(meta: MatcherMeta, min: char, max: char, inverted: bool) -> CharRangeMatcher {
        CharRangeMatcher {
            max,
            min,
            inverted,
            meta,
        }
    }

    pub fn check_char(&self, check_char: char) -> bool {
        (check_char >= self.min && check_char <= self.max) ^ self.inverted
    }
}

impl Matcher for CharRangeMatcher {
    impl_meta!();
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        match source.get(pos) {
            Some(c) if self.check_char(*c) => Ok(Token {
                matcher_name: self.name().clone(),
                children: vec![],
                source,
                range: pos..pos + 1,
                matcher_id: self.id(),
                failure: None,
            }),
            _ => Err(FluxError::new_matcher(
                "expected",
                pos,
                self.priority(),
                self.name().clone(),
            )),
        }
    }

    fn min_length(&self) -> usize {
        1
    }
}
