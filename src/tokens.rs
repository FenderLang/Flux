use crate::matchers::MatcherName;
use std::{fmt::Debug, ops::Range};

#[derive(Clone)]
pub struct Token<'a> {
    pub matcher_name: MatcherName,
    pub matcher_id: usize,
    pub children: Vec<Token<'a>>,
    pub source: &'a [char],
    pub range: Range<usize>,
}

impl<'a> Token<'a> {
    fn get_match(&self) -> String {
        self.source[self.range.clone()].iter().collect()
    }
}

impl<'a> Debug for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Token");
        debug.field(
            "name",
            &self.matcher_name.borrow().clone().unwrap_or("".to_string()),
        );
        debug.field("match", &self.get_match());
        debug.field("range", &self.range);
        if !self.children.is_empty() {
            debug.field("children", &self.children);
        }
        debug.finish()
    }
}
