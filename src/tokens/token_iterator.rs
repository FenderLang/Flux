use super::Token;

pub struct Iter<'a> {
    token: &'a Token,
    index: usize,
    ignore_list: Vec<String>,
}

pub struct RecursiveIter<'a> {
    token: &'a Token,
    index: usize,
    stack: Vec<(&'a Token, usize)>,
    ignore_list: Vec<String>,
}

impl<'a> Iter<'a> {
    pub fn new(token: &'a Token) -> Self {
        Self {
            token,
            index: 0,
            ignore_list: Vec::new(),
        }
    }
    pub fn ignore<S: AsRef<str>>(mut self, name: S) -> Iter<'a> {
        self.ignore_list.push(name.as_ref().into());
        self
    }
}

impl<'a> RecursiveIter<'a> {
    pub fn new(token: &'a Token) -> Self {
        Self {
            token,
            index: 0,
            stack: vec![(token, 0)],
            ignore_list: Vec::new(),
        }
    }

    pub fn ignore<S: AsRef<str>>(mut self, name: S) -> RecursiveIter<'a> {
        self.ignore_list.push(name.as_ref().into());
        self
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

        if let Some(true) = next_token
            .get_name()
            .as_ref()
            .map(|name| self.ignore_list.contains(name))
        {
            self.next()
        } else {
            Some(next_token)
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

        if let Some(true) = self
            .token
            .get_name()
            .as_ref()
            .map(|name| self.ignore_list.contains(name))
        {
            self.next()
        } else {
            Some(self.token)
        }
    }
}
