use crate::error::{FluxError, Result};
use crate::matchers::MatcherRef;
use crate::tokens::Token;
use std::collections::HashMap;
use std::vec;

#[derive(Clone, Copy, Debug)]
pub enum CullStrategy {
    /// Leave the token alone
    None,
    /// Delete the token and all of its children
    DeleteAll,
    /// Delete the children of the token but not the token itself
    DeleteChildren,
    /// Delete the token and replace it with its children in its parent
    LiftChildren,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    root: MatcherRef,
    retain_empty: bool,
    unnamed_rule: CullStrategy,
    names: HashMap<String, usize>,
    named_rules: Vec<CullStrategy>,
}

impl Lexer {
    pub fn new(root: MatcherRef, names: HashMap<String, usize>) -> Lexer {
        Lexer {
            root,
            retain_empty: false,
            unnamed_rule: CullStrategy::LiftChildren,
            named_rules: vec![CullStrategy::None; names.len() + 1],
            names,
        }
    }

    pub fn set_retain_empty(&mut self, retain_empty: bool) {
        self.retain_empty = retain_empty;
    }

    pub fn set_unnamed_rule(&mut self, unnamed_rule: CullStrategy) {
        self.unnamed_rule = unnamed_rule;
    }

    pub fn add_rule_for_names(&mut self, names: Vec<String>, rule: CullStrategy) {
        for name in names.into_iter() {
            if let Some(id) = self.names.get(&name) {
                self.named_rules[*id] = rule;
            }
        }
    }

    pub fn tokenize<'a>(&'a self, input: &'a [char]) -> Result<Token> {
        let pos = 0;
        let token = self.root.apply(input, pos)?;
        if token.range.len() < input.len() {
            Err(FluxError::new("unexpected token", token.range.end))
        } else {
            Ok(self.prune(token))
        }
    }

    fn get_cull_strat(&self, token: &Token) -> CullStrategy {
        if token.matcher_name.borrow().is_none() {
            return self.unnamed_rule;
        }
        if token.range.is_empty() && !self.retain_empty {
            return CullStrategy::DeleteAll;
        }
        self.named_rules[token.matcher_id]
    }

    fn prune<'a>(&'a self, mut parent: Token<'a>) -> Token<'a> {
        let mut tmp_children = Vec::new();
        for child in parent.children {
            let child = self.prune(child);
            let strat = self.get_cull_strat(&child);
            tmp_children.extend(apply_cull_strat(strat, child));
        }
        parent.children = tmp_children;
        parent
    }
}

fn apply_cull_strat(cull_strat: CullStrategy, mut token: Token) -> Vec<Token> {
    match cull_strat {
        CullStrategy::None => vec![token],
        CullStrategy::DeleteAll => Vec::new(),
        CullStrategy::DeleteChildren => {
            token.children = Vec::new();
            vec![token]
        }
        CullStrategy::LiftChildren => token.children,
    }
}
