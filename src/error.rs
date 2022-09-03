use std::fmt::{Debug, Display};

pub type Result<T> = std::result::Result<T, FluxError>;

pub struct FluxError {
    description: String,
    location: usize,
    match_ref: Option<String>,
}

impl FluxError {
    pub fn new(description: String, location: usize) -> FluxError {
        FluxError {
            description,
            location,
            match_ref: None,
        }
    }
    pub fn new_matcher(description: String, location: usize, match_ref: String) -> FluxError {
        FluxError {
            description,
            location,
            match_ref: Some(match_ref),
        }
    }
}

impl std::error::Error for FluxError {}

impl Debug for FluxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FluxError")
            .field("description", &self.description)
            .field("location", &self.location)
            .field("match_ref", &self.match_ref)
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
                Some(m) => format!("matcher named `{}`", m),
                None => "no matcher".into(),
            },
            self.description
        )
    }
}
