use std::{
    fmt::{Debug, Display},
};

use crate::matchers::MatcherRef;

pub type Result<T> = std::result::Result<T, FluxError>;

pub struct FluxError {
    description: String,
    location: usize,
    match_ref: Option<MatcherRef>,
}

impl std::error::Error for FluxError {}

impl Debug for FluxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FluxError")
            .field("description", &self.description)
            .field("location", &self.location)
            .field("match_ref", &self.match_ref.clone().map(|m| m.name()))
            .finish()
    }
}

impl Display for FluxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FluxError at {} with {} description \"{}\"",
            self.location,
            match &self.match_ref {
                Some(m) => format!("matcher named `{}`", m.name()),
                None => "no matcher".into(),
            },
            self.description
        )
    }
}
