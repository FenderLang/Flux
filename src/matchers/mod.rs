use crate::tokens::Token;
use std::rc::Rc;
use crate::error::Result;

pub type MatcherRef = Rc<dyn Matcher>;

pub trait Matcher {
    fn apply(&self, source: Vec<char>, pos: usize) -> Result<Token>;
    fn min_length(&self) -> usize;
    fn name(&self) -> &str;
    fn children(&self) -> Vec<MatcherRef>;
}

mod char_group;
mod choice;
mod list;
mod repeating;
mod string;