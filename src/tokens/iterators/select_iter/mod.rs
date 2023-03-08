use self::{
    many::{IgnoreManyTokensIterator, SelectManyTokensIterator},
    single::{IgnoreSingleTokenIterator, SelectSingleTokenIterator},
};
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

pub trait SelectTokensIteratorExt<'a>: Iterator {
    fn select_tokens<S: ToString>(self, select_list: Vec<S>) -> SelectManyTokensIterator<'a, Self>
    where
        Self: Sized + Iterator<Item = &'a Token>,
    {
        SelectManyTokensIterator {
            select_list: select_list.into_iter().map(|s| s.to_string()).collect(),
            held_iter: self,
        }
    }

    fn select_token<S: ToString>(self, select: S) -> SelectSingleTokenIterator<'a, Self>
    where
        Self: Sized + Iterator<Item = &'a Token>,
    {
        SelectSingleTokenIterator {
            include: select.to_string(),
            held_iter: self,
        }
    }
}

impl<'a, I: Iterator<Item = &'a Token>> SelectTokensIteratorExt<'a> for I {}
