use std::{ops::Range, rc::Rc};

pub struct Token {
    pub matcher_name: Option<Rc<String>>,
    pub children: Vec<Token>,
    pub source: Rc<Vec<char>>,
    pub range: Range<usize>,
}
