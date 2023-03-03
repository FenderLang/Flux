use crate::tokens::Token;

pub struct IgnoreManyTokensIterator<'a, I>
where
    I: Iterator<Item = &'a Token>,
{
    pub(super)ignore_list: Vec<String>,
    pub(super)held_iter: I,
}

impl<'a, I> Iterator for IgnoreManyTokensIterator<'a, I>
where
    I: Iterator<Item = &'a Token>,
{
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(next_token) = self.held_iter.next() else{
            return None;
        };
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
