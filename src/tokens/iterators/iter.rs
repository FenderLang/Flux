use crate::tokens::Token;

pub struct Iter<'a> {
    token: &'a Token,
    index: usize,
}

impl<'a> Iter<'a> {
    pub fn new(token: &'a Token) -> Self {
        Self { token, index: 0 }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.token.children.len() {
            return None;
        }

        let next_token = &self.token.children[self.index];

        self.index += 1;

        Some(next_token)
    }
}
