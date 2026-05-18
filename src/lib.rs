//! # selinux_configfile
//!
//! 100% safe Rust parser and writer for `/etc/selinux/config`.
//!
//! - Type-safe getters/setters for all 5 standard SELinux config keys
//! - All write operations preserve original formatting (comments, whitespace,
//!   inline comments, blank lines)
//! - Atomic file writes (write temp + fsync + rename)
//! - Zero unsafe code
//!
//! ## Quick start
//!
//! ```rust,no_run,ignore
//! // NOTE: ConfigFile is implemented in Phase 4; this will be enabled then.
//! use selinux_configfile::{ConfigFile, SelinuxMode};
//!
//! let mut cfg = ConfigFile::read_default().unwrap();
//! assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
//! cfg.set_selinux(SelinuxMode::Permissive);
//! cfg.write_default().unwrap();
//! ```

pub mod config_file;
pub mod error;
pub mod parser;
pub mod types;

pub use config_file::ConfigFile;
pub use error::{IoError, ParseError, ValueError};
pub use types::{
    Line, SelinuxMode, AUTORELABEL_KEY, REQUIRESEUSERS_KEY, SELINUXTYPE_DEFAULT, SELINUXTYPE_KEY,
    SELINUX_KEY, SETLOCALDEFS_KEY,
};
