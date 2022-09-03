use super::{Matcher, MatcherChildren, MatcherRef};
use crate::{error::FluxError, tokens::Token};
use std::{cell::RefCell, ops::Deref, rc::Rc};

pub struct StringMatcher {
    name: Rc<String>,
    to_match: String,
    case_sensitive: bool,
}

impl StringMatcher {
    pub fn new<S: ToString>(name: S, to_match: String, case_sensitive: bool) -> Self {
        Self {
            name: Rc::new(name.to_string()),
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
                    "failed to match string".into(),
                    pos,
                    self.name.deref().clone(),
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

    fn name(&self) -> &str {
        &self.name
    }
}
