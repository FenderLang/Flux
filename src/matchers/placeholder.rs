use super::{Matcher, MatcherMeta};
use crate::{error::Result, tokens::Token};
use std::sync::Arc;

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
    fn apply(&self, _: Arc<Vec<char>>, _: usize, _: usize) -> Result<Token> {
        unreachable!()
    }

    fn min_length(&self) -> usize {
        unreachable!()
    }

    fn is_placeholder(&self) -> bool {
        true
    }
}
