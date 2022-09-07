use crate::matchers::MatcherRef;
use crate::error::{Result, FluxError};

pub struct BNFParserState {
    pub source: Vec<char>,
    pub pos: usize,
}

impl BNFParserState {
    pub fn parse(&self) -> Result<MatcherRef> {
        todo!()
    }

    pub fn peek(&self) -> Option<char> {
        self.source.get(self.pos).map(|c| *c)
    }

    pub fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        self.pos += 1;
        return c;
    }

    pub fn assert_char(&mut self, match_char: char) -> Result<()> {
        match self.advance() {
            Some(c) if c == match_char => Ok(()),
            _ => Err(FluxError::new("", self.pos))
        }
    }

    pub fn assert_string(&mut self, match_string: &str) -> Result<()> {
        for c in match_string.chars() {
            self.assert_char(c)?
        }
        Ok(())
    }

    pub fn parse_word(&mut self) -> Result<String> {
        let mut out = String::new();
        while self.peek().map_or(false, |c| c.is_alphabetic()) {
            out.push(self.advance().unwrap());
        }
        Ok(out)
    }
}