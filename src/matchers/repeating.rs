use super::{Matcher, MatcherChildren, MatcherMeta, MatcherRef};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::sync::{Arc, RwLockWriteGuard};

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
            child: MatcherChildren::new(vec![child]),
        }
    }
}

impl Matcher for RepeatingMatcher {
    impl_meta!();
    fn apply(&self, source: Arc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        let mut children: Vec<Token> = Vec::new();

        let child = &self.child.get()[0];
        let mut cursor = pos;
        let mut child_error = None;
        while children.len() < self.max {
            match child.apply(source.clone(), cursor, self.next_depth(depth)) {
                Ok(mut child_token) => {
                    cursor = child_token.range.end;
                    let err = std::mem::replace(&mut child_token.failure, None);
                    child_error = err.or(child_error);
                    children.push(child_token);
                }
                Err(mut err) => {
                    if self.min == 0 && err.location == pos {
                        break;
                    }
                    if err.matcher_name.is_none() {
                        err.matcher_name = self.name().clone();
                    }
                    child_error = Some(err);
                    break;
                }
            }
        }

        if children.len() < self.min {
            let mut error = FluxError::new_matcher("expected", cursor, depth, self.name().clone(),Some(source));
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
                failure: child_error,
            })
        }
    }

    fn children<'a>(&'a self) -> Option<RwLockWriteGuard<'a, Vec<MatcherRef>>> {
        Some(self.child.get_mut())
    }
}
