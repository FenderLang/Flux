use super::{Matcher, MatcherChildren, MatcherMeta, MatcherRef};
use crate::error::{FluxError, Result};
use crate::tokens::Token;
use std::sync::Arc;
use std::sync::RwLockWriteGuard;

#[derive(Debug, Clone)]
pub struct ChoiceMatcher {
    meta: MatcherMeta,
    children: MatcherChildren,
}

impl ChoiceMatcher {
    pub fn new(meta: MatcherMeta, children: Vec<MatcherRef>) -> ChoiceMatcher {
        ChoiceMatcher {
            meta,
            children: MatcherChildren::new(children),
        }
    }
}

impl Matcher for ChoiceMatcher {
    impl_meta!();
    fn apply(&self, source: Arc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        let mut errors: Vec<FluxError> = vec![];
        for child in &*self.children.get() {
            let matched = child.apply(source.clone(), pos, self.next_depth(depth));
            match matched {
                Ok(mut token) => {
                    return Ok(Token {
                        matcher_name: self.name().clone(),
                        range: token.range.clone(),
                        failure: {
                            let failure = std::mem::replace(&mut token.failure, None);
                            errors.extend(failure);
                            errors.into_iter().reduce(FluxError::max)
                        },
                        children: vec![token],
                        source,
                        matcher_id: self.id(),
                    });
                }
                Err(mut err) => {
                    if err.matcher_name.is_none() {
                        err.matcher_name = self.name().clone();
                    }
                    errors.push(err)
                }
            }
        }
        errors.push(FluxError::new_matcher(
            "expected",
            pos,
            depth,
            self.name().clone(),
            Some(source),
        ));
        Err(errors
            .into_iter()
            .reduce(FluxError::max)
            .expect("error always present"))
    }

    fn children(& self) -> Option<RwLockWriteGuard<Vec<MatcherRef>>> {
        Some(self.children.get_mut())
    }
}
