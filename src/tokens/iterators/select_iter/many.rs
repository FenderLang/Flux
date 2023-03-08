use crate::tokens::Token;

use super::GetDepthTokenIter;

#[derive(Debug)]
pub struct IgnoreManyTokensIterator<'a, I>
where
    I: Iterator<Item = &'a Token>,
{
    pub(super) ignore_list: Vec<String>,
    pub(super) held_iter: I,
}

impl<'a, I> Iterator for IgnoreManyTokensIterator<'a, I>
where
    I: Iterator<Item = &'a Token> + GetDepthTokenIter,
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
            let target_depth = self.held_iter.get_depth();
            loop {
                dbg!("loop");
                dbg!(target_depth);
                dbg!(self.held_iter.get_depth());
                let next = self.next();

                if self.held_iter.get_depth() == target_depth {
                    return next;
                }
            }
        } else {
            Some(next_token)
        }
    }
}

#[derive(Debug)]
pub struct SelectManyTokensIterator<'a, I>
where
    I: Iterator<Item = &'a Token>,
{
    pub(super) select_list: Vec<String>,
    pub(super) held_iter: I,
}

impl<'a, I> Iterator for SelectManyTokensIterator<'a, I>
where
    I: Iterator<Item = &'a Token>,
{
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(next_token) = self.held_iter.next() else{
            return None;
        };
        if let Some(false) = next_token
            .get_name()
            .as_ref()
            .map(|name| self.select_list.contains(name))
        {
            self.next()
        } else {
            Some(next_token)
        }
    }
}
