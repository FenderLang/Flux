use super::{Matcher, MatcherName};
use crate::{error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct PlaceholderMatcher {
    name: MatcherName,
}

impl PlaceholderMatcher {
    pub fn new(name: String) -> PlaceholderMatcher {
        PlaceholderMatcher {
            name: Rc::new(RefCell::new(Some(name))),
        }
    }
}

impl Matcher for PlaceholderMatcher {
    fn apply<'a>(&self, _: &'a [char], _: usize) -> Result<Token<'a>> {
        unreachable!()
    }

    fn min_length(&self) -> usize {
        unreachable!()
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, _: String) {
        unreachable!()
    }

    fn is_placeholder(&self) -> bool {
        true
    }
}
