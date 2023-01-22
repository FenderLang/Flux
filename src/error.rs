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
    pub description: ErrorMessage,
    pub location: usize,
    pub depth: usize,
    pub matcher_name: MatcherName,
    pub backtrace: Backtrace,
}

impl FluxError {
    pub fn new(description: &'static str, location: usize) -> FluxError {
        FluxError {
            description: ErrorMessage::Constant(description),
            location,
            depth: 0,
            matcher_name: Rc::new(None),
            backtrace: Backtrace::capture(),
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
        }
    }

    pub fn new_dyn(description: String, location: usize) -> FluxError {
        FluxError {
            description: ErrorMessage::Dynamic(description),
            location,
            depth: 0,
            matcher_name: Rc::new(None),
            backtrace: Backtrace::capture(),
        }
    }

    pub fn max(self, b: FluxError) -> FluxError {
        match (&*self.matcher_name, &*b.matcher_name) {
            (None, None) | (Some(_), Some(_)) => {
                if self.location > b.location {
                    self
                } else if b.location > self.location {
                    b
                } else if b.depth < self.depth {
                    b
                } else {
                    self
                }
            }
            (None, Some(_)) => b,
            (Some(_), None) => self,
        }
    }

    pub fn generate_error_display(&self, source: String) -> String {
        let (line_start, mut line_num) = source
            .chars()
            .take(self.location)
            .enumerate()
            .filter(|(_, c)| *c == '\n')
            .map(|(i, _)| (i, 1))
            .reduce(|(a, b), (x, y)| (a.max(x), b + y))
            .unwrap_or((0, 0));
        let col = self.location - line_start;
        line_num += 1;
        let name = (&*self.matcher_name)
            .clone()
            .unwrap_or_else(|| "token".to_string());
        let mut output = String::from(format!(
            "error on line {line_num} col {col} (position {}):",
            self.location
        ));
        output.push_str("\n\n");
        output.push_str(
            source
                .lines()
                .nth(line_num - 1)
                .expect("line always exists"),
        );
        output.push_str("\n");
        output.push_str(&("-".repeat(col) + "^\n"));
        output.push_str(&format!(
            "{}{} {}",
            " ".repeat(col),
            self.description.get_message(),
            name
        ));
        output
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
