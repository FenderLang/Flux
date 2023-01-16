use super::{Matcher, MatcherChildren, MatcherMeta, MatcherRef};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc, vec};

#[derive(Debug, Clone)]
pub struct InvertedMatcher {
    meta: MatcherMeta,
    child: MatcherChildren,
}

impl InvertedMatcher {
    pub fn new(meta: MatcherMeta, child: MatcherRef) -> Self {
        Self {
            meta,
            child: vec![RefCell::new(child)],
        }
    }
}

impl Matcher for InvertedMatcher {
    impl_meta!();
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        match self.child[0].borrow().apply(source.clone(), pos) {
            Ok(_) => Err(FluxError::new_matcher(
                "unexpected",
                pos,
                self.name().clone(),
            )),
            Err(_) => Ok(Token {
                children: vec![],
                matcher_name: self.name().clone(),
                source,
                range: pos..pos,
                matcher_id: self.id(),
            }),
        }
    }

    fn min_length(&self) -> usize {
        0
    }

    fn children(&self) -> Option<&super::MatcherChildren> {
        Some(&self.child)
    }
}
