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
        source: Rc::new(input.chars().collect()),
        pos: 0,
        templates: HashMap::new(),
    }
    .parse()
}

fn replace_placeholders(rule: &MatcherRef, map: &HashMap<String, MatcherRef>, error_on_fail: bool) -> Result<()> {
    let Some(children) = rule.children() else {
        return Ok(());
    };
    for c in children {
        if c.borrow().is_placeholder() {
            let borrow = c.borrow();
            let name = &**borrow.name();
            let matcher = name.as_ref().and_then(|n| map.get(n));
            let Some(matcher) = matcher else {
                if error_on_fail {
                    return Err(FluxError::new_dyn(format!("Missing matcher for {}", name.as_ref().unwrap()), 0, None));
                } else {
                    continue;
                }
            };
            drop(borrow);
            c.replace(matcher.clone());
        } else {
            replace_placeholders(&c.borrow(), map, error_on_fail)?;
        }
    }
    Ok(())
}

struct TemplateRule {
    params: Vec<String>,
    rule: MatcherRef,
}

fn deep_copy(matcher: MatcherRef) -> MatcherRef {
    let matcher = matcher.with_meta(matcher.meta().clone());
    let Some(children) = matcher.children() else {
        return matcher;
    };
    for child in children {
        let clone = deep_copy(child.borrow().clone());
        child.replace(clone);
    }
    matcher
}

impl TemplateRule {
    fn to_matcher(&self, params: Vec<MatcherRef>, pos: usize, source: Rc<Vec<char>>) -> Result<MatcherRef> {
        let mut names = HashMap::new();
        if params.len() != self.params.len() {
            return Err(FluxError::new("wrong number of template arguments", pos, Some(source.clone())));
        }
        for (matcher, name) in params.into_iter().zip(&self.params) {
            names.insert(name.clone(), matcher);
        }
        let matcher = deep_copy(self.rule.clone());
        replace_placeholders(&matcher, &names, false)?;
        Ok(matcher)
    }
}

enum ParseLineResult {
    Rule(MatcherRef, String),
    Template(MatcherRef, Vec<String>, String),
}

struct BNFParserState {
    templates: HashMap<String, TemplateRule>,
    source: Rc<Vec<char>>,
    pos: usize,
}

impl BNFParserState {
    fn parse(&mut self) -> Result<Lexer> {
        let mut rules = Vec::new();
        let mut rule_map: HashMap<String, MatcherRef> = HashMap::new();
        let mut id_map = HashMap::new();
        self.consume_line_breaks();
        use ParseLineResult::*;
        while self.pos < self.source.len() {
            match self.parse_rule(rules.len() + 1)? {
                Some(Rule(rule, name)) => {
                    rules.push(rule.clone());
                    id_map.insert(name.clone(), rule.id());
                    if rule_map.insert(name.clone(), rule).is_some() {
                        return Err(FluxError::new_dyn(
                            format!("Duplicate rule name {}", name),
                            0,
                            Some(self.source.clone()),
                        ));
                    }
                },
                Some(Template(rule, params, name)) => {
                    self.templates.insert(name, TemplateRule { params, rule });
                }
                _ => (),
            }
            self.consume_line_breaks();
        }
        for rule in &rules {
            replace_placeholders(rule, &rule_map, true)?;
        }
        let root = rule_map
            .get("root")
            .ok_or_else(|| FluxError::new("No root matcher specified", 0, Some(self.source.clone())))?;
        Ok(Lexer::new(root.clone(), id_map))
    }

    fn parse_rule(&mut self, id: usize) -> Result<Option<ParseLineResult>> {
        if self.check_str("//") {
            self.consume_comment();
            return Ok(None);
        }
        let name = self.parse_word()?;
        let mut template_params = None;
        if self.check_char('<') {
            template_params = Some(self.parse_template_param_names()?);
        }
        let meta = MatcherMeta::new(Some(name.clone()), id);
        let transparent = self.check_str("!0");
        self.call_assert("whitespace", Self::consume_whitespace)?;
        self.assert_str("::=")?;
        self.call_assert("whitespace", Self::consume_whitespace)?;
        let mut matcher = self.parse_list()?;
        if matcher.is_placeholder() {
            matcher = Rc::new(ListMatcher::new(
                Default::default(),
                vec![RefCell::new(matcher)],
            ));
        }
        self.consume_whitespace();
        if self.check_str("//") {
            self.consume_comment();
        }
        if !transparent {
            matcher = matcher.with_meta(meta);
        }
        if let Some(params) = template_params {
            Ok(Some(ParseLineResult::Template(matcher, params, name)))
        } else {
            Ok(Some(ParseLineResult::Rule(matcher, name)))
        }
    }

