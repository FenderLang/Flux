use self::{many::IgnoreManyTokensIterator, single::IgnoreSingleTokenIterator};
use crate::tokens::Token;

pub mod many;
pub mod single;

pub trait IgnoreTokensIteratorExt<'a>: Iterator {
    fn ignore_tokens<S: ToString>(self, ignore_list: Vec<S>) -> IgnoreManyTokensIterator<'a, Self>
    where
        Self: Sized + Iterator<Item = &'a Token>,
    {
        IgnoreManyTokensIterator {
            ignore_list: ignore_list.into_iter().map(|s| s.to_string()).collect(),
            held_iter: self,
        }
    }

    fn ignore_token<S: ToString>(self, ignore: S) -> IgnoreSingleTokenIterator<'a, Self>
    where
        Self: Sized + Iterator<Item = &'a Token>,
    {
        IgnoreSingleTokenIterator {
            ignore: ignore.to_string(),
            held_iter: self,
        }
    }
}

impl<'a, I: Iterator<Item = &'a Token>> IgnoreTokensIteratorExt<'a> for I {}
