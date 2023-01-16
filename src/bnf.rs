use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::{FluxError, Result};
use crate::lexer::Lexer;
use crate::matchers::char_range::CharRangeMatcher;
use crate::matchers::choice::ChoiceMatcher;
use crate::matchers::eof::EofMatcher;
use crate::matchers::inverted::InvertedMatcher;
use crate::matchers::list::ListMatcher;
use crate::matchers::placeholder::PlaceholderMatcher;
use crate::matchers::repeating::RepeatingMatcher;
use crate::matchers::string::StringMatcher;
use crate::matchers::MatcherMeta;
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
        let mut rules = Vec::new();
        self.consume_line_breaks();
        while self.pos < self.source.len() {
            rules.extend(self.parse_rule(rules.len() + 1)?);
            self.consume_line_breaks();
        }
        let mut rule_map: HashMap<String, MatcherRef> = HashMap::new();
        for rule in &rules {
            if let Some(name) = &**rule.get_name() {
                if rule_map.contains_key(name) {
                    return Err(FluxError::new_dyn(
                        format!("Duplicate rule name {}", name),
                        0,
                    ));
                }
                rule_map.insert(name.clone(), rule.clone());
            }
        }
        let root = rule_map
            .get("root")
            .ok_or_else(|| FluxError::new("No root matcher specified", 0))?;
        let mut id_map = HashMap::new();
        for rule in &rules {
            id_map.insert((**rule.get_name()).clone().unwrap(), rule.id());
        }
        Ok(Lexer::new(root.clone(), id_map))
    }

    fn replace_placeholders(rule: &MatcherRef, map: &HashMap<String, MatcherRef>) -> Result<()> {
        let children = match rule.children() {
            Some(children) => children,
            None => return Ok(()),
        };

        for c in children {
            Self::replace_placeholders(&c.borrow(), map)?;
            if c.borrow().is_placeholder() {
                let borrow = c.borrow();
                let name = &**borrow.get_name();
                let matcher = name.as_ref().and_then(|n| map.get(n)).ok_or_else(|| {
                    FluxError::new_dyn(format!("Missing matcher for {}", name.as_ref().unwrap()), 0)
                })?;
                c.replace(matcher.clone());
            }
        }
        Ok(())
    }

    fn parse_rule(&mut self, id: usize) -> Result<Option<MatcherRef>> {
        if self.check_str("//") {
            self.consume_comment();
            return Ok(None);
        }
        let name = self.parse_word()?;
        let meta = MatcherMeta::new(Some(name), id);
        self.call_assert("whitespace", Self::consume_whitespace)?;
        self.assert_str("::=")?;
        self.call_assert("whitespace", Self::consume_whitespace)?;
        let mut matcher = self.parse_list(meta)?;
        if matcher.is_placeholder() {
            matcher = Rc::new(ListMatcher::new(meta, vec![RefCell::new(matcher)]));
        }
        self.consume_whitespace();
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

    fn call_assert(&mut self, error_context: &str, func: fn(&mut Self)) -> Result<()> {
        let start = self.pos;
        func(self);
        if start == self.pos {
            Err(FluxError::new_dyn(
                format!("Expected {}", error_context),
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

    /// Consume all whitespace including line breaks
    fn consume_line_breaks(&mut self) {
        self.consume_whitespace();
        while self.peek() == Some('\n') {
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
            _ => Err(FluxError::new("Unexpected end of file", self.pos)),
        }
    }

    fn invalid_escape_sequence(&self) -> FluxError {
        FluxError::new("Invalid escape sequence", self.pos)
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

    fn parse_placeholder(&mut self) -> Result<MatcherRef> {
        let name = self.parse_word()?;
        let matcher = PlaceholderMatcher::new(MatcherMeta::new(Some(name), 0));
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

    fn parse_matcher_with_modifiers(&mut self, meta: MatcherMeta) -> Result<MatcherRef> {
        let inverted = self.check_char('!');
        let mut matcher = self.parse_matcher(meta)?;
        let pos = self.pos;
        match self.peek() {
            Some('+') => {
                self.advance();
                matcher = Rc::new(RepeatingMatcher::new(
                    meta,
                    1,
                    usize::MAX,
                    matcher.reset_meta(),
                ));
            }
            Some('*') => {
                self.advance();
                matcher = Rc::new(RepeatingMatcher::new(
                    meta,
                    0,
                    usize::MAX,
                    matcher.reset_meta(),
                ));
            }
            Some('?') => {
                self.advance();
                matcher = Rc::new(RepeatingMatcher::new(meta, 0, 1, matcher.reset_meta()));
            }
            Some('{') => {
                let bounds = self.parse_repeating_bounds()?;
                matcher = Rc::new(RepeatingMatcher::new(
                    meta,
                    bounds.0,
                    bounds.1,
                    matcher.reset_meta(),
                ));
            }
            _ => (),
        }
        if inverted {
            matcher = Rc::new(InvertedMatcher::new(meta, matcher.reset_meta()));
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

    fn parse_matcher(&mut self, meta: MatcherMeta) -> Result<MatcherRef> {
        match self.peek() {
            Some('(') => {
                self.assert_char('(')?;
                let list = self.parse_list(meta)?;
                self.assert_char(')')?;
                Ok(list)
            }
            Some('[') => {
                let pos = self.pos;
                let matcher = self.parse_char_range(meta).or_else(|_| {
                    self.pos = pos;
                    self.parse_char_set(meta)
                })?;
                Ok(matcher)
            }
            Some('<') => {
                self.assert_str("<eof>")?;
                Ok(Rc::new(EofMatcher::new(meta)))
            }
            Some('"') => self.parse_string(meta),
            Some('i') if self.source.get(self.pos + 1) == Some(&'"') => self.parse_string(meta),
            Some(c) if c.is_alphabetic() => self.parse_placeholder(),
            _ => Err(FluxError::new(
                "Unexpected character or end of file",
                self.pos,
            )),
        }
    }

    fn parse_char_range(&mut self, meta: MatcherMeta) -> Result<MatcherRef> {
        self.assert_char('[')?;
        let inverted = self.check_char('^');
        let low = self.parse_char_or_escape_seq()?;
        self.assert_char('-')?;
        let high = self.parse_char_or_escape_seq()?;
        self.assert_char(']')?;
        let matcher = CharRangeMatcher::new(meta, low, high, inverted);
        Ok(Rc::new(matcher))
    }

    fn parse_char_set(&mut self, meta: MatcherMeta) -> Result<MatcherRef> {
        self.assert_char('[')?;
        let inverted = self.check_char('^');
        let chars = self.parse_str_chars(']')?;
        self.assert_char(']')?;
        let matcher = CharSetMatcher::new(meta, chars.chars().collect(), inverted);
        Ok(Rc::new(matcher))
    }

    fn parse_string(&mut self, meta: MatcherMeta) -> Result<MatcherRef> {
        let case_sensitive = !self.check_char('i');
        self.assert_char('"')?;
        let chars = self.parse_str_chars('"')?;
        self.assert_char('"')?;
        let matcher = StringMatcher::new(meta, chars, case_sensitive);
        Ok(Rc::new(matcher))
    }

    fn maybe_list(&mut self, meta: MatcherMeta, mut list: Vec<MatcherRef>) -> MatcherRef {
        if list.len() == 1 {
            list.remove(0)
        } else {
            let children = list.into_iter().map(RefCell::new).collect();
            let list_matcher = ListMatcher::new(meta, children);
            Rc::new(list_matcher)
        }
    }

    fn list_should_continue(&self) -> bool {
        let next = self.peek();
        next != Some('\n') && next != Some('/')
    }

    fn parse_list(&mut self, meta: MatcherMeta) -> Result<MatcherRef> {
        let mut matcher_list = Vec::new();
        let mut choices = Vec::new();
        while self.pos < self.source.len() && self.list_should_continue() {
            matcher_list.push(self.parse_matcher_with_modifiers(Default::default())?);
            if !self.call_check(Self::consume_whitespace) {
                break;
            }
            if self.check_char('|') {
                let matcher = self.maybe_list(Default::default(), matcher_list);
                self.consume_whitespace();
                matcher_list = Vec::new();
                choices.push(matcher);
            }
        }
        if !choices.is_empty() {
            let list_matcher = self.maybe_list(Default::default(), matcher_list);
            choices.push(list_matcher);
            let choices = choices.into_iter().map(RefCell::new).collect();
            Ok(Rc::new(ChoiceMatcher::new(Default::default(), choices)))
        } else {
            Ok(self.maybe_list(meta, matcher_list))
        }
    }
}
