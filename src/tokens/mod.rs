use std::{rc::Rc, ops::Range};

use crate::matchers::Matcher;

pub struct Token {
    pub matcher: Rc<dyn Matcher>,
    pub children: Vec<Token>,
    pub source: Rc<Vec<char>>,
    pub range: Range<usize>,
}