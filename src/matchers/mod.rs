use crate::error::Result;
use crate::tokens::Token;

use std::fmt::{Debug, Display};

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub(crate) type MatcherRef = Arc<dyn Matcher + Send + Sync>;
#[derive(Debug)]
pub(crate) struct MatcherChildren(RwLock<Vec<MatcherRef>>);
pub(crate) type MatcherName = Arc<Option<String>>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct MatcherMeta {
    pub name: MatcherName,
    pub id: usize,
}

#[macro_export]
macro_rules! impl_meta {
    () => {
        fn with_meta(&self, meta: MatcherMeta) -> $crate::matchers::MatcherRef {
            std::sync::Arc::new(Self {
                meta,
                ..self.clone()
            })
        }
        fn meta(&self) -> &MatcherMeta {
            &self.meta
        }
    };
}

impl MatcherMeta {
    pub fn new(name: Option<String>, id: usize) -> MatcherMeta {
        MatcherMeta {
            name: Arc::new(name),
            id,
        }
    }
}

pub trait Matcher: Debug {
    fn apply(&self, source: Arc<Vec<char>>, pos: usize, depth: usize) -> Result<Token>;
    fn meta(&self) -> &MatcherMeta;
    fn with_meta(&self, meta: MatcherMeta) -> MatcherRef;

    fn children<'a>(&'a self) -> Option<RwLockWriteGuard<'a, Vec<MatcherRef>>> {
        None
    }

    fn is_placeholder(&self) -> bool {
        false
    }

    fn name(&self) -> &Arc<Option<String>> {
        &self.meta().name
    }

    fn id(&self) -> usize {
        self.meta().id
    }

    fn next_depth(&self, depth: usize) -> usize {
        match **self.name() {
            Some(_) => depth + 1,
            None => depth,
        }
    }
}

impl Display for dyn Matcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl Clone for MatcherChildren {
    fn clone(&self) -> Self {
        Self(RwLock::new(self.0.write().unwrap().clone()))
    }
}

impl MatcherChildren {
    fn new(children: Vec<MatcherRef>) -> MatcherChildren {
        Self(RwLock::new(children))
    }

    pub fn get_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Vec<MatcherRef>> {
        self.0.write().unwrap()
    }

    pub fn get<'a>(&'a self) -> RwLockReadGuard<'a, Vec<MatcherRef>> {
        self.0.read().unwrap()
    }
}

pub mod char_range;
pub mod char_set;
pub mod choice;
pub mod eof;
pub mod inverted;
pub mod list;
pub mod placeholder;
pub mod repeating;
pub mod string;
