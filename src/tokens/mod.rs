use std::{ops::Range, rc::Rc};

use crate::matchers::MatcherName;

pub struct Token {
    pub matcher_name: MatcherName,
    pub children: Vec<Token>,
    pub source: Rc<Vec<char>>,
    pub range: Range<usize>,
}
