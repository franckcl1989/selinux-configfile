//! Core types for SELinux config: [`SelinuxMode`], [`Line`], and key constants.

use crate::error::ValueError;
use std::fmt;
use std::str::FromStr;

/// SELinux enforcement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelinuxMode {
    /// SELinux policy is enforced; access denials are logged and blocked.
    Enforcing,
    /// SELinux policy is not enforced, but denials are logged.
    Permissive,
    /// No SELinux policy is loaded (deprecated; prefer `selinux=0` kernel flag).
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
/// Standard key name for [`SELINUXTYPE`](SELINUXTYPE_KEY).
pub const SELINUXTYPE_KEY: &str = "SELINUXTYPE";
/// Standard key name for [`REQUIRESEUSERS`](REQUIRESEUSERS_KEY).
pub const REQUIRESEUSERS_KEY: &str = "REQUIRESEUSERS";
/// Standard key name for [`AUTORELABEL`](AUTORELABEL_KEY).
pub const AUTORELABEL_KEY: &str = "AUTORELABEL";
/// Standard key name for [`SETLOCALDEFS`](SETLOCALDEFS_KEY).
pub const SETLOCALDEFS_KEY: &str = "SETLOCALDEFS";

/// Default SELinux policy type.
pub const SELINUXTYPE_DEFAULT: &str = "targeted";

/// One line in the config file, preserving original formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Line {
    /// A `#` comment line, preserved verbatim including leading whitespace
    /// and trailing newline (e.g., `"# This is a comment\n"`).
    Comment(String),
    /// A blank or whitespace-only line (e.g., `"\n"` or `"   \n"`).
    Blank(String),
    /// An unrecognized line that could not be parsed as a key-value entry
    /// (e.g., a malformed line with no `=` sign). Preserved verbatim.
    Raw(String),
    /// A parsed key-value entry. All formatting metadata is stored alongside
    /// the logical [`value`](Line::Entry::value) so that serialization can
    /// reconstruct the line exactly — only `value` is ever modified.
    Entry {
        /// Original key text, preserving the case as it appears in the file.
        key_raw: String,
        /// Logical value with trailing whitespace, control characters, and
        /// inline comments stripped.
        value: String,
        /// Text before the key (leading whitespace / indentation).
        raw_leading: String,
        /// The `=` sign and any whitespace immediately surrounding it
        /// (e.g., `"="`, `" = "`, `"  = "`).
        raw_separator: String,
        /// Everything after the logical value through end-of-line —
        /// inline comments, trailing whitespace, and the newline character.
        raw_suffix: String,
    },
}
