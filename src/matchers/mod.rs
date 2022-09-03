use crate::error::Result;
use crate::tokens::Token;
use std::cell::RefCell;
use std::rc::Rc;

pub type MatcherRef = Rc<dyn Matcher>;
pub type MatcherChildren = Vec<RefCell<MatcherRef>>;

pub trait Matcher {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token>;
    fn min_length(&self) -> usize;
    fn name(&self) -> &str;
    fn children(&self) -> Option<&MatcherChildren> {
        None
    }
    fn is_placeholder(&self) -> bool {
        false
    }
}

pub mod char_group;
pub mod choice;
pub mod list;
pub mod repeating;
pub mod string;
pub mod placeholder;
pub mod inverter;
