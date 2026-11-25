//! Errors for the text -> Table stage.
//!
//! v1 is deliberately simple: a line number and a message.
//! REFACTOR (later): replace `message: String` with an `ErrorKind` enum so
//! callers can match on *what* went wrong, and add a column / span.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl ParseError {
    /// `impl Into<String>` lets callers pass "literal" or format!(...) alike.
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

// Display = how the error looks to a human. Required for std::error::Error.
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

// Marker impl: lets ParseError work with `Box<dyn Error>` and `?` in main.
impl std::error::Error for ParseError {}
