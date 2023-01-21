use super::{Matcher, MatcherMeta};
use crate::{error::Result, tokens::Token};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct PlaceholderMatcher {
    meta: MatcherMeta,
}

impl PlaceholderMatcher {
    pub fn new(meta: MatcherMeta) -> PlaceholderMatcher {
        PlaceholderMatcher { meta }
    }
}

impl Matcher for PlaceholderMatcher {
    impl_meta!();
    fn apply(&self, _: Rc<Vec<char>>, _: usize, _: usize) -> Result<Token> {
        unreachable!()
    }

    fn min_length(&self) -> usize {
        unreachable!()
    }

    fn is_placeholder(&self) -> bool {
        true
    }
}
