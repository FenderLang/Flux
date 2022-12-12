use crate::matchers::MatcherName;
use std::{fmt::Debug, ops::{Range, Deref}, rc::Rc};

#[derive(Clone)]
pub struct Token {
    pub matcher_name: MatcherName,
    pub matcher_id: usize,
    pub children: Vec<Token>,
    pub source: Rc<Vec<char>>,
    pub range: Range<usize>,
}

impl Token {
    fn get_match(&self) -> String {
        self.source[self.range.clone()].iter().collect()
    }

    pub fn get_name(&self)->Option<String>{
        match self.matcher_name.deref().borrow().deref(){
            Some(v) => Some(String::from(v)),
            None => None,
        }
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Token");
        debug.field(
            "name",
            &self
                .matcher_name
                .borrow()
                .clone()
                .unwrap_or_default(),
        );
        debug.field("match", &self.get_match());
        debug.field("range", &self.range);
        if !self.children.is_empty() {
            debug.field("children", &self.children);
        }
        debug.finish()
    }
}
