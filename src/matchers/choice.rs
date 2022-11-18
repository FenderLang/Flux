use super::{Matcher, MatcherName, MatcherRef};
use crate::error::{FluxError, Result};
use crate::tokens::Token;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct ChoiceMatcher {
    name: MatcherName,
    min_length: RefCell<Option<usize>>,
    children: Vec<RefCell<MatcherRef>>,
}

impl ChoiceMatcher {
    pub fn new(children: Vec<RefCell<MatcherRef>>) -> ChoiceMatcher {
        ChoiceMatcher {
            name: Rc::new(RefCell::new(None)),
            min_length: RefCell::new(None),
            children,
        }
    }
}

impl Matcher for ChoiceMatcher {
    fn apply<'a>(&self, source: &'a Vec<char>, pos: usize) -> Result<Token<'a>> {
        for child in &self.children {
            if let Ok(token) = child.borrow().apply(source, pos) {
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

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, new_name: String) {
        self.name.replace(Some(new_name));
    }
    fn children(&self) -> Option<&Vec<RefCell<MatcherRef>>> {
        Some(&self.children)
    }
}
