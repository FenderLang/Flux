use crate::matchers::MatcherName;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub matcher_name: MatcherName,
    pub children: Vec<Token<'a>>,
    pub source: &'a [char],
    pub range: Range<usize>,
}
