use super::{Matcher, MatcherChildren, MatcherMeta, MatcherRef};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::sync::{Arc, RwLockWriteGuard};

#[derive(Debug, Clone)]
pub struct ListMatcher {
    meta: MatcherMeta,
    children: MatcherChildren,
}

impl ListMatcher {
    pub fn new(meta: MatcherMeta, children: Vec<MatcherRef>) -> ListMatcher {
        ListMatcher {
            meta,
            children: MatcherChildren::new(children),
        }
    }
}

impl Matcher for ListMatcher {
    impl_meta!();
    fn apply(&self, source: Arc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        let mut children: Vec<Token> = Vec::with_capacity(self.children.get().len());
        let mut cursor = pos;
        let mut failures = Vec::new();
        for child in &*self.children.get() {
            match child.apply(source.clone(), cursor, self.next_depth(depth)) {
                Ok(mut child_token) => {
                    cursor = child_token.range.end;
                    let failure = std::mem::replace(&mut child_token.failure, None);
                    failures.extend(failure);
                    child_token.failure = None;
                    children.push(child_token);
                }
                Err(mut err) => {
                    if err.matcher_name.is_none() {
                        err.matcher_name = self.name().clone()
                    }
                    failures.push(err);
                    let reduced = failures
                        .into_iter()
                        .reduce(FluxError::max)
                        .expect("error always present");
                    return Err(reduced);
                }
            }
        }

        Ok(Token {
            range: (pos..children.iter().last().unwrap().range.end),
            children,
            matcher_name: self.name().clone(),
            source,
            matcher_id: self.id(),
            failure: failures.into_iter().reduce(FluxError::max),
        })
    }

    fn children(&self) -> Option<RwLockWriteGuard<Vec<MatcherRef>>> {
        Some(self.children.get_mut())
    }

    fn is_placeholder(&self) -> bool {
        false
    }
}
