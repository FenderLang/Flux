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
    fn apply<'a>(&self, source: &'a [char], pos: usize) -> Result<Token<'a>> {
        if pos == source.len() {
            Ok(Token {
                matcher_name: self.name.clone(),
                children: Vec::new(),
                source,
                range: (pos..pos),
                matcher_id: *self.id.borrow(),
            })
        } else {
            Err(FluxError::new_matcher("expected", 0, self.name.clone()))
        }
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

    fn id(&self) -> &RefCell<usize> {
        &self.id
    }
}