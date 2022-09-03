use std::{borrow::Borrow, cell::RefCell};

use crate::error::FluxError;

use super::{Matcher, MatcherRef};

pub struct ChoiceMatcher {
    name: String,
    min_length: usize,
    children: Vec<RefCell<MatcherRef>>,
}

impl ChoiceMatcher {
    pub fn new<S: ToString>(name: S, children: Vec<RefCell<MatcherRef>>) -> ChoiceMatcher {
        ChoiceMatcher {
            name: name.to_string(),
            min_length: children
                .iter()
                .map(|child| child.borrow().min_length())
                .min()
                .unwrap_or_default(),
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

        for child in &self.children{
            if let Ok(token) =  child.borrow().apply(source.clone(), pos){
                return Ok(token);
            }
        }

        Err(FluxError::new_matcher("in ChoiceMatcher all children failed", pos, self.name.clone()))
    }

    fn min_length(&self) -> usize {
        self.min_length
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn children(&self) -> Option<&Vec<RefCell<MatcherRef>>> {
        Some(&self.children)
    }
}
