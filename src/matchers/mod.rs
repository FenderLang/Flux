use std::rc::Rc;

use crate::tokens::Token;

pub trait Matcher {
    fn apply(&self, source: Vec<char>, pos: usize) -> Option<Token>;
    fn min_length(&self) -> usize;
    fn name(&self) -> String;
    fn children(&self) -> Vec<Rc<dyn Matcher>>;
}
