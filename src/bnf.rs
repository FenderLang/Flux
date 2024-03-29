use crate::error::{FluxError, Result};
use crate::lexer::{CullStrategy, Lexer};
use crate::matchers::{Matcher, MatcherType};
use std::collections::HashMap;
use std::sync::Arc;

const COMMENT_SYMBOL: &str = "//";
const ERROR_TRANSPARENT_SYMBOL: char = '!';

pub fn parse(input: &str) -> Result<Lexer> {
    let id_map: HashMap<String, usize> = input
        .lines()
        .map(str::trim_start)
        .filter(|s| !s.is_empty() && !s.starts_with(COMMENT_SYMBOL))
        .map(|s| {
            s.chars()
                .take_while(|c| !c.is_whitespace() && *c != ERROR_TRANSPARENT_SYMBOL)
                .collect()
        })
        .filter(|s: &String| !s.contains('<'))
        .enumerate()
        .map(|(i, s)| (s, i))
        .collect();
    BNFParserState {
        matchers: vec![
            Matcher {
                name: None.into(),
                id: 0,
                matcher_type: MatcherType::Placeholder,
                cull_strategy: CullStrategy::None,
                show_in_errors: true,
            };
            id_map.len()
        ],
        templates: HashMap::new(),
        id_map,
        source: input.chars().collect(),
        pos: 0,
    }
    .parse()
}

struct BNFParserState {
    id_map: HashMap<String, usize>,
    templates: HashMap<String, TemplateRule>,
    matchers: Vec<Matcher>,
    source: Arc<[char]>,
    pos: usize,
}

#[derive(Clone)]
struct TemplateRule {
    rule_start: usize,
    names: Vec<String>,
}

enum ParseLineOutput {
    Rule(MatcherType, String, bool),
    TemplateRule(TemplateRule, String),
}

impl BNFParserState {
    fn parse(mut self) -> Result<Lexer> {
        self.consume_line_breaks();
        while self.pos < self.source.len() {
            match self.parse_rule()? {
                Some(ParseLineOutput::Rule(rule, name, show_in_errors)) => {
                    self.add_named_matcher(rule, name, show_in_errors);
                }
                Some(ParseLineOutput::TemplateRule(rule, name)) => {
                    self.templates.insert(name, rule);
                }
                _ => (),
            }
            self.consume_line_breaks();
        }
        self.flatten_wrappers();
        let root = self.id_map.get("root").ok_or_else(|| {
            FluxError::new("No root matcher specified", 0, Some(self.source.clone()))
        })?;
        Ok(Lexer::new(*root, self.id_map, self.matchers))
    }

    fn flatten_wrappers(&mut self) {
        let mut wrappers: HashMap<usize, usize> = HashMap::new();
        for (index, matcher) in self.matchers.iter().enumerate() {
            if let MatcherType::Wrapper(child) = matcher.matcher_type {
                wrappers.insert(index, child);
            };
        }
        for index in 0..self.matchers.len() {
            let Some(children) = self.matchers[index].children() else {continue};
            for child in children {
                *child = *wrappers.get(child).unwrap_or(child);
            }
        }
    }

    fn add_matcher(&mut self, matcher_type: MatcherType) -> &Matcher {
        let matcher = Matcher {
            name: None.into(),
            id: self.matchers.len(),
            matcher_type,
            cull_strategy: CullStrategy::None,
            show_in_errors: false,
        };
        self.matchers.push(matcher);
        &self.matchers[self.matchers.len() - 1]
    }

    fn add_named_matcher(
        &mut self,
        matcher_type: MatcherType,
        name: String,
        show_in_errors: bool,
    ) -> &Matcher {
        let id = self.id_map[&name];
        let matcher = Matcher {
            name: Some(name).into(),
            id,
            matcher_type,
            cull_strategy: CullStrategy::None,
            show_in_errors,
        };
        self.matchers[id] = matcher;
        &self.matchers[id]
    }

    fn parse_rule(&mut self) -> Result<Option<ParseLineOutput>> {
        if self.check_str(COMMENT_SYMBOL) {
            self.consume_comment();
            return Ok(None);
        }
        let name = self.parse_word()?;
        let show_in_errors = !self.check_char(ERROR_TRANSPARENT_SYMBOL);
        let template = self.check_char('<').then(|| self.parse_generic_params());
        self.call_assert("whitespace", Self::consume_whitespace)?;
        self.assert_str("::=")?;
        self.call_assert("whitespace", Self::consume_whitespace)?;
        if let Some(params) = template {
            let template_rule = TemplateRule {
                rule_start: self.pos,
                names: params?,
            };
            while !self.check_char('\n') {
                self.advance();
            }
            return Ok(Some(ParseLineOutput::TemplateRule(template_rule, name)));
        }
        let matcher = self.parse_list(&None)?;
        self.consume_whitespace();
        if self.check_str(COMMENT_SYMBOL) {
            self.consume_comment();
        }
        Ok(Some(ParseLineOutput::Rule(matcher, name, show_in_errors)))
    }

