use super::{Matcher, MatcherMeta, MatcherRef};
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
    impl_meta!();
    fn apply(&self, source: Rc<Vec<char>>, pos: usize, depth: usize) -> Result<Token> {
        let mut errors: Vec<FluxError> = vec![];
        for child in &self.children {
            let matched = child
                .borrow()
                .apply(source.clone(), pos, self.next_depth(depth));
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
        ));
        Err(errors
            .into_iter()
            .reduce(FluxError::max)
            .expect("error always present"))
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
}
