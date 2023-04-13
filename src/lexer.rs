use bumpalo::Bump;

use crate::error::Result;
use crate::matchers::{Matcher, MatcherType, TokenOutput};
use crate::tokens::Token;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

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
        let mut lexer = Lexer {
            root,
            retain_empty: false,
            names,
            matchers,
        };
        lexer.init_caches();
        lexer
    }

    fn init_caches(&mut self) {
        for i in 0..self.matchers.len() {
            match &self.matchers[i].matcher_type {
                MatcherType::Choice(children, None) => {
                    let cache = std::array::from_fn(|i| {
                        children
                            .iter()
                            .cloned()
                            .filter(|c| {
                                self.matchers[*c].can_start_with(i as u8 as char, &self.matchers)
                            })
                            .collect()
                    });
                    self.matchers[i].matcher_type =
                        MatcherType::Choice(children.clone(), Some(cache.into()));
                }
                MatcherType::Repeating(child, match_range, None) => {
                    let cache = std::array::from_fn(|i| {
                        self.matchers[*child].can_start_with(i as u8 as char, &self.matchers)
                    });
                    self.matchers[i].matcher_type =
                        MatcherType::Repeating(*child, match_range.clone(), Some(cache.into()));
                }
                _ => (),
            }
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

    pub fn check(&self, input: impl AsRef<str>) -> Result<()> {
        self.tokenize(input, |_| ())
    }

    pub fn check_with(&self, matcher: &str, input: impl AsRef<str>) -> Result<()> {
        self.tokenize_with(matcher, input, |_| ())
    }

    pub fn tokenize<T>(
        &self,
        input: impl AsRef<str>,
        processor: impl FnOnce(&mut Token) -> T,
    ) -> Result<T> {
        let root = &self.matchers[self.root];
        self.do_tokenize(root, input, processor)
    }

    pub fn tokenize_with<T>(
        &self,
        matcher: &str,
        input: impl AsRef<str>,
        processor: impl FnOnce(&mut Token) -> T,
    ) -> Result<T> {
        let matcher = &self.matchers[self.names[matcher]];
        self.do_tokenize(matcher, input, processor)
    }

    fn do_tokenize<T>(
        &self,
        root: &Matcher,
        input: impl AsRef<str>,
        processor: impl FnOnce(&mut Token) -> T,
    ) -> Result<T> {
        let input = input.as_ref();
        let source: Arc<[char]> = input.chars().collect();
        let pos = 0;
        let bump = Rc::new(Bump::with_capacity(10000));
        let mut output = TokenOutput {
            tokens: bumpalo::collections::Vec::new_in(&bump),
            last_success: Default::default(),
        };
        let range = root
            .apply(source.clone(), &mut output, &self.matchers, pos, 0, &bump)
            .ok_or_else(|| output.create_error(source.clone(), &self.matchers))?;
        if output.tokens.is_empty() || range.end != source.len() {
            Err(output.create_error(source, &self.matchers))
        } else {
            let mapped_value = processor(&mut output.tokens[0]);
            std::mem::forget(output);
            Ok(mapped_value)
        }
    }
}
