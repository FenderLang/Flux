use crate::error::Result;
use crate::tokens::Token;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::sync::Arc;

pub(crate) type MatcherRef = Arc<dyn Matcher>;
pub(crate) type MatcherChildren = Vec<RefCell<MatcherRef>>;
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
    fn min_length(&self) -> usize;
    fn meta(&self) -> &MatcherMeta;
    fn with_meta(&self, meta: MatcherMeta) -> MatcherRef;

    fn children(&self) -> Option<&MatcherChildren> {
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

pub mod char_range;
pub mod char_set;
pub mod choice;
pub mod eof;
pub mod inverted;
pub mod list;
pub mod placeholder;
pub mod repeating;
pub mod string;
