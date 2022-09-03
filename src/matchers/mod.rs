use std::rc::Rc;
use crate::tokens::Token;

type MatcherRef = Rc<dyn Matcher>;

pub trait Matcher {
    fn apply(&self, source: Vec<char>, pos: usize) -> Option<Token>;
    fn min_length(&self) -> usize;
    fn name(&self) -> String;
    fn children(&self) -> Vec<MatcherRef>;
}
