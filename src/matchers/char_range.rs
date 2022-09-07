use super::Matcher;
use crate::{error::FluxError, tokens::Token};
use std::{rc::Rc, vec};

#[derive(Clone)]
pub struct CharRangeMatcher {
    name: Option<Rc<String>>,
    min: char,
    max: char,
    inverted: bool,
}

impl CharRangeMatcher {
    pub fn new<S: ToString>(
        name: Option<S>,
        min: char,
        max: char,
        inverted: bool,
    ) -> CharRangeMatcher {
        CharRangeMatcher {
            name: name.map(|name| Rc::new(name.to_string())),
            max,
            min,
            inverted,
        }
    }

    pub fn check_char(&self, check_char: char) -> bool {
        if self.inverted {
            check_char < self.min || check_char > self.max
        } else {
            check_char >= self.min && check_char <= self.max
        }
    }
}

impl Matcher for CharRangeMatcher {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> crate::error::Result<Token> {
        match source.get(pos) {
            Some(c) if self.check_char(*c) => Ok(Token {
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

    fn name(&self) -> Option<&str> {
        if let Some(name) = &self.name {
            Some(name.as_str())
        } else {
            None
        }
    }
}
