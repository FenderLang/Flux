use super::{Matcher, MatcherChildren, MatcherRef, MatcherMeta};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct RepeatingMatcher {
    meta: MatcherMeta,
    min: usize,
    max: usize,
    child: MatcherChildren,
}

impl RepeatingMatcher {
    pub fn new(meta: MatcherMeta, min: usize, max: usize, child: MatcherRef) -> Self {
        Self {
            meta,
            min,
            max,
            child: vec![RefCell::new(child)],
        }
    }
}

impl Matcher for RepeatingMatcher {
    impl_meta!();
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        let mut children: Vec<Token> = Vec::new();

        let child = self.child[0].borrow();
        let mut cursor = pos;
        while children.len() < self.max {
            match child.apply(source.clone(), cursor) {
                Ok(child_token) => {
                    cursor = child_token.range.end;
                    children.push(child_token);
                }
                Err(_) => break,
            }
        }

        if children.len() < self.min {
            Err(FluxError::new_matcher("expected", pos, self.get_name().clone()))
        } else {
            Ok(Token {
                range: (pos..cursor),
                children,
                matcher_name: self.get_name().clone(),
                source,
                matcher_id: self.id(),
            })
        }
    }

    fn min_length(&self) -> usize {
        self.child[0].borrow().min_length() * self.min
    }

    fn children(&self) -> Option<&MatcherChildren> {
        Some(&self.child)
    }
}
