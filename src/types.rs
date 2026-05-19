use std::fmt;
use std::str::FromStr;
use crate::error::ValueError;

/// SELinux enforcement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelinuxMode {
    Enforcing,
    Permissive,
    Disabled,
}

impl fmt::Display for SelinuxMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelinuxMode::Enforcing => write!(f, "enforcing"),
            SelinuxMode::Permissive => write!(f, "permissive"),
            SelinuxMode::Disabled => write!(f, "disabled"),
        }
    }
}

impl FromStr for SelinuxMode {
    type Err = ValueError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "enforcing" => Ok(SelinuxMode::Enforcing),
            "permissive" => Ok(SelinuxMode::Permissive),
            "disabled" => Ok(SelinuxMode::Disabled),
            _ => Err(ValueError {
                key: String::from("SELINUX"),
                message: format!("invalid SELinux mode: '{}'", s),
            }),
        }
    }
}

/// Standard SELinux config key names.
pub const SELINUX_KEY: &str = "SELINUX";
pub const SELINUXTYPE_KEY: &str = "SELINUXTYPE";
pub const REQUIRESEUSERS_KEY: &str = "REQUIRESEUSERS";
pub const AUTORELABEL_KEY: &str = "AUTORELABEL";
pub const SETLOCALDEFS_KEY: &str = "SETLOCALDEFS";

/// Default SELinux policy type.
pub const SELINUXTYPE_DEFAULT: &str = "targeted";

/// One line in the config file, preserving original formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    /// Comment line (e.g., `# This is a comment\n`).
    Comment(String),
    /// Blank line or whitespace-only line (e.g., `\n` or `   \n`).
    Blank(String),
    /// Unrecognized line — kept as-is.
    Raw(String),
    /// Key-value entry with formatting metadata for lossless writes.
    Entry {
        /// Original key text (preserved case).
        key_raw: String,
        /// Logical value (whitespace and inline comments stripped).
        value: String,
        /// Text before the key (indentation whitespace).
        raw_leading: String,
        /// The `=` and surrounding whitespace (e.g., `" = "` or `"="`).
        raw_separator: String,
        /// Everything after the value to end-of-line (inline comments,
        /// trailing whitespace, newline).
        raw_suffix: String,
    },
}
