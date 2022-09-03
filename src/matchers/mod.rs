use crate::tokens::Token;
use std::rc::Rc;
use crate::error::Result;

pub type MatcherRef = Rc<dyn Matcher>;

pub trait Matcher {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token>;
    fn min_length(&self) -> usize;
    fn name(&self) -> &str;
    fn children(&self) -> Vec<MatcherRef>;
}

pub mod char_group;
pub mod choice;
pub mod list;
pub mod repeating;
pub mod string;