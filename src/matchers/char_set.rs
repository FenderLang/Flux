use super::Matcher;
use crate::{error::FluxError, tokens::Token};
use std::{collections::HashSet, rc::Rc};

#[derive(Clone, Debug)]
pub struct CharSetMatcher {
    name: Rc<String>,
    matching_set: HashSet<char>,
}

impl CharSetMatcher {
    pub fn new<S: ToString>(name: S, matching_set: HashSet<char>) -> Self {
        Self {
            name: Rc::new(name.to_string()),
            matching_set,
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
                if self.matching_set.contains(c) {
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

    fn name(&self) -> &str {
        &self.name
    }
}
