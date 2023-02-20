use self::token_iterator::Iter;
use crate::{error::FluxError, matchers::MatcherName};
use std::{fmt::Debug, ops::Range, sync::Arc};

pub mod token_iterator;

pub struct Token {
    pub matcher_name: MatcherName,
    pub matcher_id: usize,
    pub children: Vec<Token>,
    pub source: Arc<Vec<char>>,
    pub range: Range<usize>,
    pub failure: Option<FluxError>,
}

impl Token {
    /// Get the content the token is matching from the source.
    pub fn get_match(&self) -> String {
        self.source[self.range.clone()].iter().collect()
    }

    /// Return the name of the matcher that created the token.
    pub fn get_name(&self) -> &Option<String> {
        &self.matcher_name
    }

    /// Get the first child of the token
    pub fn first(&self) -> Option<&Token> {
        self.children.get(0)
    }

    /// Get an iterator over all children of token
    /// 
    /// Equivalent to calling `self.iter()`
    pub fn all_children(&self) -> Iter {
        Iter::new(self)
    }

    /// Get an iterator over all children of `self` with a given `name`
    pub fn children_named<'a, 'b: 'a>(&'a self, name: &'b str) -> impl Iterator<Item = &'a Token> + 'a {
        Iter::new(self)
            .filter(move |t| matches!(t.matcher_name.as_ref(), Some(n) if n == name))
    }

    /// Get an iterator over all children in `self`
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Token");

        debug.field("name", &*self.matcher_name);
        debug.field("match", &self.get_match());
        debug.field("range", &self.range);
        if !self.children.is_empty() {
            debug.field("children", &self.children);
            debug.finish_non_exhaustive()
        } else {
            debug.finish()
        }
    }
}

impl<'a> IntoIterator for &'a Token {
    type Item = &'a Token;

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
