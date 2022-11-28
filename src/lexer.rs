#![allow(dead_code)]
use crate::error::Result;
use crate::matchers::MatcherRef;
use crate::tokens::Token;
use std::collections::HashMap;

enum CullStrategy {
    /// Leave the token alone
    None,
    /// Delete the token and all of its children
    DeleteAll,
    /// Delete the children of the token but not the token itself
    DeleteChildren,
    /// Delete the token and replace it with its children in its parent
    LiftChildren,
}

struct Lexer {
    root: MatcherRef,
    retain_empty: bool,
    retain_literal: bool,
    unnamed_rule: CullStrategy,
    named_rules: HashMap<String, CullStrategy>,
}

impl Lexer {
    pub fn tokenize(&self, input: &str) -> Result<Token> {
        todo!()
    }

    fn prune(root: Token) -> Token {
        todo!()
    }
}
