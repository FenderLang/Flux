use crate::matchers::MatcherName;
use std::{ops::Range, fmt::Debug};

#[derive(Clone)]
pub struct Token<'a> {
    pub matcher_name: MatcherName,
    pub matcher_id: usize,
    pub children: Vec<Token<'a>>,
    pub source: &'a [char],
    pub range: Range<usize>,
}

impl<'a> Debug for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let matched = &self.source[self.range.start..self.range.end]
            .iter()
            .collect::<String>();
        let mut debug = f.debug_struct("Token");
        debug.field("name", &self.matcher_name.borrow().clone().unwrap_or("".to_string()));
        debug.field("match", matched);
        debug.field("range", &self.range);
        if !self.children.is_empty() {
            debug.field("children", &self.children);
        }
        debug.finish()
    }
}
