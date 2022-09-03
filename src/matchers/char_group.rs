use super::Matcher;
use crate::{error::FluxError, tokens::Token};
use std::{rc::Rc, vec};

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
                "expected single char but no characters remaining",
                pos,
                self.name.clone(),
            )),
            Some(_) => Err(FluxError::new_matcher(
                "expected character matcher",
                pos,
                self.name.clone(),
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
