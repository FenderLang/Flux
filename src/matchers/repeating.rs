use super::{Matcher, MatcherChildren, MatcherName, MatcherRef};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct RepeatingMatcher {
    name: MatcherName,
    min: usize,
    max: usize,
    child: MatcherRef,
}

impl RepeatingMatcher {
    pub fn new(min: usize, max: usize, child: MatcherRef) -> Self {
        Self {
            name: Rc::new(RefCell::new(None)),
            min,
            max,
            child,
        }
    }
}

impl Matcher for RepeatingMatcher {
    fn apply<'a>(&self, source: &'a Vec<char>, pos: usize) -> Result<Token<'a>> {
        let mut children: Vec<Token> = Vec::new();

        while children.len() < self.max {
            match self.child.apply(source, pos) {
                Ok(child_token) => children.push(child_token),
                Err(_) => break,
            }
        }

        if children.len() < self.min {
            Err(FluxError::new_matcher("expected", pos, self.name.clone()))
        } else {
            Ok(Token {
                range: (pos..children.iter().last().unwrap().range.end),
                children,
                matcher_name: self.name.clone(),
                source,
            })
        }
    }

    fn min_length(&self) -> usize {
        self.child[0].borrow().min_length() * self.min
    }

    fn children(&self) -> Option<&MatcherChildren> {
        Some(&self.child)
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, new_name: String) {
        self.name.replace(Some(new_name));
    }
}
