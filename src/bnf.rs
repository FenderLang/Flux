use std::collections::HashMap;
use std::sync::Arc;

use crate::matchers::{Matcher, MatcherType};
use crate::error::{FluxError, Result};
use crate::lexer::Lexer;

pub fn parse(input: &str) -> Result<Lexer> {
    let id_map: HashMap<String, usize> = input
        .lines()
        .map(str::trim_start)
        .filter(|s| s.len() > 0 && !s.starts_with('#'))
        .map(|s| s.chars().take_while(|c| !c.is_whitespace()).collect())
        .enumerate()
        .map(|(i, s)| (s, i))
        .collect();
    BNFParserState {
        matchers: vec![
            Matcher {
                name: None.into(),
                id: 0,
                matcher_type: MatcherType::Placeholder
            };
            id_map.len()
        ],
        id_map,
        source: Arc::new(input.chars().collect()),
        pos: 0,
    }
    .parse()
}

enum ParseLineResult {
    Rule(MatcherType, String),
}

struct BNFParserState {
    id_map: HashMap<String, usize>,
    matchers: Vec<Matcher>,
    source: Arc<Vec<char>>,
    pos: usize,
}

impl BNFParserState {
    fn parse(mut self) -> Result<Lexer> {
        self.consume_line_breaks();
        use ParseLineResult::*;
        while self.pos < self.source.len() {
            match self.parse_rule()? {
                Some(Rule(rule, name)) => {
                    self.add_named_matcher(rule, name);
                }
                _ => (),
            }
            self.consume_line_breaks();
        }
        let root = self.id_map.get("root").ok_or_else(|| {
            FluxError::new("No root matcher specified", 0, Some(self.source.clone()))
        })?;
        Ok(Lexer::new(*root, self.id_map, self.matchers))
    }

    fn add_matcher(&mut self, matcher_type: MatcherType) -> &Matcher {
        let matcher = Matcher {
            name: None.into(),
            id: self.matchers.len(),
            matcher_type,
        };
        self.matchers.push(matcher);
        &self.matchers[self.matchers.len() - 1]
    }

    fn add_named_matcher(&mut self, matcher_type: MatcherType, name: String) -> &Matcher {
        let id = self.id_map[&name];
        let matcher = Matcher {
            name: Some(name).into(),
            id,
            matcher_type,
        };
        self.matchers[id] = matcher;
        &self.matchers[id]
    }

    fn parse_rule(&mut self) -> Result<Option<ParseLineResult>> {
        if self.check_str("//") {
            self.consume_comment();
            return Ok(None);
        }
        let name = self.parse_word()?;
        self.call_assert("whitespace", Self::consume_whitespace)?;
        self.assert_str("::=")?;
        self.call_assert("whitespace", Self::consume_whitespace)?;
        let matcher = self.parse_list()?;
        self.consume_whitespace();
        if self.check_str("//") {
            self.consume_comment();
        }
        Ok(Some(ParseLineResult::Rule(matcher, name)))
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        self.pos += 1;
        c
    }

    fn assert_char(&mut self, match_char: char) -> Result<()> {
        match self.advance() {
            Some(c) if c == match_char => Ok(()),
            _ => Err(FluxError::new_dyn(
                format!("Expected {}", match_char),
                self.pos,
                Some(self.source.clone()),
            )),
        }
    }

