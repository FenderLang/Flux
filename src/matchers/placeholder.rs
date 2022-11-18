use std::{cell::RefCell, rc::Rc};

use super::{Matcher, MatcherName};

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
    fn apply<'a>(
        &self,
        _: &'a Vec<char>,
        _: usize,
    ) -> crate::error::Result<crate::tokens::Token<'a>> {
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
