use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::matchers::MatcherName;

pub type Result<T> = std::result::Result<T, FluxError>;

#[derive(Debug, PartialEq, Eq)]
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

#[derive(PartialEq, Eq)]
pub struct FluxError {
    pub description: ErrorMessage,
    pub location: usize,
    pub depth: usize,
    pub matcher_name: MatcherName,
    pub src_text: Option<Rc<Vec<char>>>,
}

impl FluxError {
    pub fn new(
        description: &'static str,
        location: usize,
        src_text: Option<Rc<Vec<char>>>,
    ) -> FluxError {
        FluxError {
            description: ErrorMessage::Constant(description),
            location,
            depth: 0,
            matcher_name: Rc::new(None),
            src_text,
        }
    }

    pub fn new_matcher(
        description: &'static str,
        location: usize,
        depth: usize,
        matcher_name: MatcherName,
        src_text: Option<Rc<Vec<char>>>,
    ) -> FluxError {
        FluxError {
            description: ErrorMessage::Constant(description),
            location,
            depth,
            matcher_name,
            src_text,
        }
    }

    pub fn new_dyn(
        description: String,
        location: usize,
        src_text: Option<Rc<Vec<char>>>,
    ) -> FluxError {
        FluxError {
            description: ErrorMessage::Dynamic(description),
            location,
            depth: 0,
            matcher_name: Rc::new(None),
            src_text,
        }
    }
}

impl Ord for FluxError {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&*self.matcher_name, &*other.matcher_name) {
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            _ => {
                if self.location > other.location {
                    std::cmp::Ordering::Greater
                } else if other.location > self.location || other.depth < self.depth {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            }
        }
    }
}

impl PartialOrd for FluxError {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (&*self.matcher_name, &*other.matcher_name) {
            (Some(_), None) => Some(std::cmp::Ordering::Greater),
            (None, Some(_)) => Some(std::cmp::Ordering::Less),
            _ => {
                if self.location > other.location {
                    Some(std::cmp::Ordering::Greater)
                } else if other.location > self.location || other.depth < self.depth {
                    Some(std::cmp::Ordering::Less)
                } else {
                    Some(std::cmp::Ordering::Greater)
                }
            }
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
            .finish()
    }
}

impl Display for FluxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src_highlight = match self.src_text.clone() {
            Some(source) => {
                let (line_start, mut line_num) = source
                    .iter()
                    .take(self.location)
                    .enumerate()
                    .filter(|(_, c)| **c == '\n')
                    .map(|(i, _)| (i, 1))
                    .reduce(|(a, b), (x, y)| (a.max(x), b + y))
                    .unwrap_or((0, 0));
                let col = self.location - line_start;
                line_num += 1;

                let mut output = format!("at line {line_num} col {col}");
                output.push_str("\n\n");
                output.push_str(
                    source
                        .iter()
                        .collect::<String>()
                        .lines()
                        .nth(line_num - 1)
                        .unwrap_or_default(),
                );
                output.push('\n');

                let num_spaces = ((col as i32).max(1) - 6).max(0) as usize;
                let num_underscores = col.min(6) - 1;

                output.push_str(
                    &(" ".repeat(num_spaces)
                        + &"_".repeat(num_underscores)
                        + "^"
                        + &"_".repeat(num_underscores)),
                );

                output
            }
            None => format!("at position {}", self.location),
        };

        write!(
            f,
            "FluxError at {} with {}\n\ndescription: \"{}\"\n{src_highlight}",
            self.location,
            match &*self.matcher_name {
                Some(m) => format!("matcher named `{}`", m),
                None => "no matcher".into(),
            },
            self.description.get_message(),
        )
    }
}
