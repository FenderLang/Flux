use super::{Matcher, MatcherName};
use crate::{error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct PlaceholderMatcher {
    name: MatcherName,
    id: RefCell<usize>,
}

impl PlaceholderMatcher {
    pub fn new(name: String) -> PlaceholderMatcher {
        PlaceholderMatcher {
            name: Rc::new(RefCell::new(Some(name))),
            id: RefCell::new(0),
        }
    }
}

impl Matcher for PlaceholderMatcher {
    fn apply(&self, _: Rc<Vec<char>>, _: usize) -> Result<Token> {
        unreachable!()
    }

    fn min_length(&self) -> usize {
        unreachable!()
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, _: String) {
        unreachable!()
    }

    fn is_placeholder(&self) -> bool {
        true
    }

    fn id(&self) -> &RefCell<usize> {
        &self.id
    }
}
