use super::{Matcher, MatcherName, MatcherRef, MatcherMeta};
use crate::error::{FluxError, Result};
use crate::tokens::Token;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct ChoiceMatcher {
    meta: MatcherMeta,
    min_length: RefCell<Option<usize>>,
    children: Vec<RefCell<MatcherRef>>,
}

impl ChoiceMatcher {
    pub fn new(meta: MatcherMeta, children: Vec<RefCell<MatcherRef>>) -> ChoiceMatcher {
        ChoiceMatcher {
            meta,
            min_length: RefCell::new(None),
            children,
        }
    }
}

impl Matcher for ChoiceMatcher {
    with_meta!();
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        for child in &self.children {
            if let Ok(token) = child.borrow().apply(source.clone(), pos) {
                return Ok(Token {
                    matcher_name: self.get_name().clone(),
                    range: token.range.clone(),
                    children: vec![token],
                    source,
                    matcher_id: self.id(),
                });
            }
        }

        Err(FluxError::new_matcher(
            "in ChoiceMatcher all children failed",
            pos,
            self.get_name().clone(),
        ))
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

    fn children(&self) -> Option<&Vec<RefCell<MatcherRef>>> {
        Some(&self.children)
    }

    fn meta(&self) -> &MatcherMeta {
        &self.meta
    }
}
