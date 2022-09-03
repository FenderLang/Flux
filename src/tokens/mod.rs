use std::{rc::Rc, ops::Range};

pub struct Token {
    pub matcher_name: Rc<String>,
    pub children: Vec<Token>,
    pub source: Rc<Vec<char>>,
    pub range: Range<usize>,
}