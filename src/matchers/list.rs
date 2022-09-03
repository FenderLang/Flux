use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{error::FluxError, tokens::Token};

use super::{Matcher, MatcherRef};

pub struct ListMatcher {
    name: Rc<String>,
    min_length: RefCell<Option<usize>>,
    children: Vec<RefCell<MatcherRef>>,
}

impl ListMatcher {
    pub fn new<S: ToString>(name: S, children: Vec<RefCell<MatcherRef>>) -> ListMatcher {
        ListMatcher {
            name: Rc::new(name.to_string()),
            min_length: RefCell::new(None),
            children,
        }
    }
}

impl Matcher for ListMatcher {
    fn apply(
        &self,
        source: std::rc::Rc<Vec<char>>,
        pos: usize,
    ) -> crate::error::Result<crate::tokens::Token> {
        let mut children: Vec<Token> = Vec::new();

        for child in self.children.iter() {
            match child.borrow().apply(source.clone(), pos) {
                Ok(child_token) => children.push(child_token),
                Err(_) => {
                    return Err(FluxError::new_matcher(
                        "failed in list matcher".into(),
                        pos,
                        self.name.deref().clone(),
                    ))
                } //TODO don't remember to fix later
            }
        }

        Ok(Token {
            range: (pos..children.iter().last().unwrap().range.end),
            children,
            matcher_name: self.name.clone(),
            source: source.clone(),
        })
    }

    fn min_length(&self) -> usize {
        if let Some(len) = *self.min_length.borrow() {
            len
        } else {
            let len = self
                .children
                .iter()
                .map(|child| child.borrow().min_length())
                .min()
                .unwrap_or_default();
            *self.min_length.borrow_mut() = Some(len);
            len
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn children(&self) -> Option<&super::MatcherChildren> {
        Some(&self.children)
    }

    fn is_placeholder(&self) -> bool {
        false
    }
}
