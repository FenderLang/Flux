use crate::tokens::Token;

pub mod iter;
pub mod rec_iter;

pub trait SelectTokens<'a>: Iterator<Item = &'a Token<'a>> {
    fn select_tokens(self, select_list: Vec<&'a str>) -> Self;

    fn select_token(self, select: &'a str) -> Self;
}

pub trait IgnoreTokens<'a>: Iterator<Item = &'a Token<'a>> {
    fn ignore_tokens(self, ignore_list: Vec<&'a str>) -> Self;

    fn ignore_token(self, ignore: &'a str) -> Self;
}

macro_rules! impl_token_iter_traits {
    ($name:ident) => {
        impl<'a> SelectTokens<'a> for $name<'a> {
            fn select_tokens(mut self, mut select_list: Vec<&'a str>) -> Self {
                self.selected.append(&mut select_list);
                self
            }

            fn select_token(mut self, select: &'a str) -> Self {
                self.selected.push(select);
                self
            }
        }

        impl<'a> IgnoreTokens<'a> for $name<'a> {
            fn ignore_tokens(mut self, mut ignore_list: Vec<&'a str>) -> Self {
                self.ignored.append(&mut ignore_list);
                self
            }

            fn ignore_token(mut self, ignore: &'a str) -> Self {
                self.ignored.push(ignore);
                self
            }
        }
    };
}

use iter::Iter;
use rec_iter::RecursiveIter;

impl_token_iter_traits!(RecursiveIter);
impl_token_iter_traits!(Iter);
