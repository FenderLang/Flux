use super::Matcher;
use crate::tokens::Token;
pub struct CharGroupMatcher {
    name: String,
    min: char,
    max: char,
}

impl Matcher for CharGroupMatcher {

    fn min_length(&self) -> usize {
        1
    }

    fn name(&self) -> String {
        self.name
    }

    fn children(&self) -> Vec<super::MatcherRef> {
        vec![]
    }

    fn apply(&self, source: Vec<char>, pos: usize) -> crate::error::Result<Token> {
        todo!()
    }
}
