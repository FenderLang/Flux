use std::ops::Range;

use crate::matchers::MatcherName;

#[derive(Debug)]
pub struct Token<'a> {
    pub matcher_name: MatcherName,
    pub children: Vec<Token<'a>>,
    pub source: &'a Vec<char>,
    pub range: Range<usize>,
}
