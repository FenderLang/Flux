use super::{Matcher, MatcherName};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

#[derive(Clone, Debug)]
pub struct CharSetMatcher {
    name: MatcherName,
    matching_set: HashSet<char>,
    inverted: bool,
}

impl CharSetMatcher {
    pub fn new(matching_set: HashSet<char>, inverted: bool) -> Self {
        Self {
            name: Rc::new(RefCell::new(None)),
            matching_set,
            inverted,
        }
    }

    pub fn check_char(&self, check_char: &char) -> bool {
        self.matching_set.contains(check_char) ^ self.inverted
    }
}

impl Matcher for CharSetMatcher {
    fn apply<'a>(&self, source: &'a Vec<char>, pos: usize) -> Result<Token<'a>> {
        match source.get(pos) {
            Some(c) if self.check_char(c) => {
                Ok(Token {
                    children: vec![],
                    matcher_name: self.name.clone(),
                    range: pos..pos + 1,
                    source,
                })
            }
            _ => Err(FluxError::new_matcher(
                "expected",
                pos,
                self.name.clone(),
            )),
        }
    }

    fn min_length(&self) -> usize {
        1
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, new_name: String) {
        self.name.replace(Some(new_name));
    }
}
