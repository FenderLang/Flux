use super::{Matcher, MatcherName};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc, vec};

#[derive(Clone, Debug)]
pub struct CharRangeMatcher {
    name: MatcherName,
    min: char,
    max: char,
    inverted: bool,
}

impl CharRangeMatcher {
    pub fn new(min: char, max: char, inverted: bool) -> CharRangeMatcher {
        CharRangeMatcher {
            name: Rc::new(RefCell::new(None)),
            max,
            min,
            inverted,
        }
    }

    pub fn check_char(&self, check_char: char) -> bool {
        (check_char >= self.min && check_char <= self.max) ^ self.inverted
    }
}

impl Matcher for CharRangeMatcher {
    fn apply<'a>(&self, source: &'a [char], pos: usize) -> Result<Token<'a>> {
        match source.get(pos) {
            Some(c) if self.check_char(*c) => Ok(Token {
                matcher_name: self.name.clone(),
                children: vec![],
                source,
                range: pos..pos + 1,
            }),
            _ => Err(FluxError::new_matcher("expected", pos, self.name.clone())),
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
