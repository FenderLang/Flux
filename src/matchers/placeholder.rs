use std::{rc::Rc, cell::RefCell};

use super::{Matcher, MatcherName};


pub struct PlaceholderMatcher {

    name: MatcherName,

}

impl PlaceholderMatcher {
    pub fn new(name: String) -> PlaceholderMatcher {
        
        PlaceholderMatcher {name: Rc::new(RefCell::new(Some(name)))}
    }
}

impl Matcher for PlaceholderMatcher {
    fn apply(&self, _: std::rc::Rc<Vec<char>>, _: usize) -> crate::error::Result<crate::tokens::Token> {
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