    fn check_char(&mut self, match_char: char) -> bool {
        match self.peek() {
            Some(c) if c == match_char => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    fn check_str(&mut self, match_str: &str) -> bool {
        if match_str.len() + self.pos < self.source.len()
            && self.source[self.pos..self.pos + match_str.len()]
                .iter()
                .zip(match_str.chars())
                .all(|(c1, c2)| *c1 == c2)
        {
            self.pos += match_str.len();
            true
        } else {
            false
        }
    }

    fn call_assert(&mut self, error_context: &str, func: fn(&mut Self)) -> Result<()> {
        let start = self.pos;
        func(self);
        if start == self.pos {
            Err(FluxError::new_dyn(
                format!("Expected {}", error_context),
                self.pos,
                Some(self.source.clone()),
            ))
        } else {
            Ok(())
        }
    }

    fn call_check(&mut self, func: impl Fn(&mut Self)) -> bool {
        let start = self.pos;
        func(self);
        start != self.pos
    }

    fn consume_comment(&mut self) {
        while self.peek().map_or(false, |c| c != '\n') {
            self.pos += 1;
        }
    }

    /// Consume all whitespace including line breaks
    fn consume_line_breaks(&mut self) {
        self.consume_whitespace();
        while let Some('\n' | '\r') = self.peek() {
            self.advance();
            self.consume_whitespace();
        }
    }

    /// Identifies a non line break whitespace
    fn is_whitespace(c: char) -> bool {
        c.is_whitespace() && c != '\n'
    }

    /// Consume all whitespace excluding line breaks
    fn consume_whitespace(&mut self) {
        while self.peek().map_or(false, BNFParserState::is_whitespace) {
            self.advance();
        }
    }

    fn assert_str(&mut self, match_string: &str) -> Result<()> {
        for c in match_string.chars() {
            self.assert_char(c)?
        }
        Ok(())
    }

    fn parse_char_or_escape_seq(&mut self) -> Result<char> {
        match self.advance() {
            Some('\\') => self.parse_escape_seq(),
            Some(c) => Ok(c),
            _ => Err(FluxError::new(
                "Unexpected end of file",
                self.pos,
                Some(self.source.clone()),
            )),
        }
    }

    fn invalid_escape_sequence(&self) -> FluxError {
        FluxError::new(
            "Invalid escape sequence",
            self.pos,
            Some(self.source.clone()),
        )
    }

    fn parse_escape_seq(&mut self) -> Result<char> {
        match self.advance() {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('u') if self.pos + 4 < self.source.len() => {
                let parsed = u32::from_str_radix(
                    &self.source[self.pos..].iter().take(4).collect::<String>(),
                    16,
                )
                .map_err(|_| self.invalid_escape_sequence())?;
                self.pos += 4;
                char::from_u32(parsed).ok_or_else(|| self.invalid_escape_sequence())
            }
            _ => Err(self.invalid_escape_sequence()),
        }
    }

    fn parse_str_chars(&mut self, terminator: char) -> Result<String> {
        let mut contents = String::new();
        while matches!(self.peek(), Some(c) if c != terminator) {
            let c = self.advance().unwrap();
            if c == '\\' {
                contents.push(self.parse_escape_seq()?);
            } else {
                contents.push(c);
            }
        }
        Ok(contents)
    }

    fn parse_word(&mut self) -> Result<String> {
        let mut out = String::new();
        while matches!(self.peek(), Some(c) if c.is_alphabetic()) {
            out.push(self.advance().unwrap());
        }
        Ok(out)
    }

    fn parse_number(&mut self) -> Result<usize> {
        let mut out = String::new();
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            out.push(self.advance().unwrap());
        }
        out.parse()
            .map_err(|_| FluxError::new("Invalid number", self.pos, Some(self.source.clone())))
    }

    fn parse_matcher_with_modifiers(&mut self) -> Result<MatcherType> {
        let inverted = self.check_char('!');
        let mut matcher = self.parse_matcher()?;
        let _pos = self.pos;
        match self.peek() {
            Some('+') => {
                self.advance();
                let child = self.add_matcher(matcher);
                matcher = MatcherType::Repeating(child.id, 1..=usize::MAX);
            }
            Some('*') => {
                self.advance();
                let child = self.add_matcher(matcher);
                matcher = MatcherType::Repeating(child.id, 0..=usize::MAX);
            }
            Some('?') => {
                self.advance();
                let child = self.add_matcher(matcher);
                matcher = MatcherType::Repeating(child.id, 0..=1);
            }
            Some('{') => {
                let bounds = self.parse_repeating_bounds()?;
                let child = self.add_matcher(matcher);
                matcher = MatcherType::Repeating(child.id, bounds.0..=bounds.1);
            }
            _ => (),
        }
        if inverted {
            let child = self.add_matcher(matcher);
            matcher = MatcherType::Inverted(child.id);
        }
        Ok(matcher)
    }

    fn parse_repeating_bounds(&mut self) -> Result<(usize, usize)> {
        self.assert_char('{')?;
        let min = if let Some(',') = self.peek() {
            0
        } else {
            self.parse_number()?
        };
        if self.check_char(',') {
            let max = if let Some('}') = self.peek() {
                usize::MAX
            } else {
                self.parse_number()?
            };
            self.assert_char('}')?;
            Ok((min, max))
        } else {
            self.assert_char('}')?;
            Ok((min, min))
        }
    }

    fn parse_matcher(&mut self) -> Result<MatcherType> {
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
            Some('<') => {
                self.assert_str("<eof>")?;
                Ok(MatcherType::Eof)
            }
            Some('"') => self.parse_string(),
            Some('i') if self.source.get(self.pos + 1) == Some(&'"') => self.parse_string(),
            Some(c) if c.is_alphabetic() => {
                let name = self.parse_word()?;
                let id = self.id_map.get(&name).ok_or_else(|| {
                    FluxError::new_dyn(
                        format!("No template rule with name {name}"),
                        self.pos,
                        Some(self.source.clone()),
                    )
                })?;
                Ok(MatcherType::Wrapper(*id))
            }
            _ => Err(FluxError::new(
                "Unexpected character or end of file",
                self.pos,
                Some(self.source.clone()),
            )),
        }
    }

