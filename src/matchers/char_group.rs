use std::rc::Rc;

use super::Matcher;
use crate::{error::FluxError, tokens::Token};

#[derive(Clone)]
pub struct CharGroupMatcher {
    name: String,
    min: char,
    max: char,
}

impl Matcher for CharGroupMatcher {
    fn apply(&self, source: Vec<char>, pos: usize) -> crate::error::Result<Token> {
        todo!()
    }

    fn min_length(&self) -> usize {
        todo!()
    }

    fn name(&self) -> &str {
        todo!()
    }

    fn children(&self) -> Vec<super::MatcherRef> {
        todo!()
    }
}

impl Matcher for Rc<CharGroupMatcher> {
    fn apply(&self, source: Vec<char>, pos: usize) -> crate::error::Result<Token> {
        let check_char = match source.get(pos) {
            Some(c) => c,
            None => {
                return Err(FluxError::new_matcher(
                    "expected single char but no characters remaining".into(),
                    pos,
                    self.name.clone(),
                ))
            }
        };
        let tmp = Token{
            matcher: self.clone(),
            children: todo!(),
            source: todo!(),
            range: todo!(),
        };
        todo!()
    }

    fn min_length(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn children(&self) -> Vec<super::MatcherRef> {
        vec![]
    }
}
