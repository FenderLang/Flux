use std::{fmt::{Debug, Display}, rc::Rc};

pub type Result<T> = std::result::Result<T, FluxError>;

pub struct FluxError {
    description: &'static str,
    location: usize,
    matcher_name:  Option<Rc<String>>
}

impl FluxError {
    pub fn new(description: &'static str, location: usize) -> FluxError {
        FluxError {
            description,
            location,
            matcher_name: None,
        }
    }
    pub fn new_matcher(description: &'static str, location: usize, match_ref: Rc<String>) -> FluxError {
        FluxError {
            description,
            location,
            matcher_name: Some(match_ref),
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
            match &self.matcher_name {
                Some(m) => format!("matcher named `{}`", m),
                None => "no matcher".into(),
            },
            self.description
        )
    }
}
