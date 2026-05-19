//! Error types: [`ParseError`], [`ValueError`], and [`IoError`].

use std::fmt;
use std::path::PathBuf;

/// Parse error with line number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The 0-based line number where the error occurred.
    pub line: usize,
    /// A human-readable description of the problem.
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Value validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueError {
    /// The key whose value failed validation.
    pub key: String,
    /// A human-readable description of the validation failure.
    pub message: String,
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.message)
    }
}

impl std::error::Error for ValueError {}

/// IO error wrapper for file operations.
#[derive(Debug)]
pub struct IoError {
    /// The path that caused the error.
    pub path: PathBuf,
    /// The underlying IO error.
    pub source: std::io::Error,
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IO error on {}: {}", self.path.display(), self.source)
    }
}

impl std::error::Error for IoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
