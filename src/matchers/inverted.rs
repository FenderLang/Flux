use super::{Matcher, MatcherChildren, MatcherName, MatcherRef};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc, vec};

#[derive(Debug)]
pub struct InvertedMatcher {
    name: MatcherName,
    child: MatcherChildren,
    id: RefCell<usize>,
}

impl InvertedMatcher {
    pub fn new(child: MatcherRef) -> Self {
        Self {
            name: Rc::new(RefCell::new(None)),
            child: vec![RefCell::new(child)],
            id: RefCell::new(0),
        }
    }
}

impl Matcher for InvertedMatcher {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        match self.child[0].borrow().apply(source.clone(), pos) {
            Ok(_) => Err(FluxError::new_matcher("unexpected", pos, self.name.clone())),
            Err(_) => Ok(Token {
                children: vec![],
                matcher_name: self.name.clone(),
                source,
                range: pos..pos,
                matcher_id: *self.id.borrow(),
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

    fn children(&self) -> Option<&super::MatcherChildren> {
        Some(&self.child)
    }

    fn id(&self) -> &RefCell<usize> {
        &self.id
    }
}
