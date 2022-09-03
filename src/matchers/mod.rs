use crate::tokens::Token;

pub trait Matcher {

    fn apply(&self) -> Option<Token>;
    fn min_length(&self) -> usize;
    fn name(&self) -> String;

}