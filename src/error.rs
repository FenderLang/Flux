use std::{
    backtrace::Backtrace,
    fmt::{Debug, Display},
    rc::Rc,
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
    depth: usize,
    matcher_name: MatcherName,
    backtrace: Backtrace,
    losers: Vec<FluxError>,
}

impl FluxError {
    pub fn new(description: &'static str, location: usize) -> FluxError {
        FluxError {
            description: ErrorMessage::Constant(description),
            location,
            depth: 0,
            matcher_name: Rc::new(None),
            backtrace: Backtrace::capture(),
            losers: vec![],
        }
    }

    pub fn new_matcher(
        description: &'static str,
        location: usize,
        depth: usize,
        matcher_name: MatcherName,
    ) -> FluxError {
        FluxError {
            description: ErrorMessage::Constant(description),
            location,
            depth,
            matcher_name,
            backtrace: Backtrace::capture(),
            losers: vec![],
        }
    }

    pub fn new_dyn(description: String, location: usize) -> FluxError {
        FluxError {
            description: ErrorMessage::Dynamic(description),
            location,
            depth: 0,
            matcher_name: Rc::new(None),
            backtrace: Backtrace::capture(),
            losers: vec![],
        }
    }

    pub fn max(self, b: FluxError) -> FluxError {
        match (&*self.matcher_name, &*b.matcher_name) {
            (None, None) | (Some(_), Some(_)) => {
                if self.depth > b.depth {
                    self.add_loser(b)
                } else if b.depth > self.depth {
                    b.add_loser(self)
                } else if self.location > b.location {
                    self.add_loser(b)
                } else {
                    b.add_loser(self)
                }
            }
            (None, Some(_)) => b.add_loser(self),
            (Some(_), None) => self.add_loser(b),
        }
    }

    fn add_loser(mut self, b: FluxError) -> FluxError {
        self.losers.push(b);
        self
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
            match &*self.matcher_name {
                Some(m) => format!("matcher named `{}`", m),
                None => "no matcher".into(),
            },
            self.description.get_message()
        )
    }
}
