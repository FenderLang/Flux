use super::{Matcher, MatcherName};
use crate::{error::FluxError, tokens::Token};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

#[derive(Clone, Debug)]
pub struct CharSetMatcher {
    name: MatcherName,
    matching_set: HashSet<char>,
    inverted: bool,
}

impl CharSetMatcher {
    pub fn new(matching_set: HashSet<char>, inverted: bool) -> Self {
        Self {
            name: Rc::new(RefCell::new(None)),
            matching_set,
            inverted,
        }
    }

    pub fn check_char(&self, check_char: &char) -> bool {
        if self.inverted {
            !self.matching_set.contains(check_char)
        } else {
            self.matching_set.contains(check_char)
        }
    }
}

impl Matcher for CharSetMatcher {
    fn apply(
        &self,
        source: Rc<Vec<char>>,
        pos: usize,
    ) -> crate::error::Result<crate::tokens::Token> {
        match source.get(pos) {
            Some(c) => {
                if self.check_char(c) {
                    Ok(Token {
                        children: vec![],
                        matcher_name: self.name.clone(),
                        range: pos..pos + 1,
                        source: source.clone(),
                    })
                } else {
                    Err(FluxError::new_matcher(
                        "char_set no character at pos not in set",
                        pos,
                        self.name.clone(),
                    ))
                }
            }
            None => Err(FluxError::new_matcher(
                "char_set no character at the position",
                pos,
                self.name.clone(),
            )),
        }
    }

    fn min_length(&self) -> usize {
        1
    }

    fn get_name(&self) -> MatcherName {
        self.name.clone()
    }

    fn set_name(&self, new_name: String) {
        self.name.replace(Some(new_name));
    }
}
