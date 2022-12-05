use super::{Matcher, MatcherName};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct EofMatcher {
    name: MatcherName,
}

impl EofMatcher {
    pub fn new() -> EofMatcher {
        EofMatcher {
            name: Rc::new(RefCell::new(None)),
        }
    }
}

impl Matcher for EofMatcher {
    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }
}