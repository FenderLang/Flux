use super::{Matcher, MatcherName};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

#[derive(Clone, Debug)]
pub struct CharSetMatcher {
    name: MatcherName,
    matching_set: HashSet<char>,
    inverted: bool,
    id: RefCell<usize>,
}

impl CharSetMatcher {
    pub fn new(matching_set: HashSet<char>, inverted: bool) -> Self {
        Self {
            name: Rc::new(RefCell::new(None)),
            matching_set,
            inverted,
            id: RefCell::new(0),
        }
    }

    pub fn check_char(&self, check_char: &char) -> bool {
        self.matching_set.contains(check_char) ^ self.inverted
    }
}

impl Matcher for CharSetMatcher {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        match source.get(pos) {
            Some(c) if self.check_char(c) => Ok(Token {
                children: vec![],
                matcher_name: self.name.clone(),
                range: pos..pos + 1,
                source,
                matcher_id: *self.id.borrow()
            }),
            _ => Err(FluxError::new_matcher("expected", pos, self.name.clone())),
        }
    }

    fn min_length(&self) -> usize {
        1
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, new_name: String) {
        self.name.replace(Some(new_name));
    }

    fn id(&self) -> &RefCell<usize> {
        &self.id
    }
}
