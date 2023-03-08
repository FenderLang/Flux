use crate::tokens::Token;

#[derive(Debug)]
pub struct IgnoreSingleTokenIterator<'a, I>
where
    I: Iterator<Item = &'a Token>,
{
    pub(super) ignore: String,
    pub(super) held_iter: I,
}

impl<'a, I> Iterator for IgnoreSingleTokenIterator<'a, I>
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
            .map(|name| self.ignore == *name)
        {
            self.next()
        } else {
            Some(next_token)
        }
    }
}

#[derive(Debug)]
pub struct SelectSingleTokenIterator<'a, I>
where
    I: Iterator<Item = &'a Token>,
{
    pub(super) include: String,
    pub(super) held_iter: I,
}

impl<'a, I> Iterator for SelectSingleTokenIterator<'a, I>
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
            .map(|name| self.include != *name)
        {
            self.next()
        } else {
            Some(next_token)
        }
    }
}
