use super::{Matcher, MatcherChildren, MatcherMeta, MatcherRef};
use crate::{error::FluxError, error::Result, tokens::Token, bnf, lexer::Lexer};
use std::{sync::Arc, vec};

#[derive(Debug, Clone)]
pub struct InvertedMatcher {
    meta: MatcherMeta,
    child: MatcherChildren,
}

impl InvertedMatcher {
    pub fn new(meta: MatcherMeta, child: MatcherRef) -> Self {
        Self {
            meta,
            child: MatcherChildren::new(vec![child]),
        }
    }
}

impl Matcher for InvertedMatcher {
    impl_meta!();
    fn apply(&self, source: Arc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        match self.child[0].apply(source.clone(), pos, self.next_depth(depth)) {
            Ok(_) => Err(FluxError::new_matcher(
                "unexpected",
                pos,
                depth,
                self.name().clone(),
                Some(source)
            )),
            Err(err) => Ok(Token {
                children: vec![],
                matcher_name: self.name().clone(),
                source,
                range: pos..pos,
                matcher_id: self.id(),
                failure: Some(err),
            }),
        }
    }

    fn children(&self) -> Option<&mut Vec<MatcherRef>> {
        Some(self.child.get_mut())
    }
}
