use crate::tokens::Token;

#[derive(Debug)]
pub struct Iter<'a> {
    token: &'a Token,
    index: usize,
    pub(super) selected: Vec<&'a str>,
    pub(super) ignored: Vec<&'a str>,
}

impl<'a> Iter<'a> {
    pub fn new(token: &'a Token) -> Self {
        Self {
            token,
            index: 0,
            selected: Vec::new(),
            ignored: Vec::new(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.token.children.len() {
            return None;
        }
        while self.ignored.contains(
            &self.token.children[self.index]
                .get_name()
                .as_deref()
                .unwrap_or_default(),
        ) {
            self.index += 1;
        }

        let next_token = &self.token.children[self.index];

        self.index += 1;

        if !self.selected.is_empty()
            && !self
                .selected
                .contains(&next_token.get_name().as_deref().unwrap_or_default())
        {
            return self.next();
        }

        Some(next_token)
    }
}