    fn parse_char_range(&mut self) -> Result<MatcherType> {
        self.assert_char('[')?;
        let inverted = self.check_char('^');
        let low = self.parse_char_or_escape_seq()?;
        self.assert_char('-')?;
        let high = self.parse_char_or_escape_seq()?;
        self.assert_char(']')?;
        Ok(MatcherType::CharRange(low..=high, inverted))
    }

    fn parse_char_set(&mut self) -> Result<MatcherType> {
        self.assert_char('[')?;
        let inverted = self.check_char('^');
        let chars = self.parse_str_chars(']')?;
        self.assert_char(']')?;
        Ok(MatcherType::CharSet(chars.chars().collect(), inverted))
    }

    fn parse_string(&mut self) -> Result<MatcherType> {
        let case_sensitive = !self.check_char('i');
        self.assert_char('"')?;
        let chars = self.parse_str_chars('"')?;
        self.assert_char('"')?;
        Ok(MatcherType::String(chars.chars().collect(), case_sensitive))
    }

    fn maybe_list(&mut self, mut list: Vec<MatcherType>) -> MatcherType {
        if list.len() == 1 {
            list.remove(0)
        } else {
            let children = list.into_iter().map(|m| self.add_matcher(m).id).collect();
            MatcherType::List(children)
        }
    }

    fn list_should_continue(&self) -> bool {
        let next = self.peek();
        next != Some('\n') && next != Some('/')
    }

    fn parse_list(&mut self) -> Result<MatcherType> {
        let mut matcher_list = Vec::new();
        let mut choices = Vec::new();
        while self.pos < self.source.len() && self.list_should_continue() {
            matcher_list.push(self.parse_matcher_with_modifiers()?);
            if !self.call_check(Self::consume_whitespace) {
                break;
            }
            if self.check_char('|') {
                let matcher = self.maybe_list(matcher_list);
                self.consume_whitespace();
                matcher_list = Vec::new();
                choices.push(matcher);
            }
        }
        if !choices.is_empty() {
            let list_matcher = self.maybe_list(matcher_list);
            choices.push(list_matcher);
            let choices = choices
                .into_iter()
                .map(|m| self.add_matcher(m).id)
                .collect();
            Ok(MatcherType::Choice(choices))
        } else {
            Ok(self.maybe_list(matcher_list))
        }
    }
}
