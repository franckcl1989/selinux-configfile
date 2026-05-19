//! # selinux-configfile
//!
//! 100% safe Rust parser and writer for `/etc/selinux/config` with full
//! format preservation.  Zero `unsafe` code.
//!
//! ## Features
//!
//! - **Type-safe getters/setters** for all 5 standard SELinux config keys:
//!   `SELINUX`, `SELINUXTYPE`, `REQUIRESEUSERS`, `AUTORELABEL`, `SETLOCALDEFS`
//! - **Generic key-value API** for unknown or custom keys
//! - **Format preservation** — comments, indentation, inline comments, blank
//!   lines, and trailing whitespace are all preserved across read–modify–write
//!   cycles
//! - **Atomic writes** — write via temp file + `fsync` + rename
//! - **Zero unsafe code** — verified with `grep -r unsafe src/`
//!
//! ## Quick start
//!
//! ```rust
//! use selinux_configfile::{ConfigFile, SelinuxMode};
//!
//! // Parse an in-memory config string
//! let mut cfg = ConfigFile::parse(
//!     "SELINUX=enforcing\nSELINUXTYPE=targeted\n"
//! ).unwrap();
//!
//! assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
//! assert_eq!(cfg.selinuxtype(), Some("targeted"));
//!
//! // Modify values
//! cfg.set_selinux(SelinuxMode::Permissive);
//! cfg.set_selinuxtype("mls").unwrap();
//!
//! // Serialize back to string
//! let output = cfg.to_string();
//! assert!(output.contains("SELINUX=permissive"));
//! assert!(output.contains("SELINUXTYPE=mls"));
//! ```
//!
//! ## Reading and writing files
//!
//! ```rust,no_run
//! use selinux_configfile::{ConfigFile, SelinuxMode};
//!
//! // Read from the default system path (/etc/selinux/config)
//! let mut cfg = ConfigFile::read_default().unwrap();
//!
//! // Modify and write back
//! cfg.set_selinux(SelinuxMode::Permissive);
//! cfg.write_default().unwrap();
//! ```
//!
//! ## Format preservation example
//!
//! ```rust
//! use selinux_configfile::ConfigFile;
//!
//! let input = "# My config\nSELINUX = enforcing  # inline comment\n";
//! let cfg = ConfigFile::parse(input).unwrap();
//! let output = cfg.to_string();
//! assert_eq!(input, output);
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod config_file;
pub mod error;
pub mod io;
pub mod parser;
pub mod serializer;
pub mod types;

pub use config_file::ConfigFile;
pub use error::{IoError, ParseError, ValueError};
pub use types::{
    AUTORELABEL_KEY, Line, REQUIRESEUSERS_KEY, SELINUX_KEY, SELINUXTYPE_DEFAULT, SELINUXTYPE_KEY,
    SETLOCALDEFS_KEY, SelinuxMode,
};
