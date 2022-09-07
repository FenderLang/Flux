use std::cell::RefCell;
use std::rc::Rc;

use crate::error::{FluxError, Result};
use crate::matchers::char_range::CharRangeMatcher;
use crate::matchers::choice::ChoiceMatcher;
use crate::matchers::inverted::InvertedMatcher;
use crate::matchers::list::ListMatcher;
use crate::matchers::placeholder::PlaceholderMatcher;
use crate::matchers::repeating::RepeatingMatcher;
use crate::matchers::string::StringMatcher;
use crate::matchers::{char_set::CharSetMatcher, MatcherRef};

pub struct BNFParserState {
    pub source: Vec<char>,
    pub pos: usize,
}

impl BNFParserState {
    pub fn parse(&self) -> Result<MatcherRef> {
        todo!()
    }

    pub fn parse_rule(&mut self) -> Result<MatcherRef> {
        let name = self.parse_word()?;
        self.assert_whitespace()?;
        self.assert_string("::=")?;
        self.assert_whitespace()?;
        let mut matcher = self.parse_list()?;
        
        todo!()
    }

    pub fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    pub fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        self.pos += 1;
        c
    }

    pub fn assert_char(&mut self, match_char: char) -> Result<()> {
        match self.advance() {
            Some(c) if c == match_char => Ok(()),
            _ => Err(FluxError::new("", self.pos)),
        }
    }

    pub fn check_char(&mut self, match_char: char) -> bool {
        match self.peek() {
            Some(c) if c == match_char => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    pub fn assert_whitespace(&mut self) -> Result<()> {
        let mut whitespace = false;
        while self.peek().map_or(false, |c| c.is_whitespace()) {
            whitespace = true;
            self.advance();
        }
        if whitespace {
            Ok(())
        } else {
            Err(FluxError::new("", self.pos))
        }
    }

    pub fn assert_string(&mut self, match_string: &str) -> Result<()> {
        for c in match_string.chars() {
            self.assert_char(c)?
        }
        Ok(())
    }

    pub fn parse_char_or_escape_seq(&mut self) -> Result<char> {
        match self.advance() {
            Some('\\') => self.parse_escape_seq(),
            Some(c) => Ok(c),
            _ => Err(FluxError::new("", self.pos)),
        }
    }

    pub fn parse_escape_seq(&mut self) -> Result<char> {
        match self.advance() {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            _ => Err(FluxError::new("", self.pos)),
        }
    }

    pub fn parse_str_chars(&mut self, terminator: char) -> Result<String> {
        let mut out = String::new();
        while self.peek().map_or(false, |c| c != terminator) {
            let c = self.advance().unwrap();
            if c == '\\' {
                out.push(self.parse_escape_seq()?);
            } else {
                out.push(c);
            }
        }
        Ok(out)
    }

    pub fn parse_word(&mut self) -> Result<String> {
        let mut out = String::new();
        while self.peek().map_or(false, char::is_alphabetic) {
            out.push(self.advance().unwrap());
        }
        Ok(out)
    }

    pub fn parse_placeholder(&mut self) -> Result<MatcherRef> {
        let name = self.parse_word()?;
        let matcher = PlaceholderMatcher::new(name);
        Ok(Rc::new(matcher))
    }

    pub fn parse_number(&mut self) -> Result<usize> {
        let mut out = String::new();
        while self.peek().map_or(false, |c| c.is_digit(10)) {
            out.push(self.advance().unwrap());
        }
        out.parse().map_err(|_| FluxError::new("", self.pos))
    }

    pub fn parse_matcher_with_modifiers(&mut self) -> Result<MatcherRef> {
        let inverted = self.check_char('!');
        let mut matcher = self.parse_matcher()?;
        match self.peek() {
            Some('+') => {
                self.advance();
                matcher = Rc::new(RepeatingMatcher::new(1, usize::MAX, matcher));
            }
            Some('*') => {
                self.advance();
                matcher = Rc::new(RepeatingMatcher::new(0, usize::MAX, matcher));
            }
            Some('?') => {
                self.advance();
                matcher = Rc::new(RepeatingMatcher::new(0, 1, matcher));
            }
            Some('{') => {
                let bounds = self.parse_repeating_bounds()?;
                matcher = Rc::new(RepeatingMatcher::new(bounds.0, bounds.1, matcher));
            }
            _ => {}
        }
        if inverted {
            matcher = Rc::new(InvertedMatcher::new(matcher));
        }
        Ok(matcher)
    }

    pub fn parse_repeating_bounds(&mut self) -> Result<(usize, usize)> {
        self.assert_char('{')?;
        let min = if let Some(',') = self.peek() {0} else {self.parse_number()?};
        self.assert_char(',')?;
        let max = if let Some('}') = self.peek() {usize::MAX} else {self.parse_number()?};
        self.assert_char('}')?;
        Ok((min, max))
    }

    pub fn parse_matcher(&mut self) -> Result<MatcherRef> {
        match self.peek() {
            Some('(') => {
                self.assert_char('(')?;
                let list = self.parse_list()?;
                self.assert_char(')')?;
                Ok(list)
            }
            Some('[') => {
                let pos = self.pos;
                let matcher = self.parse_char_range().or_else(|_| {
                    self.pos = pos;
                    self.parse_char_set()
                })?;
                Ok(matcher)
            }
            Some('"') => self.parse_string(),
            Some(c) if c.is_alphabetic() => self.parse_placeholder(),
            _ => Err(FluxError::new("", self.pos)),
        }
    }

    pub fn parse_char_range(&mut self) -> Result<MatcherRef> {
        self.assert_char('[')?;
        let inverted = self.check_char('^');
        let low = self.parse_char_or_escape_seq()?;
        self.assert_char('-')?;
        let high = self.parse_char_or_escape_seq()?;
        self.assert_char(']')?;
        let matcher = CharRangeMatcher::new(low, high, inverted);
        Ok(Rc::new(matcher))
    }

    pub fn parse_char_set(&mut self) -> Result<MatcherRef> {
        self.assert_char('[')?;
        let inverted = self.check_char('^');
        let chars = self.parse_str_chars(']')?;
        self.assert_char(']')?;
        let matcher = CharSetMatcher::new(chars.chars().collect(), inverted);
        Ok(Rc::new(matcher))
    }

    pub fn parse_string(&mut self) -> Result<MatcherRef> {
        let case_sensitive = !self.check_char('i');
        self.assert_char('"')?;
        let chars = self.parse_str_chars('"')?;
        self.assert_char('"')?;
        let matcher = StringMatcher::new(chars, case_sensitive);
        Ok(Rc::new(matcher))
    }

    pub fn maybe_list(&mut self, mut list: Vec<MatcherRef>) -> MatcherRef {
        if list.len() == 1 {
            list.remove(0)
        } else {
            let children = list.into_iter().map(RefCell::new).collect();
            let list_matcher = ListMatcher::new(children);
            Rc::new(list_matcher)
        }
    }

    pub fn parse_list(&mut self) -> Result<MatcherRef> {
        let mut list = Vec::new();
        let mut choice = Vec::<MatcherRef>::new();
        while self.peek().map_or(false, |c| c != ')') {
            list.push(self.parse_matcher_with_modifiers()?);
            if !self.check_char(')') {
                self.assert_whitespace()?;
            }
            if self.check_char('|') {
                let matcher = self.maybe_list(list);
                list = Vec::new();
                choice.push(matcher);
                self.assert_whitespace()?;
            }
        }
        if !choice.is_empty() {
            let list_matcher = self.maybe_list(list);
            choice.push(list_matcher);
            let choice = choice.into_iter().map(RefCell::new).collect();
            let choice_matcher = ChoiceMatcher::new(choice);
            Ok(Rc::new(choice_matcher))
        } else {
            Ok(self.maybe_list(list))
        }
    }
}
