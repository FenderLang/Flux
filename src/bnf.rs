use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::{FluxError, Result};
use crate::lexer::Lexer;
use crate::matchers::char_range::CharRangeMatcher;
use crate::matchers::choice::ChoiceMatcher;
use crate::matchers::inverted::InvertedMatcher;
use crate::matchers::list::ListMatcher;
use crate::matchers::placeholder::PlaceholderMatcher;
use crate::matchers::repeating::RepeatingMatcher;
use crate::matchers::string::StringMatcher;
use crate::matchers::{char_set::CharSetMatcher, MatcherRef};

pub fn parse(input: &str) -> Result<Lexer> {
    BNFParserState {
        source: input.chars().collect(),
        pos: 0,
    }
    .parse()
}

struct BNFParserState {
    source: Vec<char>,
    pos: usize,
}

impl BNFParserState {
    fn parse(&mut self) -> Result<Lexer> {
        self.consume_line_break();
        let mut rules = Vec::new();
        while self.pos < self.source.len() {
            rules.extend(self.parse_rule()?);
            self.consume_line_break();
        }
        let mut map: HashMap<String, MatcherRef> = HashMap::new();
        for rule in &rules {
            if let Some(name) = rule.get_name().borrow().as_ref() {
                if map.contains_key(name) {
                    return Err(FluxError::new_dyn(
                        format!("Duplicate rule name {}", name),
                        0,
                    ));
                }
                map.insert(name.clone(), rule.clone());
            }
        }
        for (rule, id) in map.values().zip(1..) {
            rule.id().replace(id);
            Self::replace_placeholders(&rule, &map)?;
        }
        let root = map
            .get("root")
            .ok_or_else(|| FluxError::new("No root matcher specified", 0))?;
        let mut id_map = HashMap::new();
        for rule in &rules {
            id_map.insert(
                rule.get_name().borrow().clone().unwrap(),
                *rule.id().borrow(),
            );
        }
        Ok(Lexer::new(root.clone(), id_map))
    }

    fn replace_placeholders(root: &MatcherRef, map: &HashMap<String, MatcherRef>) -> Result<()> {
        if root.children().is_none() {
            return Ok(());
        }
        let children = root.children().unwrap();
        for c in children {
            Self::replace_placeholders(&c.borrow(), map)?;
            if c.borrow().is_placeholder() {
                let name = c.borrow().get_name();
                let name = name.borrow();
                let name = name.as_ref().unwrap();
                let matcher = map.get(name).ok_or_else(|| {
                    FluxError::new_dyn(format!("Missing matcher for {}", name), 0)
                })?;
                c.replace(matcher.clone());
            }
        }
        Ok(())
    }

