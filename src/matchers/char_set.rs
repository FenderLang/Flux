use super::{Matcher, MatcherMeta};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{collections::HashSet, rc::Rc};

#[derive(Clone, Debug)]
pub struct CharSetMatcher {
    meta: MatcherMeta,
    matching_set: HashSet<char>,
    inverted: bool,
}

impl CharSetMatcher {
    pub fn new(meta: MatcherMeta, matching_set: HashSet<char>, inverted: bool) -> Self {
        Self {
            meta,
            matching_set,
            inverted,
        }
    }

    pub fn check_char(&self, check_char: &char) -> bool {
        self.matching_set.contains(check_char) ^ self.inverted
    }
}

impl Matcher for CharSetMatcher {
    impl_meta!();
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        match source.get(pos) {
            Some(c) if self.check_char(c) => Ok(Token {
                children: vec![],
                matcher_name: self.get_name().clone(),
                range: pos..pos + 1,
                source,
                matcher_id: self.id(),
            }),
            _ => Err(FluxError::new_matcher("expected", pos, self.get_name().clone())),
        }
    }

    fn min_length(&self) -> usize {
        1
    }
}
