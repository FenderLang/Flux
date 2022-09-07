use crate::error::Result;
use crate::tokens::Token;
use std::cell::RefCell;
use std::rc::Rc;

pub type MatcherRef = Rc<dyn Matcher>;
pub type MatcherChildren = Vec<RefCell<MatcherRef>>;
pub type MatcherName = Rc<RefCell<Option<String>>>;

pub trait Matcher {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token>;
    fn min_length(&self) -> usize;
    fn get_name(&self) -> MatcherName;
    fn set_name(&self, new_name: String);
    fn children(&self) -> Option<&MatcherChildren> {
        None
    }
    fn is_placeholder(&self) -> bool {
        false
    }
}

pub mod char_range;
pub mod char_set;
pub mod choice;
pub mod inverted;
pub mod list;
pub mod placeholder;
pub mod repeating;
pub mod string;
