use crate::tokens::Token;

#[derive(Debug)]
pub struct RecursiveIter<'a> {
    token: &'a Token,
    index: usize,
    stack: Vec<(&'a Token, usize)>,
    pub(super) selected: Vec<&'a str>,
    pub(super) ignored: Vec<&'a str>,
}

impl<'a> RecursiveIter<'a> {
    pub fn new(token: &'a Token) -> Self {
        Self {
            token,
            index: 0,
            stack: vec![(token, 0)],
            ignored: Vec::new(),
            selected: Vec::new(),
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

        if self.ignored.contains(
            &self.token.children[self.index]
                .get_name()
                .as_deref()
                .unwrap_or_default(),
        ) {
            self.index += 1;
            return self.next();
        }

        let next_child = &self.token.children[self.index];
        self.stack.push((self.token, self.index));

        self.index = 0;
        self.token = next_child;

        if !self.selected.is_empty()
            && !self
                .selected
                .contains(&self.token.get_name().as_deref().unwrap_or_default())
        {
            return self.next();
        }

        Some(self.token)
    }
}
