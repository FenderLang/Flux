use super::{Matcher, MatcherMeta};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct EofMatcher {
    meta: MatcherMeta,
}

impl EofMatcher {
    pub fn new(meta: MatcherMeta) -> EofMatcher {
        EofMatcher { meta }
    }
}

impl Matcher for EofMatcher {
    impl_meta!();
    fn apply(&self, source: Arc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        if pos == source.len() {
            Ok(Token {
                matcher_name: self.name().clone(),
                children: Vec::with_capacity(0),
                source,
                range: (pos..pos),
                matcher_id: self.id(),
                failure: None,
            })
        } else {
            Err(FluxError::new_matcher(
                "expected end of file",
                pos,
                depth,
                self.name().clone(),
                Some(source),
            ))
        }
    }
}
