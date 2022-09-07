use super::{Matcher, MatcherChildren, MatcherName, MatcherRef};
use crate::{error::FluxError, tokens::Token};
use std::{cell::RefCell, rc::Rc};

pub struct RepeatingMatcher {
    name: MatcherName,
    min: usize,
    max: usize,
    child: MatcherChildren,
}

impl RepeatingMatcher {
    pub fn new(min: usize, max: usize, child: MatcherRef) -> Self {
        Self {
            name: Rc::new(RefCell::new(None)),
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
