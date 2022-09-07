use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    ops::Deref,
    rc::Rc,
};

use crate::matchers::MatcherName;

pub type Result<T> = std::result::Result<T, FluxError>;

pub struct FluxError {
    description: &'static str,
    location: usize,
    matcher_name: MatcherName,
}

impl FluxError {
    pub fn new(description: &'static str, location: usize) -> FluxError {
        FluxError {
            description,
            location,
            matcher_name: Rc::new(RefCell::new(None)),
        }
    }
    pub fn new_matcher(
        description: &'static str,
        location: usize,
        matcher_name: MatcherName,
    ) -> FluxError {
        FluxError {
            description,
            location,
            matcher_name,
        }
    }
}

impl std::error::Error for FluxError {}

impl Debug for FluxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FluxError")
            .field("description", &self.description)
            .field("location", &self.location)
            .field("match_ref", &self.matcher_name)
            .finish()
    }
}

impl Display for FluxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FluxError at {} with {} description \"{}\"",
            self.location,
            match &self.matcher_name.as_ref().borrow().deref() {
                Some(m) => format!("matcher named `{}`", m),
                None => "no matcher".into(),
            },
            self.description
        )
    }
}