    fn parse_template_param_names(&mut self) -> Result<Vec<String>> {
        let mut names = vec![self.parse_word()?];
        while let Some(c) = self.advance() {
            match c {
                ',' => {
                    self.consume_whitespace();
                    names.push(self.parse_word()?);
                }
                '>' => return Ok(names),
                _ => break,
            }
        }
        Err(FluxError::new("Expected >", self.pos, Some(self.source.clone())))
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
            _ => Err(FluxError::new("Unexpected end of file", self.pos, Some(self.source.clone()))),
        }
    }

    fn invalid_escape_sequence(&self) -> FluxError {
        FluxError::new("Invalid escape sequence", self.pos, Some(self.source.clone()))
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
        if self.check_char('<') {
            let params = self.parse_template_params()?;
            let template = self.templates.get(&name).ok_or(FluxError::new_dyn(
                format!("No template rule with name {name}"),
                self.pos,
                Some(self.source.clone()),
            ))?;
            return Ok(template.to_matcher(params, self.pos, self.source.clone())?);
        }
        let matcher = PlaceholderMatcher::new(MatcherMeta::new(Some(name), 0));
        Ok(Rc::new(matcher))
    }

    fn parse_template_params(&mut self) -> Result<Vec<MatcherRef>> {
        let mut params = vec![self.parse_list()?];
        while let Some(c) = self.advance() {
            match c {
                ',' => {
                    self.consume_whitespace();
                    params.push(self.parse_list()?);
                }
                '>' => return Ok(params),
                _ => break,
            }
        }
        Err(FluxError::new("Expected >", self.pos, Some(self.source.clone())))
    }

    fn parse_number(&mut self) -> Result<usize> {
        let mut out = String::new();
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            out.push(self.advance().unwrap());
        }
        out.parse()
            .map_err(|_| FluxError::new("Invalid number", self.pos, Some(self.source.clone())))
    }

    fn parse_matcher_with_modifiers(&mut self) -> Result<MatcherRef> {
        let inverted = self.check_char('!');
        let mut matcher = self.parse_matcher()?;
        let _pos = self.pos;
        match self.peek() {
            Some('+') => {
                self.advance();
                matcher = Rc::new(RepeatingMatcher::new(
                    Default::default(),
                    1,
                    usize::MAX,
                    matcher,
                ));
            }
            Some('*') => {
                self.advance();
                matcher = Rc::new(RepeatingMatcher::new(
                    Default::default(),
                    0,
                    usize::MAX,
                    matcher,
                ));
            }
            Some('?') => {
                self.advance();
                matcher = Rc::new(RepeatingMatcher::new(Default::default(), 0, 1, matcher));
            }
            Some('{') => {
                let bounds = self.parse_repeating_bounds()?;
                matcher = Rc::new(RepeatingMatcher::new(
                    Default::default(),
                    bounds.0,
                    bounds.1,
                    matcher,
                ));
            }
            _ => (),
        }
        if inverted {
            matcher = Rc::new(InvertedMatcher::new(Default::default(), matcher));
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
            Some('<') => {
                self.assert_str("<eof>")?;
                Ok(Rc::new(EofMatcher::new(Default::default())))
            }
            Some('"') => self.parse_string(),
            Some('i') if self.source.get(self.pos + 1) == Some(&'"') => self.parse_string(),
            Some(c) if c.is_alphabetic() => self.parse_placeholder(),
            _ => Err(FluxError::new(
                "Unexpected character or end of file",
                self.pos,
                Some(self.source.clone()),
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
        let matcher = CharRangeMatcher::new(Default::default(), low, high, inverted);
        Ok(Rc::new(matcher))
    }

    fn parse_char_set(&mut self) -> Result<MatcherRef> {
        self.assert_char('[')?;
        let inverted = self.check_char('^');
        let chars = self.parse_str_chars(']')?;
        self.assert_char(']')?;
        let matcher = CharSetMatcher::new(Default::default(), chars.chars().collect(), inverted);
        Ok(Rc::new(matcher))
    }

    fn parse_string(&mut self) -> Result<MatcherRef> {
        let case_sensitive = !self.check_char('i');
        self.assert_char('"')?;
        let chars = self.parse_str_chars('"')?;
        self.assert_char('"')?;
        let matcher = StringMatcher::new(Default::default(), chars, case_sensitive);
        Ok(Rc::new(matcher))
    }

    fn maybe_list(&mut self, mut list: Vec<MatcherRef>) -> MatcherRef {
        if list.len() == 1 {
            list.remove(0)
        } else {
            let children = list.into_iter().map(RefCell::new).collect();
            let list_matcher = ListMatcher::new(Default::default(), children);
            Rc::new(list_matcher)
        }
    }

    fn list_should_continue(&self) -> bool {
        let next = self.peek();
        next != Some('\n') && next != Some('/')
    }

    fn parse_list(&mut self) -> Result<MatcherRef> {
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
            let choices = choices.into_iter().map(RefCell::new).collect();
            Ok(Rc::new(ChoiceMatcher::new(Default::default(), choices)))
        } else {
            Ok(self.maybe_list(matcher_list))
        }
    }
}
