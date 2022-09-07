use super::{Matcher, MatcherRef};
use crate::{error::FluxError, tokens::Token};
use std::{rc::Rc, vec};

pub struct InvertedMatcher {
    name: Option<Rc<String>>,
    child: MatcherRef,
}

impl InvertedMatcher {
    pub fn new(child: MatcherRef) -> Self {
        Self { name: None, child }
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

    fn get_name(&self) -> Option<&str> {
        match &self.name {
            Some(name) => Some(name.as_str()),
            None => None,
        }
    }

    fn set_name(&mut self, new_name: String) {
        self.name = Some(Rc::new(new_name));
    }
}
