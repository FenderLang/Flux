use super::{Matcher, MatcherChildren, MatcherMeta, MatcherRef};
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
    fn apply(&self, source: Rc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        let mut children: Vec<Token> = Vec::new();

        let child = self.child[0].borrow();
        let mut cursor = pos;
        let mut child_error = None;
        while children.len() < self.max {
            match child.apply(source.clone(), cursor, depth + 1) {
                Ok(child_token) => {
                    cursor = child_token.range.end;
                    children.push(child_token);
                }
                Err(err) => child_error = Some(err),
            }
        }

        if children.len() < self.min {
            let mut error = FluxError::new_matcher("expected", pos, depth, self.name().clone());
            if let Some(e) = child_error {
                error = error.max(e);
            }
            Err(error)
        } else {
            Ok(Token {
                range: (pos..cursor),
                children,
                matcher_name: self.name().clone(),
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
