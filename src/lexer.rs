use crate::error::Result;
use crate::matchers::{Matcher, MatcherFuncArgs, TokenOutput};
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

    pub fn add_rule_for_names(
        &mut self,
        names: impl IntoIterator<Item = impl AsRef<str>>,
        rule: CullStrategy,
    ) {
        for matcher in names.into_iter().map(|n| self.names[n.as_ref()]) {
            self.matchers[matcher].cull_strategy = rule;
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
        let mut output = TokenOutput {
            tokens: vec![],
            last_success: Default::default(),
        };
        let range = root
            .apply(
                MatcherFuncArgs {
                    source: source.clone(),
                    pos,
                    depth: 0,
                    output: &mut output,
                },
                &self.matchers,
            )
            .ok_or_else(|| output.create_error(source.clone(), &self.matchers))?;
        if output.tokens.is_empty() || range.end != source.len() {
            Err(output.create_error(source, &self.matchers))
        } else {
            Ok(output.tokens.remove(0))
        }
    }
}
