use super::{Matcher, MatcherName};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct EofMatcher {
    name: MatcherName,
    id: RefCell<usize>,
}

impl EofMatcher {
    pub fn new() -> EofMatcher {
        EofMatcher {
            name: Rc::new(RefCell::new(None)),
            id: RefCell::new(0),
        }
    }
}

impl Matcher for EofMatcher {
    fn apply<'a>(&self, _: &'a [char], _: usize) -> Result<Token<'a>> {
        unreachable!()
    }

    fn min_length(&self) -> usize {
        0
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, name: String) {
        self.name.replace(Some(name));
    }

    fn is_placeholder(&self) -> bool {
        true
    }

    fn id(&self) -> &RefCell<usize> {
        &self.id
    }
}