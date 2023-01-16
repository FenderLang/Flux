use crate::error::Result;
use crate::tokens::Token;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::rc::Rc;

pub type MatcherRef = Rc<dyn Matcher>;
pub type MatcherChildren = Vec<RefCell<MatcherRef>>;
pub type MatcherName = Rc<Option<String>>;

#[derive(Clone, Debug)]
pub(crate) struct MatcherMeta {
    pub name: MatcherName,
    pub id: usize,
}

#[macro_export]
macro_rules! with_meta {
    () => {
        fn with_meta(&self, meta: MatcherMeta) -> crate::matchers::MatcherRef {
            Rc::new(Self {
                meta,
                ..*self
            })
        }
    }
}

impl MatcherMeta {
    pub fn new(name: Option<String>, id: usize) -> MatcherMeta {
        MatcherMeta { name: Rc::new(name), id }
    }
}

impl Default for MatcherMeta {
    fn default() -> Self {
        MatcherMeta { name: Rc::new(None), id: 0 }
    }
}

pub trait Matcher: Debug {
    fn apply(&self, source: Rc<Vec<char>>, pos: usize) -> Result<Token>;
    fn min_length(&self) -> usize;
    fn meta(&self) -> &MatcherMeta;

    fn children(&self) -> Option<&MatcherChildren> {
        None
    }

    fn is_placeholder(&self) -> bool {
        false
    }

    fn get_name(&self) -> &Rc<Option<String>> {
        &self.meta().name
    }

    fn id(&self) -> usize {
        self.meta().id
    }

    fn with_meta(&self, meta: MatcherMeta) -> MatcherRef;

    fn reset_meta(&self) -> MatcherRef {
        self.with_meta(Default::default())
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
