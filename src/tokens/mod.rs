use std::{rc::Rc, ops::Range};

use crate::matchers::Matcher;

pub struct Token {
    matcher: Rc<dyn Matcher>,
    children: Vec<Token>,
    source: Rc<Vec<char>>,
    range: Range<usize>,
}