use std::rc::Rc;

use super::{Matcher, MatcherRef};

pub struct InvertedMatcher {
    name: Option<Rc<String>>,
    child: MatcherRef,
}

impl InvertedMatcher {
    pub fn new(child: MatcherRef) -> Self {
        Self { name: None, child }
    }
}

impl Matcher for InvertedMatcher {
    fn apply(
        &self,
        source: Rc<Vec<char>>,
        pos: usize,
    ) -> crate::error::Result<crate::tokens::Token> {
        todo!()
    }

    fn min_length(&self) -> usize {
        todo!()
    }

    fn get_name(&self) -> Option<&str> {
        todo!()
    }

    fn set_name(&mut self, new_name: String) {
        todo!()
    }
}
