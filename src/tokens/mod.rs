use crate::{error::FluxError, matchers::MatcherName};
use std::{fmt::Debug, ops::Range, sync::Arc};

use self::iterators::{rec_iter::RecursiveIter, iter::Iter};

pub mod iterators {
    pub mod ignore_iter;
    pub mod iter;
    pub mod rec_iter;
}

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

    /// Get an iterator over all children of `self`, recursively
    ///
    /// Equivalent to calling `self.rec_iter()`
    pub fn recursive_children(&self) -> RecursiveIter {
        RecursiveIter::new(self)
    }

    /// Get an iterator over all children of `self` with a given `name`, recursively
    pub fn recursive_children_named<'a, 'b: 'a>(
        &'a self,
        name: &'b str,
    ) -> impl Iterator<Item = &'a Token> + 'a {
        RecursiveIter::new(self)
            .filter(move |t| matches!(t.matcher_name.as_ref(), Some(n) if n == name))
    }

    /// Get an iterator over the direct children of `self` with a given name `name`
    pub fn children_named<'a, 'b: 'a>(&'a self, name: &'b str) -> impl Iterator<Item = &'a Token> {
        self.children
            .iter()
            .filter(move |t| matches!(t.matcher_name.as_ref(), Some(n) if n == name))
    }

    /// Get an iterator over children of `self`
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    /// Get an iterator over all children in `self`, recursively
    pub fn rec_iter(&self) -> RecursiveIter {
        RecursiveIter::new(self)
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

    type IntoIter = RecursiveIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.rec_iter()
    }
}