    fn parse_generic_params(&mut self) -> Result<Vec<String>> {
        let mut names = vec![];
        while !self.check_char('>') {
            let name = self.parse_word()?;
            names.push(name);
            if self.check_char(',') {
                self.consume_whitespace();
            }
        }
        Ok(names)
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
        if self.source[self.pos..]
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
        while matches!(self.peek(), Some(c) if c.is_alphabetic() || c == '_') {
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

    fn parse_matcher_with_modifiers(
        &mut self,
        extras: &Option<HashMap<String, usize>>,
    ) -> Result<MatcherType> {
        let inverted = self.check_char('!');
        let mut matcher = self.parse_matcher(extras)?;
        let _pos = self.pos;
        match self.peek() {
            Some('+') => {
                self.advance();
                let child = self.add_matcher(matcher);
                matcher = MatcherType::Repeating(child.id, 1..=usize::MAX, None);
            }
            Some('*') => {
                self.advance();
                let child = self.add_matcher(matcher);
                matcher = MatcherType::Repeating(child.id, 0..=usize::MAX, None);
            }
            Some('?') => {
                self.advance();
                let child = self.add_matcher(matcher);
                matcher = MatcherType::Repeating(child.id, 0..=1, None);
            }
            Some('{') => {
                let bounds = self.parse_repeating_bounds()?;
                let child = self.add_matcher(matcher);
                matcher = MatcherType::Repeating(child.id, bounds.0..=bounds.1, None);
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

    fn parse_matcher(&mut self, extras: &Option<HashMap<String, usize>>) -> Result<MatcherType> {
        match self.peek() {
            Some('(') => {
                self.assert_char('(')?;
                let list = self.parse_list(extras)?;
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
                if self.check_str("<eof>") {
                    Ok(MatcherType::Eof)
                } else if self.check_str("<nl>") {
                    Ok(MatcherType::Newline)
                } else {
                    Err(self.create_error("Expected <eof> or <nl>".to_string()))
                }
            }
            Some('"') => self.parse_string(),
            Some('i') if self.source.get(self.pos + 1) == Some(&'"') => self.parse_string(),
            Some(c) if c.is_alphabetic() => self.parse_named(extras),
            _ => Err(FluxError::new(
                "Unexpected character or end of file",
                self.pos,
                Some(self.source.clone()),
            )),
        }
    }

    fn parse_named(&mut self, extras: &Option<HashMap<String, usize>>) -> Result<MatcherType> {
        let name = self.parse_word()?;
        if self.check_char('<') {
            let template = self
                .templates
                .get(&name)
                .ok_or_else(|| self.create_error(format!("No template rule with name {name}")))?;
            return self.parse_template(template.clone(), extras);
        }
        let id = extras
            .as_ref()
            .and_then(|m| m.get(&name))
            .or_else(|| self.id_map.get(&name));
        let id = id.ok_or_else(|| self.create_error(format!("No rule with name {name}")))?;
        Ok(MatcherType::Wrapper(*id))
    }

    fn parse_template(
        &mut self,
        template: TemplateRule,
        extras: &Option<HashMap<String, usize>>,
    ) -> Result<MatcherType> {
        let mut params = Vec::with_capacity(template.names.len());
        for _ in 0..template.names.len() - 1 {
            self.consume_whitespace();
            let param = self.parse_list(extras)?;
            let id = self.add_matcher(param).id;
            params.push(id);
            self.assert_char(',')?;
            self.consume_whitespace();
        }
        let last = self.parse_list(extras)?;
        let last_id = self.add_matcher(last).id;
        params.push(last_id);
        self.consume_whitespace();
        self.assert_char('>')?;
        let new_extras: HashMap<_, _> = template.names.iter().cloned().zip(params).collect();
        let old_pos = self.pos;
        self.pos = template.rule_start;
        let parsed = self.parse_list(&Some(new_extras))?;
        self.pos = old_pos;
        Ok(parsed)
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

    fn parse_list(&mut self, extras: &Option<HashMap<String, usize>>) -> Result<MatcherType> {
        let mut matcher_list = Vec::new();
        let mut choices = Vec::new();
        while self.pos < self.source.len() && self.list_should_continue() {
            matcher_list.push(self.parse_matcher_with_modifiers(extras)?);
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
            Ok(MatcherType::Choice(choices, None))
        } else {
            Ok(self.maybe_list(matcher_list))
        }
    }

    fn create_error(&self, msg: String) -> FluxError {
        FluxError::new_dyn(msg, self.pos, Some(self.source.clone()))
    }
}
