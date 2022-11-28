#![allow(dead_code)]
use crate::error::Result;
use crate::matchers::MatcherRef;
use crate::tokens::Token;
use std::collections::HashMap;
use std::ops::Deref;
use std::vec;

#[derive(Clone, Copy)]
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
    pub fn tokenize<'a>(&'a self, input: &'a [char]) -> Result<Token> {
        let pos = 0;
        let token = self.root.apply(input, pos)?;
        Ok(self.prune(token))
    }

    fn prune<'a>(&'a self, mut root: Token<'a>) -> Token<'a> {
        let mut new_root = Token {
            children: Vec::new(),
            ..root.clone()
        };

        for i in 0..root.children.len() {
            self.prune_rec(&mut root, i)
                .into_iter()
                .for_each(|t| new_root.children.push(t));
        }

        new_root
    }

    fn prune_rec<'a>(&'a self, parent: &mut Token<'a>, child_pos: usize) -> Vec<Token<'a>> {
        let mut tmp_children = Vec::new();

        for i in 0..(parent.children[child_pos]).children.len() {
            self.prune_rec(&mut parent.children[child_pos], i)
                .into_iter()
                .for_each(|t| tmp_children.push(t));
        }
        parent.children = tmp_children;

        if let (false, true) = (self.retain_empty, parent.children.is_empty()) {
            return Vec::new();
        }

        let cull_strat = match parent.children[child_pos].matcher_name.borrow().deref() {
            Some(name) => self.named_rules.get(name).clone(),
            None => Some(&self.unnamed_rule),
        };

        if let Some(cull_strat) = cull_strat {
            apply_cull_strat(*cull_strat, parent.children[child_pos].clone())
        } else {
            Vec::new()
        }
    }
}

fn apply_cull_strat(cull_strat: CullStrategy, mut token: Token) -> Vec<Token> {
    match cull_strat {
        CullStrategy::None => vec![token.clone()],
        CullStrategy::DeleteAll => Vec::new(),
        CullStrategy::DeleteChildren => {
            token.children = Vec::new();
            vec![token]
        }
        CullStrategy::LiftChildren => token.children,
    }
}
