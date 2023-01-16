use super::{Matcher, MatcherName};
use crate::{error::FluxError, error::Result, tokens::Token};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct StringMatcher {
    name: MatcherName,
    to_match: Vec<char>,
    case_sensitive: bool,
    id: RefCell<usize>,
}

impl StringMatcher {
    pub fn new(to_match: String, case_sensitive: bool) -> Self {
        Self {
            name: Rc::new(RefCell::new(None)),
            to_match: to_match.chars().collect(),
            case_sensitive,
            id: RefCell::new(0),
        }
    }

    fn char_matches(&self, first: &char, second: &char) -> bool {
        if self.case_sensitive {
            first == second
        } else {
            first == second || first.eq_ignore_ascii_case(second)
        }
    }
}

impl Matcher for StringMatcher {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token> {
        let mut zip = self
            .to_match
            .iter()
            .zip(&source[pos..])
            .take(self.to_match.len());

        if zip.len() == self.to_match.len() && zip.all(|(a, b)| self.char_matches(a, b)) {
            Ok(Token {
                matcher_name: self.name.clone(),
                children: vec![],
                source,
                range: pos..pos + self.to_match.len(),
                matcher_id: *self.id.borrow(),
            })
        } else {
            Err(FluxError::new_matcher("expected", pos, self.name.clone()))
        }
    }

    fn min_length(&self) -> usize {
        self.to_match.len()
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, new_name: String) {
        self.name.replace(Some(new_name));
    }

    fn id(&self) -> &RefCell<usize> {
        &self.id
    }
}
