use crate::matchers::Matcher;
use crate::error::{FluxError, Result};
use crate::tokens::Token;
use std::collections::HashMap;
use std::sync::Arc;
use std::vec;

#[derive(Clone, Copy, Debug)]
pub enum CullStrategy {
    /// Leave the token alone
    None,
    /// Delete the token and all of its children
    DeleteAll,
    /// Delete the children of the token
    DeleteChildren,
    /// Delete the token and replace it with its children in its parent
    LiftChildren,
    /// Delete the token and replace it with its children only if it has N or less children
    LiftAtMost(usize),
}

#[derive(Debug, Clone)]
pub struct Lexer {
    root: usize,
    retain_empty: bool,
    names: HashMap<String, usize>,
    matchers: Vec<Matcher>,
}

impl Lexer {
    pub fn new(root: usize, names: HashMap<String, usize>, matchers: Vec<Matcher>) -> Lexer {
        Lexer {
            root,
            retain_empty: false,
            names,
            matchers,
        }
    }

    pub fn set_retain_empty(&mut self, retain_empty: bool) {
        self.retain_empty = retain_empty;
    }

    pub fn set_unnamed_rule(&mut self, unnamed_rule: CullStrategy) {
        for matcher in &mut self.matchers {
            if matcher.name.is_none() {
                matcher.cull_strategy = unnamed_rule;
            }
        }
    }

    pub fn add_rule_for_names(&mut self, names: Vec<&str>, rule: CullStrategy) {
        for matcher in &mut self.matchers {
            if matcher.name.as_deref().map_or(false, |name| names.contains(&name)) {
                matcher.cull_strategy = rule;
            }
        }
    }

    pub fn tokenize(&self, input: impl AsRef<str>) -> Result<Token> {
        let root = &self.matchers[self.root];
        self.do_tokenize(root, input)
    }

    pub fn tokenize_with(&self, matcher: &str, input: impl AsRef<str>) -> Result<Token> {
        let matcher = &self.matchers[self.names[matcher]];
        self.do_tokenize(matcher, input)
    }

    fn do_tokenize(&self, root: &Matcher, input: impl AsRef<str>) -> Result<Token> {
        let input = input.as_ref();
        let source: Arc<[char]> = input.chars().collect();
        let pos = 0;
        let mut output = vec![];
        let range = root.apply(source.clone(), &mut output, &self.matchers, pos, 0)?;
        let token = (!output.is_empty()).then_some(output.remove(0));
        if range.len() < input.len() {
            if let Some(Token {failure: Some(err), ..}) = token {
                return Err(err);
            }
            Err(FluxError::new("unexpected", 0, Some(source)))
        } else {
            Ok(token.unwrap())
        }
    }
}