    fn parse_rule(&mut self) -> Result<Option<MatcherRef>> {
        if self.check_str("//") {
            self.consume_comment();
            return Ok(None);
        }
        let name = self.parse_word()?;
        self.call_assert("whitespace", Self::consume_whitespace)?;
        self.assert_str("::=")?;
        self.call_assert("whitespace", Self::consume_whitespace)?;
        let mut matcher = self.parse_list()?;
        if matcher.is_placeholder() {
            matcher = Rc::new(ListMatcher::new(vec![RefCell::new(matcher)]));
        }
        matcher.set_name(name);
        if self.check_str("//") {
            self.consume_comment();
        }
        Ok(Some(matcher))
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

    fn call_assert(&mut self, context: &str, func: fn(&mut Self)) -> Result<()> {
        let start = self.pos;
        func(self);
        if start == self.pos {
            Err(FluxError::new_dyn(
                format!("Expected {}", context),
                self.pos,
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

    fn consume_line_break(&mut self) {
        self.consume_whitespace();
        while self.peek() == Some('\n') {
            self.advance();
            self.consume_whitespace();
        }
    }

    fn is_whitespace(c: char) -> bool {
        c.is_whitespace() && c != '\n'
    }

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
            _ => Err(FluxError::new("Unexpected end of file", self.pos)),
        }
    }

    fn parse_escape_seq(&mut self) -> Result<char> {
        match self.advance() {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            _ => Err(FluxError::new("Invalid escape sequence", self.pos)),
        }
    }

    fn parse_str_chars(&mut self, terminator: char) -> Result<String> {
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

    fn parse_word(&mut self) -> Result<String> {
        let mut out = String::new();
        while self.peek().map_or(false, char::is_alphabetic) {
            out.push(self.advance().unwrap());
        }
        Ok(out)
    }

    fn parse_placeholder(&mut self) -> Result<MatcherRef> {
        let name = self.parse_word()?;
        let matcher = PlaceholderMatcher::new(name);
        Ok(Rc::new(matcher))
    }

    fn parse_number(&mut self) -> Result<usize> {
        let mut out = String::new();
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            out.push(self.advance().unwrap());
        }
        out.parse()
            .map_err(|_| FluxError::new("Invalid number", self.pos))
    }

    fn parse_matcher_with_modifiers(&mut self) -> Result<MatcherRef> {
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

    fn parse_repeating_bounds(&mut self) -> Result<(usize, usize)> {
        self.assert_char('{')?;
        let min = if let Some(',') = self.peek() {
            0
        } else {
            self.parse_number()?
        };
        self.assert_char(',')?;
        let max = if let Some('}') = self.peek() {
            usize::MAX
        } else {
            self.parse_number()?
        };
        self.assert_char('}')?;
        Ok((min, max))
    }

    fn parse_matcher(&mut self) -> Result<MatcherRef> {
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
            Some('i') if self.source.get(self.pos + 1) == Some(&'"') => self.parse_string(),
            Some(c) if c.is_alphabetic() => self.parse_placeholder(),
            _ => Err(FluxError::new(
                "Unexpected character or end of file",
                self.pos,
            )),
        }
    }

    fn parse_char_range(&mut self) -> Result<MatcherRef> {
        self.assert_char('[')?;
        let inverted = self.check_char('^');
        let low = self.parse_char_or_escape_seq()?;
        self.assert_char('-')?;
        let high = self.parse_char_or_escape_seq()?;
        self.assert_char(']')?;
        let matcher = CharRangeMatcher::new(low, high, inverted);
        Ok(Rc::new(matcher))
    }

    fn parse_char_set(&mut self) -> Result<MatcherRef> {
        self.assert_char('[')?;
        let inverted = self.check_char('^');
        let chars = self.parse_str_chars(']')?;
        self.assert_char(']')?;
        let matcher = CharSetMatcher::new(chars.chars().collect(), inverted);
        Ok(Rc::new(matcher))
    }

    fn parse_string(&mut self) -> Result<MatcherRef> {
        let case_sensitive = !self.check_char('i');
        self.assert_char('"')?;
        let chars = self.parse_str_chars('"')?;
        self.assert_char('"')?;
        let matcher = StringMatcher::new(chars, case_sensitive);
        Ok(Rc::new(matcher))
    }

    fn maybe_list(&mut self, mut list: Vec<MatcherRef>) -> MatcherRef {
        if list.len() == 1 {
            list.remove(0)
        } else {
            let children = list.into_iter().map(RefCell::new).collect();
            let list_matcher = ListMatcher::new(children);
            Rc::new(list_matcher)
        }
    }

    fn parse_list(&mut self) -> Result<MatcherRef> {
        let mut list = Vec::new();
        let mut choice = Vec::<MatcherRef>::new();
        while self.pos < self.source.len() && self.peek() != Some('\n') {
            list.push(self.parse_matcher_with_modifiers()?);
            let whitespace = self.call_check(Self::consume_whitespace);
            if !whitespace || self.check_char(')') {
                break;
            }
            if self.check_char('|') {
                let matcher = self.maybe_list(list);
                self.consume_whitespace();
                list = Vec::new();
                choice.push(matcher);
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
