use super::{Matcher, MatcherName, MatcherRef};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc, vec};

#[derive(Debug)]
pub struct InvertedMatcher {
    name: MatcherName,
    child: MatcherRef,
}

impl InvertedMatcher {
    pub fn new(child: MatcherRef) -> Self {
        Self {
            name: Rc::new(RefCell::new(None)),
            child,
        }
    }
}

impl Matcher for InvertedMatcher {
    fn apply<'a>(&self, source: &'a Vec<char>, pos: usize) -> Result<Token<'a>> {
        match self.child.apply(source, pos) {
            Ok(_) => Err(FluxError::new_matcher(
                "unexpected",
                pos,
                self.name.clone(),
            )),
            Err(_) => Ok(Token {
                children: vec![],
                matcher_name: self.name.clone(),
                source,
                range: pos..pos,
            }),
        }
    }

    fn min_length(&self) -> usize {
        0
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, new_name: String) {
        self.name.replace(Some(new_name));
    }
}
