use super::{Matcher, MatcherMeta};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{collections::HashSet, sync::Arc};

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
    fn apply(&self, source: Arc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        match source.get(pos) {
            Some(c) if self.check_char(c) => Ok(Token {
                children: Vec::with_capacity(0),
                matcher_name: self.name().clone(),
                range: pos..pos + 1,
                source,
                matcher_id: self.id(),
                failure: None,
            }),
            _ => Err(FluxError::new_matcher(
                "expected",
                pos,
                depth,
                self.name().clone(),
                Some(source.clone()),
            )),
        }
    }
}
