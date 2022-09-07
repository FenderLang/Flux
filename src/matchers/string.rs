use super::Matcher;
use crate::{error::FluxError, tokens::Token};
use std::rc::Rc;

pub struct StringMatcher {
    name: Option<Rc<String>>,
    to_match: String,
    case_sensitive: bool,
}

impl StringMatcher {
    pub fn new<S: ToString>(name: Option<S>, to_match: String, case_sensitive: bool) -> Self {
        Self {
            name: name.map(|name| Rc::new(name.to_string())),
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

    fn name(&self) -> Option<&str> {
        if let Some(name) = &self.name {
            Some(name.as_str())
        } else {
            None
        }
    }
}
