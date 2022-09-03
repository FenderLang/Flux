use std::{ops::Deref, rc::Rc, vec};

use super::Matcher;
use crate::{error::FluxError, tokens::Token};

#[derive(Clone)]
pub struct CharGroupMatcher {
    name: Rc<String>,
    min: char,
    max: char,
}

impl CharGroupMatcher {
    pub fn new<S: ToString>(name: S, min: char, max: char) -> CharGroupMatcher {
        CharGroupMatcher {
            name: Rc::new(name.to_string()),
            max,
            min,
        }
    }
}

impl Matcher for CharGroupMatcher {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> crate::error::Result<Token> {
        match source.get(pos) {
            Some(c) if c >= &self.min && c <= &self.max => Ok(Token {
                matcher_name: self.name.clone(),
                children: vec![],
                source: source.clone(),
                range: pos..pos + 1,
            }),
            None => Err(FluxError::new_matcher(
                "expected single char but no characters remaining".into(),
                pos,
                self.name.deref().clone(),
            )),
            Some(c) => Err(FluxError::new_matcher(
                format!("expected character between {} and {} but found {}", self.min, self.max, c),
                pos,
                self.name.deref().clone(),
            )),
        }
    }

    fn min_length(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        &self.name
    }
}
