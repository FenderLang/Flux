use crate::error::Result;
use crate::tokens::Token;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

pub(crate) type MatcherRef = Rc<dyn Matcher>;
pub(crate) type MatcherChildren = Vec<RefCell<MatcherRef>>;
pub(crate) type MatcherName = Rc<Option<String>>;

#[derive(Clone, Debug, Default)]
pub struct MatcherMeta {
    pub name: MatcherName,
    pub id: usize,
    pub priority: usize,
}

#[macro_export]
macro_rules! impl_meta {
    () => {
        fn with_meta(&self, meta: MatcherMeta) -> $crate::matchers::MatcherRef {
            Rc::new(Self {
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
    pub fn new(name: Option<String>, id: usize, priority: usize) -> MatcherMeta {
        MatcherMeta {
            name: Rc::new(name),
            id,
            priority,
        }
    }
}

pub trait Matcher: Debug {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token>;
    fn min_length(&self) -> usize;
    fn meta(&self) -> &MatcherMeta;
    fn with_meta(&self, meta: MatcherMeta) -> MatcherRef;

    fn children(&self) -> Option<&MatcherChildren> {
        None
    }

    fn is_placeholder(&self) -> bool {
        false
    }

    fn name(&self) -> &Rc<Option<String>> {
        &self.meta().name
    }

    fn id(&self) -> usize {
        self.meta().id
    }

    fn priority(&self) -> usize {
        self.meta().priority
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
