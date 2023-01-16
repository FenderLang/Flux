use super::{Matcher, MatcherMeta};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{rc::Rc};

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
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        if pos == source.len() {
            Ok(Token {
                matcher_name: self.get_name().clone(),
                children: Vec::new(),
                source,
                range: (pos..pos),
                matcher_id: self.id(),
            })
        } else {
            Err(FluxError::new_matcher(
                "expected end of file",
                pos,
                self.get_name().clone(),
            ))
        }
    }

    fn min_length(&self) -> usize {
        0
    }
}
