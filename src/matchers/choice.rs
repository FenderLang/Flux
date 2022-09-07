use super::{Matcher, MatcherRef};
use crate::error::FluxError;
use std::{cell::RefCell, rc::Rc};

pub struct ChoiceMatcher {
    name: Option<Rc<String>>,
    min_length: RefCell<Option<usize>>,
    children: Vec<RefCell<MatcherRef>>,
}

impl ChoiceMatcher {
    pub fn new<S: ToString>(name: Option<S>, children: Vec<RefCell<MatcherRef>>) -> ChoiceMatcher {
        ChoiceMatcher {
            name: name.map(|name| Rc::new(name.to_string())),
            min_length: RefCell::new(None),
            children,
        }
    }
}

impl Matcher for ChoiceMatcher {
    fn apply(
        &self,
        source: std::rc::Rc<Vec<char>>,
        pos: usize,
    ) -> crate::error::Result<crate::tokens::Token> {
        for child in &self.children {
            if let Ok(token) = child.borrow().apply(source.clone(), pos) {
                return Ok(token);
            }
        }

        Err(FluxError::new_matcher(
            "in ChoiceMatcher all children failed",
            pos,
            self.name.clone(),
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

    fn name(&self) -> Option<&str> {
        if let Some(name) = &self.name {
            Some(name.as_str())
        } else {
            None
        }
    }

    fn children(&self) -> Option<&Vec<RefCell<MatcherRef>>> {
        Some(&self.children)
    }
}
