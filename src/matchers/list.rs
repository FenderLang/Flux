use super::{Matcher, MatcherChildren, MatcherMeta, MatcherRef};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct ListMatcher {
    meta: MatcherMeta,
    min_length: RefCell<Option<usize>>,
    children: MatcherChildren,
}

impl ListMatcher {
    pub fn new(meta: MatcherMeta, children: Vec<RefCell<MatcherRef>>) -> ListMatcher {
        ListMatcher {
            meta,
            min_length: RefCell::new(None),
            children,
        }
    }
}

impl Matcher for ListMatcher {
    impl_meta!();
    fn apply(&self, source: Rc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        let mut children: Vec<Token> = Vec::new();
        let mut cursor = pos;
        let mut failures = Vec::new();
        for child in self.children.iter() {
            match child.borrow().apply(source.clone(), cursor, depth + 1) {
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
                    return Err(failures
                        .into_iter()
                        .reduce(FluxError::max)
                        .expect("error always present"));
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

    fn min_length(&self) -> usize {
        if let Some(len) = *self.min_length.borrow() {
            len
        } else {
            let len = self
                .children
                .iter()
                .map(|child| child.borrow().min_length())
                .min()
                .unwrap_or_default();
            *self.min_length.borrow_mut() = Some(len);
            len
        }
    }

    fn children(&self) -> Option<&super::MatcherChildren> {
        Some(&self.children)
    }

    fn is_placeholder(&self) -> bool {
        false
    }
}
