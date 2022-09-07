use super::{Matcher, MatcherName, MatcherRef};
use crate::{error::FluxError, tokens::Token};
use std::{cell::RefCell, rc::Rc, vec};

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
    fn apply(
        &self,
        source: Rc<Vec<char>>,
        pos: usize,
    ) -> crate::error::Result<crate::tokens::Token> {
        match self.child.apply(source.clone(), pos) {
            Ok(_) => Err(FluxError::new_matcher(
                "Inverted matcher",
                pos,
                self.name.clone(),
            )),
            Err(_) => Ok(Token {
                children: vec![],
                matcher_name: self.name.clone(),
                source,
                range: 0..0,
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
