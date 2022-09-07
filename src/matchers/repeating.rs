use super::{Matcher, MatcherChildren, MatcherRef};
use crate::{error::FluxError, tokens::Token};
use std::{cell::RefCell, rc::Rc};

pub struct RepeatingMatcher {
    name: Option<Rc<String>>,
    min: usize,
    max: usize,
    child: MatcherChildren,
}

impl RepeatingMatcher {
    pub fn new<S: ToString>(name: Option<S>, min: usize, max: usize, child: MatcherRef) -> Self {
        Self {
            name: name.map(|name| Rc::new(name.to_string())),
            min,
            max,
            child: vec![RefCell::new(child.clone())],
        }
    }
}

impl Matcher for RepeatingMatcher {
    fn apply(
        &self,
        source: Rc<Vec<char>>,
        pos: usize,
    ) -> crate::error::Result<crate::tokens::Token> {
        let mut children: Vec<Token> = Vec::new();

        for child in self.child.iter() {
            if children.len() >= self.max {
                break;
            }

            match child.borrow().apply(source.clone(), pos) {
                Ok(child_token) => children.push(child_token),
                Err(_) => {
                    return Err(FluxError::new_matcher(
                        "failed in repeating matcher",
                        pos,
                        self.name.clone(),
                    ))
                } //TODO don't remember to fix later
            }
        }

        if children.len() < self.min {
            Err(FluxError::new_matcher(
                "failed in repeating matcher did not match required number of times",
                pos,
                self.name.clone(),
            ))
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

    fn name(&self) -> Option<&str> {
        if let Some(name) = &self.name {
            Some(name.as_str())
        } else {
            None
        }
    }

    fn children(&self) -> Option<&MatcherChildren> {
        Some(&self.child)
    }
}
