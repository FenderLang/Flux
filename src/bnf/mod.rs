use crate::matchers::MatcherRef;

pub struct BNFParser {
    pub source: Vec<char>,
    pub pos: usize,
}

impl BNFParser {
    pub fn parse(&self) -> MatcherRef {
        todo!()
    }
}
