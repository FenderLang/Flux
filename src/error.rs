use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    ops::Deref,
    rc::Rc,
    backtrace::Backtrace,
};

use crate::matchers::MatcherName;

pub type Result<T> = std::result::Result<T, FluxError>;

pub enum ErrorMessage {
    Constant(&'static str),
    Dynamic(String),
}

impl ErrorMessage {
    fn get_message(&self) -> &str {
        match self {
            Self::Constant(s) => s,
            Self::Dynamic(s) => s,
        }
    }
}

pub struct FluxError {
    description: ErrorMessage,
    location: usize,
    matcher_name: MatcherName,
    backtrace: Backtrace,
}

impl FluxError {
    pub fn new(description: &'static str, location: usize) -> FluxError {
        FluxError {
            description: ErrorMessage::Constant(description),
            location,
            matcher_name: Rc::new(RefCell::new(None)),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn new_matcher(
        description: &'static str,
        location: usize,
        matcher_name: MatcherName,
    ) -> FluxError {
        FluxError {
            description: ErrorMessage::Constant(description),
            location,
            matcher_name,
            backtrace: Backtrace::capture(),
        }
    }

    pub fn new_dyn(description: String, location: usize) -> FluxError {
        FluxError {
            description: ErrorMessage::Dynamic(description),
            location,
            matcher_name: Rc::new(RefCell::new(None)),
            backtrace: Backtrace::capture(),
        }
    }
}

impl std::error::Error for FluxError {}

impl Debug for FluxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FluxError")
            .field("description", &self.description.get_message())
            .field("location", &self.location)
            .field("match_ref", &self.matcher_name)
            .field("backtrace", &self.backtrace)
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
            self.description.get_message()
        )
    }
}
