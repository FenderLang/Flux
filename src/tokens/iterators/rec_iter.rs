use crate::tokens::Token;

use super::select_iter::GetDepthTokenIter;

#[derive(Debug)]
pub struct RecursiveIter<'a> {
    token: &'a Token,
    index: usize,
    stack: Vec<(&'a Token, usize)>,
}

impl<'a> RecursiveIter<'a> {
    pub fn new(token: &'a Token) -> Self {
        Self {
            token,
            index: 0,
            stack: vec![(token, 0)],
        }
    }
}

impl<'a> Iterator for RecursiveIter<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index >= self.token.children.len() {
            let (popped_token, popped_index) = match self.stack.pop() {
                Some(popped_values) => popped_values,
                None => return None,
            };
            self.index = popped_index;
            self.token = popped_token;
            self.index += 1;
        }

        let next_child = &self.token.children[self.index];
        self.stack.push((self.token, self.index));

        self.index = 0;
        self.token = next_child;

        Some(self.token)
    }
}

impl<'a> GetDepthTokenIter for RecursiveIter<'a> {
    fn get_depth(&self) -> i32 {
        self.stack.len() as i32
    }
}
