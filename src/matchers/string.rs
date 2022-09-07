use super::{Matcher, MatcherName};
use crate::{error::FluxError, tokens::Token};
use std::{cell::RefCell, rc::Rc};

pub struct StringMatcher {
    name: MatcherName,
    to_match: String,
    case_sensitive: bool,
}

impl StringMatcher {
    pub fn new(to_match: String, case_sensitive: bool) -> Self {
        Self {
            name: Rc::new(RefCell::new(None)),
            to_match,
            case_sensitive,
        }
    }
}

impl Matcher for StringMatcher {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> crate::error::Result<Token> {
        for (a, b) in source[pos..(self.to_match.len())]
            .iter()
            .zip(self.to_match.chars())
        {
            if !match self.case_sensitive {
                true => *a == b,
                false => a.eq_ignore_ascii_case(&b),
            } {
                return Err(FluxError::new_matcher(
                    "failed to match string",
                    pos,
                    self.name.clone(),
                ));
            }
        }

        Ok(Token {
            matcher_name: self.name.clone(),
            children: vec![],
            source,
            range: pos..self.to_match.len(),
        })
    }

    fn min_length(&self) -> usize {
        self.to_match.len()
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&mut self, new_name: String) {
        let tmp = *self.name.as_ref().borrow_mut() = Some(new_name);
    }
}
