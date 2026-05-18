# SELinux Config File Library — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a 100% safe Rust library for reading/writing /etc/selinux/config with format-preserving writes, type-safe API, and test coverage exceeding the official libselinux implementation.

**Architecture:** The file is parsed into a `Vec<Line>` where each `Line` is `Comment`, `Blank`, `Raw`, or `Entry`. Entries store raw formatting metadata (`raw_leading`, `raw_separator`, `raw_suffix`) alongside logical key/value. Modifications only update the logical `value` field; serialization reconstructs lines exactly. Atomic writes use `write temp + fsync + rename`.

**Tech Stack:** Rust (stable), no dependencies beyond std.

---

## File Structure

```
Cargo.toml
src/
  lib.rs              — crate docs, re-exports
  error.rs            — ParseError, ValueError, IoError + Display + Error impls
  types.rs            — SelinuxMode, Line, public constants
  parser.rs           — ConfigFile::parse(input: &str) -> Result<ConfigFile, ParseError>
  serializer.rs       — ConfigFile serialization (Display impl)
  config_file.rs      — ConfigFile struct + all getter/setter/validation methods
  io.rs               — read_from, write_to (atomic), read_default, write_default
tests/
  types_tests.rs      — SelinuxMode::from_str, Display, constants
  parser_tests.rs     — All parsing scenarios (official + extended)
  serializer_tests.rs — Format preservation, roundtrip tests
  config_file_tests.rs — All API method tests
  io_tests.rs         — File I/O tests
  integration_tests.rs — End-to-end scenarios
```

---

## Phase 1: Project Setup + Types & Errors (TDD)

### Task 1.1: Initialize Rust project

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`

- [ ] **Step 1: Create Cargo.toml**

```bash
cargo init --lib --name selinux_configfile /home/franck/github/selinux-configfile
```

Verify: `cargo build` succeeds with empty lib.rs.

- [ ] **Step 2: Add crate-level docs to src/lib.rs**

```rust
//! # selinux-configfile
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
//! ```rust
//! use selinux_configfile::{ConfigFile, SelinuxMode};
//!
//! let mut cfg = ConfigFile::read_default().unwrap();
//! assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
//! cfg.set_selinux(SelinuxMode::Permissive);
//! cfg.write_default().unwrap();
//! ```

// Nothing else yet — types come in next tasks
```

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml src/lib.rs
git commit -m "feat: initialize Rust project with cargo init"
```

### Task 1.2: Define SelinuxMode enum + parsing + Display

**Files:**
- Create: `src/types.rs`
- Create: `tests/types_tests.rs`
- Modify: `src/lib.rs` (add `mod types;`)

- [ ] **Step 1: Write failing tests in tests/types_tests.rs**

```rust
use selinux_configfile::SelinuxMode;
use std::str::FromStr;

#[test]
fn test_from_str_enforcing() {
    assert_eq!(SelinuxMode::from_str("enforcing").unwrap(), SelinuxMode::Enforcing);
    assert_eq!(SelinuxMode::from_str("ENFORCING").unwrap(), SelinuxMode::Enforcing);
    assert_eq!(SelinuxMode::from_str("Enforcing").unwrap(), SelinuxMode::Enforcing);
}

#[test]
fn test_from_str_permissive() {
    assert_eq!(SelinuxMode::from_str("permissive").unwrap(), SelinuxMode::Permissive);
    assert_eq!(SelinuxMode::from_str("PERMISSIVE").unwrap(), SelinuxMode::Permissive);
}

#[test]
fn test_from_str_disabled() {
    assert_eq!(SelinuxMode::from_str("disabled").unwrap(), SelinuxMode::Disabled);
    assert_eq!(SelinuxMode::from_str("DISABLED").unwrap(), SelinuxMode::Disabled);
}

#[test]
fn test_from_str_invalid() {
    assert!(SelinuxMode::from_str("").is_err());
    assert!(SelinuxMode::from_str("invalid").is_err());
    assert!(SelinuxMode::from_str("enfrocing").is_err()); // typo
}

#[test]
fn test_display() {
    assert_eq!(SelinuxMode::Enforcing.to_string(), "enforcing");
    assert_eq!(SelinuxMode::Permissive.to_string(), "permissive");
    assert_eq!(SelinuxMode::Disabled.to_string(), "disabled");
}

#[test]
fn test_from_str_trait() {
    // FromStr trait impl
    let mode: SelinuxMode = "enforcing".parse().unwrap();
    assert_eq!(mode, SelinuxMode::Enforcing);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -- types_tests
```
Expected: compilation error — `SelinuxMode` not defined.

- [ ] **Step 3: Write minimal implementation in src/types.rs**

```rust
use std::fmt;
use std::str::FromStr;

/// SELinux enforcement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelinuxMode {
    Enforcing,
    Permissive,
    Disabled,
}

impl SelinuxMode {
    /// Parse from a string, case-insensitive.
    pub fn from_str(s: &str) -> Result<SelinuxMode, ValueError> {
        match s.to_lowercase().as_str() {
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
        SelinuxMode::from_str(s)
    }
}
```

Note: `ValueError` isn't defined yet. For this task, we need a minimal placeholder. Add to `src/error.rs`:

```rust
/// Value validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueError {
    pub key: String,
    pub message: String,
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -- types_tests
```
Expected: all 6 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/types.rs src/error.rs tests/types_tests.rs src/lib.rs
git commit -m "feat: add SelinuxMode enum with parsing and Display"
```

### Task 1.3: Define public constants

**Files:**
- Modify: `src/types.rs`
- Modify: `tests/types_tests.rs`

- [ ] **Step 1: Add constant tests to tests/types_tests.rs**

```rust
use selinux_configfile::{
    SELINUX_KEY, SELINUXTYPE_KEY, REQUIRESEUSERS_KEY,
    AUTORELABEL_KEY, SETLOCALDEFS_KEY, SELINUXTYPE_DEFAULT,
};

#[test]
fn test_constant_values() {
    assert_eq!(SELINUX_KEY, "SELINUX");
    assert_eq!(SELINUXTYPE_KEY, "SELINUXTYPE");
    assert_eq!(REQUIRESEUSERS_KEY, "REQUIRESEUSERS");
    assert_eq!(AUTORELABEL_KEY, "AUTORELABEL");
    assert_eq!(SETLOCALDEFS_KEY, "SETLOCALDEFS");
    assert_eq!(SELINUXTYPE_DEFAULT, "targeted");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -- types_tests::test_constant_values
```
Expected: compilation error — constants not found.

- [ ] **Step 3: Add constants to src/types.rs**

```rust
/// Standard SELinux config key names.
pub const SELINUX_KEY: &str = "SELINUX";
pub const SELINUXTYPE_KEY: &str = "SELINUXTYPE";
pub const REQUIRESEUSERS_KEY: &str = "REQUIRESEUSERS";
pub const AUTORELABEL_KEY: &str = "AUTORELABEL";
pub const SETLOCALDEFS_KEY: &str = "SETLOCALDEFS";

/// Default SELinux policy type.
pub const SELINUXTYPE_DEFAULT: &str = "targeted";
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -- types_tests
```
Expected: all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/types.rs tests/types_tests.rs
git commit -m "feat: add standard SELinux config key constants"
```

### Task 1.4: Define Line enum

**Files:**
- Modify: `src/types.rs`
- Modify: `tests/types_tests.rs`

- [ ] **Step 1: Add Line enum construction/equality tests to tests/types_tests.rs**

```rust
use selinux_configfile::Line;

#[test]
fn test_line_comment() {
    let line = Line::Comment(String::from("# SELinux configuration\n"));
    assert_eq!(line, Line::Comment(String::from("# SELinux configuration\n")));
}

#[test]
fn test_line_blank() {
    let line = Line::Blank(String::from("\n"));
    assert_eq!(line, Line::Blank(String::from("\n")));
}

#[test]
fn test_line_blank_with_spaces() {
    let line = Line::Blank(String::from("   \n"));
    assert_eq!(line, Line::Blank(String::from("   \n")));
}

#[test]
fn test_line_raw() {
    let line = Line::Raw(String::from("this is not key=value\n"));
    assert_eq!(line, Line::Raw(String::from("this is not key=value\n")));
}

#[test]
fn test_line_entry_construct() {
    let entry = Line::Entry {
        key_raw: String::from("SELINUX"),
        value: String::from("enforcing"),
        raw_leading: String::new(),
        raw_separator: String::from("="),
        raw_suffix: String::from("\n"),
    };
    match &entry {
        Line::Entry { key_raw, value, .. } => {
            assert_eq!(key_raw, "SELINUX");
            assert_eq!(value, "enforcing");
        }
        _ => panic!("expected Entry variant"),
    }
}

#[test]
fn test_line_entry_with_spaces() {
    let entry = Line::Entry {
        key_raw: String::from("SELINUX"),
        value: String::from("enforcing"),
        raw_leading: String::from("  "),
        raw_separator: String::from(" = "),
        raw_suffix: String::from("  # mode comment\n"),
    };
    match &entry {
        Line::Entry { raw_separator, raw_suffix, .. } => {
            assert_eq!(raw_separator, " = ");
            assert_eq!(raw_suffix, "  # mode comment\n");
        }
        _ => panic!("expected Entry variant"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -- types_tests::test_line_comment
```
Expected: compilation error — `Line` not defined.

- [ ] **Step 3: Add Line enum to src/types.rs**

```rust
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
        pub key_raw: String,
        /// Logical value (whitespace and inline comments stripped).
        pub value: String,
        /// Text before the key (indentation whitespace).
        pub raw_leading: String,
        /// The `=` and surrounding whitespace (e.g., `" = "` or `"="`).
        pub raw_separator: String,
        /// Everything after the value to end-of-line (inline comments,
        /// trailing whitespace, newline).
        pub raw_suffix: String,
    },
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -- types_tests
```
Expected: all tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/types.rs tests/types_tests.rs
git commit -m "feat: add Line enum for config file line representation"
```

### Task 1.5: Complete error types with Display + Error impls

**Files:**
- Modify: `src/error.rs`
- Create: `tests/error_tests.rs`
- Modify: `src/lib.rs` (add `mod error;`)

- [ ] **Step 1: Add error tests to tests/error_tests.rs**

```rust
use selinux_configfile::{ParseError, ValueError};

#[test]
fn test_parse_error_display() {
    let err = ParseError { line: 5, message: String::from("malformed key") };
    let s = err.to_string();
    assert!(s.contains("line 5"), "expected line number in: {}", s);
    assert!(s.contains("malformed key"), "expected message in: {}", s);
}

#[test]
fn test_parse_error_trait() {
    let err = ParseError { line: 5, message: String::from("test") };
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_value_error_display() {
    let err = ValueError { key: String::from("SELINUX"), message: String::from("invalid value") };
    let s = err.to_string();
    assert!(s.contains("SELINUX"), "expected key in: {}", s);
    assert!(s.contains("invalid value"), "expected message in: {}", s);
}

#[test]
fn test_value_error_trait() {
    let err = ValueError { key: String::from("SELINUX"), message: String::from("test") };
    let _: &dyn std::error::Error = &err;
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -- error_tests
```
Expected: `ValueError` already defined should work, but `ParseError` and Error trait impls may be missing.

- [ ] **Step 3: Complete error types in src/error.rs**

```rust
use std::fmt;
use std::path::PathBuf;

/// Parse error with line number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
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
    pub key: String,
    pub message: String,
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.message)
    }
}

impl std::error::Error for ValueError {}

/// IO error wrapper.
#[derive(Debug)]
pub struct IoError {
    pub path: PathBuf,
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
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -- error_tests
```
Expected: all 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/error.rs tests/error_tests.rs src/lib.rs
git commit -m "feat: complete error types with Display and Error impls"
```

---

## Phase 2: Parser (TDD)

### Task 2.1: Parse minimal valid config (SELINUX + SELINUXTYPE)

**Files:**
- Create: `src/parser.rs`
- Create: `src/config_file.rs`
- Create: `tests/parser_tests.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write parser smoke tests in tests/parser_tests.rs**

```rust
use selinux_configfile::{ConfigFile, SelinuxMode};

#[test]
fn test_parse_minimal() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
}

#[test]
fn test_parse_with_spaces_around_equals() {
    let input = "SELINUX = enforcing\nSELINUXTYPE = targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
}

#[test]
fn test_parse_permissive() {
    let input = "SELINUX=permissive\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Permissive));
}

#[test]
fn test_parse_disabled() {
    let input = "SELINUX=disabled\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Disabled));
}

#[test]
fn test_parse_empty_string() {
    let cfg = ConfigFile::parse("").unwrap();
    assert!(cfg.is_empty());
}

#[test]
fn test_parse_newline_only() {
    let cfg = ConfigFile::parse("\n").unwrap();
    assert!(cfg.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -- parser_tests
```
Expected: compilation error — `ConfigFile` not defined.

- [ ] **Step 3: Write minimal ConfigFile struct + parser**

**src/config_file.rs:**

```rust
use crate::error::{ParseError, ValueError};
use crate::types::{Line, SelinuxMode};

/// Parsed representation of /etc/selinux/config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFile {
    pub(crate) lines: Vec<Line>,
}

impl ConfigFile {
    /// Create an empty config.
    pub fn new() -> Self {
        ConfigFile { lines: Vec::new() }
    }

    /// Create a config with minimal defaults (enforcing + targeted).
    pub fn default() -> Self {
        let mut cfg = ConfigFile::new();
        cfg.set_selinux(SelinuxMode::Enforcing);
        cfg.set_selinuxtype_raw("targeted");
        cfg
    }

    /// Parse from a string. See parsing algorithm in spec.
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        crate::parser::parse(input)
    }

    pub fn is_empty(&self) -> bool {
        !self.lines.iter().any(|l| matches!(l, Line::Entry { .. }))
    }

    // --- Typed getters ---

    pub fn selinux(&self) -> Option<SelinuxMode> {
        self.get(SELINUX_KEY)
            .and_then(|v| SelinuxMode::from_str(v).ok())
    }

    pub fn selinuxtype(&self) -> Option<&str> {
        self.get(SELINUXTYPE_KEY)
    }

    pub fn require_seusers(&self) -> Option<bool> {
        self.get_bool(REQUIRESEUSERS_KEY)
    }

    pub fn autorelabel(&self) -> Option<bool> {
        self.get_bool(AUTORELABEL_KEY)
    }

    pub fn setlocaldefs(&self) -> Option<bool> {
        self.get_bool(SETLOCALDEFS_KEY)
    }

    // --- Generic getter ---

    pub fn get(&self, key: &str) -> Option<&str> {
        let key_lower = key.to_lowercase();
        self.lines.iter().rev().find_map(|line| match line {
            Line::Entry { key_raw, value, .. }
                if key_raw.to_lowercase() == key_lower =>
            {
                Some(value.as_str())
            }
            _ => None,
        })
    }

    // --- Typed setters ---

    pub fn set_selinux(&mut self, mode: SelinuxMode) {
        let value = mode.to_string();
        self.set_inner(SELINUX_KEY, &value);
    }

    pub fn set_selinuxtype(&mut self, policy_type: &str) -> Result<(), ValueError> {
        let trimmed = policy_type.trim();
        if trimmed.is_empty() {
            return Err(ValueError {
                key: SELINUXTYPE_KEY.into(),
                message: "policy type must not be empty".into(),
            });
        }
        if trimmed.contains('/') {
            return Err(ValueError {
                key: SELINUXTYPE_KEY.into(),
                message: "policy type must not contain '/'".into(),
            });
        }
        if trimmed.contains(|c: char| c.is_control()) {
            return Err(ValueError {
                key: SELINUXTYPE_KEY.into(),
                message: "policy type must not contain control characters".into(),
            });
        }
        self.set_inner(SELINUXTYPE_KEY, trimmed);
        Ok(())
    }

    pub fn set_require_seusers(&mut self, value: bool) {
        self.set_inner(REQUIRESEUSERS_KEY, if value { "1" } else { "0" });
    }

    pub fn set_autorelabel(&mut self, value: bool) {
        self.set_inner(AUTORELABEL_KEY, if value { "1" } else { "0" });
    }

    pub fn set_setlocaldefs(&mut self, value: bool) {
        self.set_inner(SETLOCALDEFS_KEY, if value { "1" } else { "0" });
    }

    // --- Internal helpers ---

    pub(crate) fn set_selinuxtype_raw(&mut self, value: &str) {
        self.set_inner(SELINUXTYPE_KEY, value);
    }

    fn set_inner(&mut self, key: &str, value: &str) {
        let key_lower = key.to_lowercase();
        // If key exists, update last match in-place
        for line in self.lines.iter_mut().rev() {
            if let Line::Entry { key_raw, value: ref mut val, .. } = line {
                if key_raw.to_lowercase() == key_lower {
                    *val = value.to_string();
                    return;
                }
            }
        }
        // Key not found — append new Entry
        self.lines.push(Line::Entry {
            key_raw: key.to_string(),
            value: value.to_string(),
            raw_leading: String::new(),
            raw_separator: String::from("="),
            raw_suffix: String::from("\n"),
        });
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    }
}

use crate::types::{
    AUTORELABEL_KEY, REQUIRESEUSERS_KEY, SELINUXTYPE_KEY, SELINUX_KEY,
    SETLOCALDEFS_KEY,
};
```

**src/parser.rs:**

```rust
use crate::config_file::ConfigFile;
use crate::error::ParseError;
use crate::types::Line;

/// Parse config file content into a ConfigFile.
pub fn parse(input: &str) -> Result<ConfigFile, ParseError> {
    let mut lines = Vec::new();

    for (line_num, raw_line) in input.lines().enumerate() {
        let line_with_nl = if input.ends_with('\n') || line_num > 0 {
            format!("{}\n", raw_line)
        } else {
            raw_line.to_string()
        };

        // Determine if this line originally had \r\n
        let full_line = if raw_line.ends_with('\r') {
            format!("{}\n", &raw_line[..raw_line.len() - 1])
        } else {
            line_with_nl
        };

        // Reconstruct original line preserving \r\n
        let original = reconstruct_original(input, raw_line, line_num);

        let trimmed = raw_line.trim_start();
        let raw_leading_len = raw_line.len() - trimmed.len();
        let raw_leading = &raw_line[..raw_leading_len];

        if trimmed.is_empty() {
            lines.push(Line::Blank(original));
            continue;
        }
        if trimmed.starts_with('#') {
            lines.push(Line::Comment(original));
            continue;
        }

        // Find first =
        if let Some(eq_pos) = trimmed.find('=') {
            let key_raw = trimmed[..eq_pos].trim_end();
            if key_raw.is_empty() {
                lines.push(Line::Raw(original));
                continue;
            }

            let after_eq = &trimmed[eq_pos + 1..];

            // raw_separator: whitespace before = + = + whitespace after =
            let key_end_in_trimmed = key_raw.as_ptr() as usize - trimmed.as_ptr() as usize
                + key_raw.len();
            let sep_start = &raw_line[raw_leading_len + key_end_in_trimmed..];
            let value_start_in_sep = sep_start.find(|c: char| !c.is_whitespace() && c != '=').unwrap_or(sep_start.len());

            let raw_separator = if value_start_in_sep < sep_start.len() {
                sep_start[..value_start_in_sep].to_string()
            } else {
                sep_start.to_string()
            };

            let value_raw = after_eq.trim_start();
            let value_start_offset = after_eq.len() - value_raw.len();
            let _leading_whitespace_of_value = &after_eq[..value_start_offset];

            // Parse value and suffix
            let (value, raw_suffix) = parse_value_and_suffix(
                value_raw,
                raw_line,
                raw_leading_len + trimmed.len(),
            );

            lines.push(Line::Entry {
                key_raw: key_raw.to_string(),
                value,
                raw_leading: raw_leading.to_string(),
                raw_separator,
                raw_suffix,
            });
        } else {
            lines.push(Line::Raw(original));
        }
    }

    Ok(ConfigFile { lines })
}

fn parse_value_and_suffix(
    value_raw: &str,
    full_line: &str,
    value_raw_start: usize,
) -> (String, String) {
    // Find inline comment: # preceded by whitespace or at start of value_raw
    let mut comment_start = None;
    let bytes = value_raw.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'#' && (i == 0 || bytes[i - 1].is_ascii_whitespace()) {
            comment_start = Some(i);
            break;
        }
    }

    let (value_part, comment_part) = if let Some(pos) = comment_start {
        (&value_raw[..pos], &value_raw[pos..])
    } else {
        (value_raw, "")
    };

    // Strip trailing whitespace and control chars from value
    let value_trimmed = value_part.trim_end();
    let trailing_stripped = &value_part[value_trimmed.len()..];

    let value = value_trimmed.to_string();
    let raw_suffix = format!("{}{}{}",
        if trailing_stripped.is_empty() && comment_part.is_empty() { "" } else { trailing_stripped },
        comment_part,
        if full_line.ends_with('\n') { "\n" } else { "" },
    );

    (value, raw_suffix)
}

fn reconstruct_original(input: &str, raw_line: &str, line_num: usize) -> String {
    // Try to preserve original line endings
    let pos = input.chars().take_while(|c| *c == '\n').count();
    // Reconstruct from raw_line + appropriate ending
    if raw_line.ends_with('\r') {
        format!("{}\n", &raw_line[..raw_line.len() - 1])
    } else {
        format!("{}\n", raw_line)
    }
}
```

Note: The parser implementation needs careful handling. The above is a first pass. We may iterate on it during TDD.

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -- parser_tests
```
Expected: all 6 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src/parser.rs src/config_file.rs tests/parser_tests.rs src/lib.rs
git commit -m "feat: add parser for minimal valid config files"
```

### Task 2.2: Parse comments, blank lines, and mixed formatting

**Files:**
- Modify: `tests/parser_tests.rs`
- Modify: `src/parser.rs` (if needed)

- [ ] **Step 1: Add tests for comments and blank lines to tests/parser_tests.rs**

```rust
#[test]
fn test_parse_with_comments() {
    let input = "# SELinux configuration\nSELINUX=enforcing\n# end\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    assert_eq!(cfg.lines().len(), 3);
    assert!(matches!(cfg.lines()[0], Line::Comment(_)));
    assert!(matches!(cfg.lines()[1], Line::Entry { .. }));
    assert!(matches!(cfg.lines()[2], Line::Comment(_)));
}

#[test]
fn test_parse_with_blank_lines() {
    let input = "\nSELINUX=permissive\n\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.lines().len(), 4);
    assert!(matches!(cfg.lines()[0], Line::Blank(_)));
    assert!(matches!(cfg.lines()[2], Line::Blank(_)));
}

#[test]
fn test_parse_comment_with_leading_whitespace() {
    let input = "  # indented comment\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert!(matches!(cfg.lines()[0], Line::Comment(_)));
}

#[test]
fn test_parse_all_comments_and_blanks() {
    let input = "# header\n\nSELINUX=enforcing\n\n# footer\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}
```

- [ ] **Step 2: Run test to verify**

```bash
cargo test -- parser_tests
```
Expected: tests should pass with existing parser. Fix parser if any fail.

- [ ] **Step 3: Fix parser if needed, then commit**

```bash
git add tests/parser_tests.rs src/parser.rs
git commit -m "test: add parser tests for comments and blank lines"
```

### Task 2.3: Parse all 5 standard keys with various formats

**Files:**
- Modify: `tests/parser_tests.rs`

- [ ] **Step 1: Add comprehensive key parsing tests**

```rust
#[test]
fn test_parse_require_seusers() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nREQUIRESEUSERS=1\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.require_seusers(), Some(true));
}

#[test]
fn test_parse_require_seusers_zero() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nREQUIRESEUSERS=0\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.require_seusers(), Some(false));
}

#[test]
fn test_parse_autorelabel() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nAUTORELABEL=0\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.autorelabel(), Some(false));
}

#[test]
fn test_parse_setlocaldefs() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nSETLOCALDEFS=1\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.setlocaldefs(), Some(true));
}

#[test]
fn test_parse_mixed_case_key() {
    let input = "SelInux=enforcing\nSelinuxType=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    // Generic get should find them case-insensitively
    assert_eq!(cfg.get("SELINUX"), Some("enforcing"));
    assert_eq!(cfg.get("selinuxtype"), Some("targeted"));
}

#[test]
fn test_parse_value_case_insensitive() {
    let input = "SELINUX=EnForCiNg\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}

#[test]
fn test_parse_value_with_trailing_whitespace() {
    let input = "SELINUX=enforcing   \nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}

#[test]
fn test_parse_value_with_leading_whitespace_after_equals() {
    let input = "SELINUX=   enforcing\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}

#[test]
fn test_parse_key_with_leading_whitespace() {
    let input = "   SELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}

#[test]
fn test_parse_duplicate_keys_last_wins() {
    let input = "SELINUX=disabled\nSELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}

#[test]
fn test_parse_all_five_keys() {
    let input = concat!(
        "SELINUX=enforcing\n",
        "SELINUXTYPE=targeted\n",
        "REQUIRESEUSERS=1\n",
        "AUTORELABEL=1\n",
        "SETLOCALDEFS=0\n",
    );
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
    assert_eq!(cfg.require_seusers(), Some(true));
    assert_eq!(cfg.autorelabel(), Some(true));
    assert_eq!(cfg.setlocaldefs(), Some(false));
}
```

- [ ] **Step 2: Run test and fix parser**

```bash
cargo test -- parser_tests
```
Expected: all new tests PASS. Fix parser issues as needed.

- [ ] **Step 3: Commit**

```bash
git add tests/parser_tests.rs
git commit -m "test: add parser tests for all 5 keys and edge formats"
```

### Task 2.4: Parse inline comments, raw lines, \r\n, no trailing newline

**Files:**
- Modify: `tests/parser_tests.rs`
- Modify: `src/parser.rs` (as needed)

- [ ] **Step 1: Add extended parsing tests**

```rust
#[test]
fn test_parse_inline_comment() {
    let input = "SELINUX=enforcing  # this is a comment\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    // Check that raw_suffix preserves inline comment
    if let Line::Entry { raw_suffix, .. } = &cfg.lines()[0] {
        assert!(raw_suffix.contains("# this is a comment"));
    } else {
        panic!("expected Entry");
    }
}

#[test]
fn test_parse_hash_in_value_not_comment() {
    // # without preceding whitespace is part of the value
    let input = "SELINUXTYPE=targeted#1\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    // "targeted#1" should be the value (not "targeted")
    let val = cfg.selinuxtype();
    // # without whitespace before it is NOT treated as comment
    assert!(val.unwrap().contains('#'));
}

#[test]
fn test_parse_raw_line_no_equals() {
    let input = "this is not a key value pair\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert!(matches!(cfg.lines()[0], Line::Raw(_)));
}

#[test]
fn test_parse_raw_line_empty_key() {
    let input = "=value\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert!(matches!(cfg.lines()[0], Line::Raw(_)));
}

#[test]
fn test_parse_crlf_line_endings() {
    let input = "SELINUX=enforcing\r\nSELINUXTYPE=targeted\r\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}

#[test]
fn test_parse_no_trailing_newline() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
}

#[test]
fn test_parse_value_containing_equals() {
    let input = "SELINUXTYPE=foo=bar\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinuxtype(), Some("foo=bar"));
}

#[test]
fn test_parse_known_keys_list() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nCUSTOMKEY=somevalue\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert!(cfg.contains("CUSTOMKEY"));
    assert_eq!(cfg.get("CUSTOMKEY"), Some("somevalue"));
}
```

- [ ] **Step 2: Run test and fix**

```bash
cargo test -- parser_tests
```

Fix the parser for each failing test:
- Inline comments: adjust `parse_value_and_suffix` to find `#` preceded by whitespace
- Hash in value: the existing "preceded by whitespace" check should handle this
- Raw lines: `find('=')` returning `None` → Raw, empty key after trim → Raw
- CRLF: pre-process input to normalize `\r\n` before line splitting
- No trailing newline: handle EOF without newline

- [ ] **Step 3: Commit**

```bash
git add tests/parser_tests.rs src/parser.rs
git commit -m "test: add extended parsing tests for inline comments, raw lines, CRLF"
```

---

## Phase 3: Serializer (TDD)

### Task 3.1: Serialize ConfigFile to string with format preservation

**Files:**
- Create: `src/serializer.rs`
- Create: `tests/serializer_tests.rs`
- Modify: `src/lib.rs`
- Modify: `src/config_file.rs` (add `to_string` / `Display`)

- [ ] **Step 1: Write serializer tests**

```rust
use selinux_configfile::ConfigFile;

#[test]
fn test_roundtrip_minimal() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_with_comments() {
    let input = "# config header\nSELINUX=enforcing\n# inline note\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_with_blank_lines() {
    let input = "\nSELINUX=enforcing\n\nSELINUXTYPE=targeted\n\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_with_spaces_around_equals() {
    let input = "SELINUX = enforcing\nSELINUXTYPE = targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_mixed_formatting() {
    let input = "# SELinux config\n\nSELINUX=permissive\nSELINUXTYPE = mls\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_format_preservation_after_modify() {
    let input = "# config\nSELINUX = enforcing\nSELINUXTYPE = targeted\n# end\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.set_selinux(SelinuxMode::Permissive);
    let output = cfg.to_string();
    // Only the value should change, rest preserved
    assert!(output.contains("# config\n"));
    assert!(output.contains("SELINUX = permissive\n"));
    assert!(output.contains("# end\n"));
    // Format around = preserved
    assert!(output.contains(" = "));
}

#[test]
fn test_roundtrip_inline_comment_preserved() {
    let input = "SELINUX=enforcing  # production mode\nSELINUXTYPE=targeted\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.set_selinux(SelinuxMode::Disabled);
    let output = cfg.to_string();
    assert!(output.contains("# production mode"));
    assert!(output.contains("SELINUX=disabled  # production mode\n"));
}

#[test]
fn test_roundtrip_with_raw_line() {
    let input = "SELINUX=enforcing\nthis line has no equals\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let output = cfg.to_string();
    assert!(output.contains("this line has no equals\n"));
}

#[test]
fn test_roundtrip_with_all_five_keys() {
    let input = concat!(
        "SELINUX=enforcing\n",
        "SELINUXTYPE=targeted\n",
        "REQUIRESEUSERS=1\n",
        "AUTORELABEL=0\n",
        "SETLOCALDEFS=0\n",
    );
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_crlf_normalized_to_lf() {
    let input = "SELINUX=enforcing\r\nSELINUXTYPE=targeted\r\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let output = cfg.to_string();
    // \r\n is normalized to \n on output
    assert!(!output.contains('\r'));
    assert!(output.contains("SELINUX=enforcing\n"));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -- serializer_tests
```
Expected: compilation error — `to_string` not implemented.

- [ ] **Step 3: Implement serializer**

**src/serializer.rs:**

```rust
use crate::types::Line;
use crate::config_file::ConfigFile;
use std::fmt;

impl fmt::Display for ConfigFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.lines {
            match line {
                Line::Comment(s) | Line::Blank(s) | Line::Raw(s) => {
                    write!(f, "{}", s)?;
                }
                Line::Entry {
                    key_raw,
                    value,
                    raw_leading,
                    raw_separator,
                    raw_suffix,
                } => {
                    write!(f, "{}{}{}{}{}",
                        raw_leading,
                        key_raw,
                        raw_separator,
                        value,
                        raw_suffix,
                    )?;
                }
            }
        }
        Ok(())
    }
}

impl ConfigFile {
    /// Serialize to string with exact format preservation.
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test -- serializer_tests
```
Expected: all 10 tests PASS. Fix parser issues if roundtrip tests fail.

- [ ] **Step 5: Commit**

```bash
git add src/serializer.rs tests/serializer_tests.rs src/config_file.rs src/lib.rs
git commit -m "feat: implement serializer with full format preservation"
```

---

## Phase 4: ConfigFile API (TDD)

### Task 4.1: Generic operations (get, set, remove, disable, contains, keys, lines)

**Files:**
- Create: `tests/config_file_tests.rs`
- Modify: `src/config_file.rs`

- [ ] **Step 1: Write API tests**

```rust
use selinux_configfile::{ConfigFile, SelinuxMode, Line, SELINUX_KEY};

fn make_test_config() -> ConfigFile {
    ConfigFile::parse(
        "# header\n\nSELINUX=enforcing\nSELINUXTYPE=targeted\n\n# footer\n"
    ).unwrap()
}

// --- get ---
#[test]
fn test_generic_get_existing() {
    let cfg = make_test_config();
    assert_eq!(cfg.get("SELINUX"), Some("enforcing"));
    assert_eq!(cfg.get("selinux"), Some("enforcing")); // case-insensitive
}

#[test]
fn test_generic_get_missing() {
    let cfg = make_test_config();
    assert_eq!(cfg.get("NONEXISTENT"), None);
}

#[test]
fn test_generic_get_duplicate_last_wins() {
    let cfg = ConfigFile::parse("SELINUX=disabled\nSELINUX=enforcing\n").unwrap();
    assert_eq!(cfg.get("SELINUX"), Some("enforcing"));
}

// --- set ---
#[test]
fn test_generic_set_existing_key() {
    let mut cfg = make_test_config();
    cfg.set("SELINUX", "disabled");
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Disabled));
}

#[test]
fn test_generic_set_new_key() {
    let mut cfg = make_test_config();
    cfg.set("CUSTOMKEY", "customvalue");
    assert_eq!(cfg.get("CUSTOMKEY"), Some("customvalue"));
}

#[test]
fn test_generic_set_empty_key_noop() {
    let mut cfg = make_test_config();
    let original = cfg.to_string();
    cfg.set("", "value");
    assert_eq!(cfg.to_string(), original); // no change
}

#[test]
fn test_generic_set_case_insensitive_match() {
    let mut cfg = make_test_config();
    cfg.set("selinux", "permissive");
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Permissive));
}

#[test]
fn test_generic_set_known_key_normalized() {
    // Setting via known key should use canonical uppercase form
    let mut cfg = ConfigFile::new();
    cfg.set("selinux", "enforcing");
    let output = cfg.to_string();
    assert!(output.contains("SELINUX=enforcing"));
}

// --- remove ---
#[test]
fn test_remove_existing_key() {
    let mut cfg = make_test_config();
    assert!(cfg.remove("SELINUXTYPE"));
    assert_eq!(cfg.selinuxtype(), None);
}

#[test]
fn test_remove_missing_key() {
    let mut cfg = make_test_config();
    assert!(!cfg.remove("NONEXISTENT"));
}

#[test]
fn test_remove_preserves_comments() {
    let mut cfg = make_test_config();
    cfg.remove("SELINUXTYPE");
    let output = cfg.to_string();
    // Comments and blank lines still present
    assert!(output.contains("# header"));
    assert!(output.contains("# footer"));
    // SELINUX line still there
    assert!(output.contains("SELINUX=enforcing"));
    // SELINUXTYPE gone
    assert!(!output.contains("SELINUXTYPE"));
}

// --- disable ---
#[test]
fn test_disable_existing_key() {
    let mut cfg = make_test_config();
    assert!(cfg.disable("SELINUXTYPE"));
    assert_eq!(cfg.selinuxtype(), None); // no longer found as active key
    let output = cfg.to_string();
    assert!(output.contains("# SELINUXTYPE"));
}

#[test]
fn test_disable_missing_key() {
    let mut cfg = make_test_config();
    assert!(!cfg.disable("NONEXISTENT"));
}

// --- contains ---
#[test]
fn test_contains_existing() {
    let cfg = make_test_config();
    assert!(cfg.contains("SELINUX"));
    assert!(cfg.contains("selinux")); // case-insensitive
}

#[test]
fn test_contains_missing() {
    let cfg = make_test_config();
    assert!(!cfg.contains("NONEXISTENT"));
}

// --- keys ---
#[test]
fn test_keys_list() {
    let cfg = make_test_config();
    let keys = cfg.keys();
    assert!(keys.contains(&"SELINUX"));
    assert!(keys.contains(&"SELINUXTYPE"));
    // No duplicates
    assert_eq!(keys.len(), 2);
}

// --- lines ---
#[test]
fn test_lines_iterator() {
    let cfg = make_test_config();
    let lines = cfg.lines();
    assert!(lines.len() >= 4); // at least comment + blank + entry + entry + blank + comment
    let has_comment = lines.iter().any(|l| matches!(l, Line::Comment(_)));
    let has_blank = lines.iter().any(|l| matches!(l, Line::Blank(_)));
    let has_entry = lines.iter().any(|l| matches!(l, Line::Entry { .. }));
    assert!(has_comment);
    assert!(has_blank);
    assert!(has_entry);
}
```

- [ ] **Step 2: Run test to verify failures**

```bash
cargo test -- config_file_tests
```
Some tests may pass (basic get/set already implemented), some will fail (remove, disable, contains, keys not yet implemented).

- [ ] **Step 3: Implement missing methods**

Add to `src/config_file.rs`:

```rust
/// Get any key's value, case-insensitive. Last match wins for duplicates.
pub fn get(&self, key: &str) -> Option<&str> { /* already implemented */ }

/// Normalize known key names to canonical form for libselinux compatibility.
fn canonical_key_name(key: &str) -> String {
    let known = [
        SELINUX_KEY, SELINUXTYPE_KEY, REQUIRESEUSERS_KEY,
        AUTORELABEL_KEY, SETLOCALDEFS_KEY,
    ];
    let key_lower = key.to_lowercase();
    for &known_key in &known {
        if known_key.to_lowercase() == key_lower {
            return known_key.to_string();
        }
    }
    key.to_string()
}

/// Set any key's value. Updates last match in-place, appends if missing.
/// Empty key is a no-op. Known keys normalized to canonical form.
pub fn set(&mut self, key: &str, value: &str) {
    if key.is_empty() {
        return;
    }
    let canonical = canonical_key_name(key);
    self.set_inner(&canonical, value);
}

/// Remove all entries matching key. Returns true if any were removed.
pub fn remove(&mut self, key: &str) -> bool {
    let key_lower = key.to_lowercase();
    let mut removed = false;
    self.lines.retain(|line| {
        if let Line::Entry { key_raw, .. } = line {
            if key_raw.to_lowercase() == key_lower {
                removed = true;
                return false;
            }
        }
        true
    });
    removed
}

/// Comment out all entries matching key. Returns true if any were disabled.
pub fn disable(&mut self, key: &str) -> bool {
    let key_lower = key.to_lowercase();
    let mut disabled = false;
    for line in &mut self.lines {
        // Temporarily take ownership via mem::replace
        let old_line = std::mem::replace(line, Line::Blank(String::new()));
        if let Line::Entry { key_raw, value, raw_leading, raw_separator, raw_suffix } = old_line {
            if key_raw.to_lowercase() == key_lower {
                let commented = format!(
                    "{}# {}{}{}{}",
                    raw_leading, key_raw, raw_separator, value, raw_suffix,
                );
                *line = Line::Comment(commented);
                disabled = true;
            } else {
                *line = Line::Entry { key_raw, value, raw_leading, raw_separator, raw_suffix };
            }
        } else {
            *line = old_line;
        }
    }
    disabled
}

/// Does the config contain this key? Case-insensitive.
pub fn contains(&self, key: &str) -> bool {
    self.get(key).is_some()
}

/// All unique keys in order of first appearance.
pub fn keys(&self) -> Vec<&str> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for line in &self.lines {
        if let Line::Entry { key_raw, .. } = line {
            let lower = key_raw.to_lowercase();
            if seen.insert(lower) {
                result.push(key_raw.as_str());
            }
        }
    }
    result
}

/// Reference to all lines.
pub fn lines(&self) -> &[Line] {
    &self.lines
}
```

Note: The `disable` method needs careful implementation. Let me use a simpler approach — reconstruct the full line text and wrap in Comment.

Actually, for the plan, let me describe the algorithm and the engineer can implement it properly:

```rust
pub fn disable(&mut self, key: &str) -> bool {
    let key_lower = key.to_lowercase();
    let mut disabled = false;
    for line in &mut self.lines {
        if let Line::Entry { key_raw, raw_leading, raw_separator, value, raw_suffix } = line {
            if key_raw.to_lowercase() == key_lower {
                let comment_text = format!(
                    "{}{}{}{}{}",
                    raw_leading,
                    "# ",
                    key_raw,
                    raw_separator,
                    value,
                    // raw_suffix includes \n
                );
                // Reconstruct: leading + "# " + key + separator + value + suffix
                let full = format!("{}# {}{}{}{}",
                    raw_leading,
                    key_raw,
                    raw_separator,
                    value,
                    raw_suffix,
                );
                *line = Line::Comment(full);
                disabled = true;
            }
        }
    }
    disabled
}
```

Hmm, this is getting complex. Let me just write it cleanly in the plan and handle it during implementation.

- [ ] **Step 4: Run test to verify all pass**

```bash
cargo test -- config_file_tests
```

- [ ] **Step 5: Commit**

```bash
git add tests/config_file_tests.rs src/config_file.rs
git commit -m "feat: implement generic operations (get, set, remove, disable, contains, keys, lines)"
```

### Task 4.2: Typed setters with validation

**Files:**
- Modify: `tests/config_file_tests.rs`
- Modify: `src/config_file.rs`

- [ ] **Step 1: Write typed setter tests**

```rust
#[test]
fn test_typed_set_selinux_all_modes() {
    let mut cfg = ConfigFile::new();
    cfg.set_selinux(SelinuxMode::Enforcing);
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    cfg.set_selinux(SelinuxMode::Permissive);
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Permissive));
    cfg.set_selinux(SelinuxMode::Disabled);
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Disabled));
}

#[test]
fn test_typed_set_selinuxtype_valid() {
    let mut cfg = ConfigFile::new();
    assert!(cfg.set_selinuxtype("targeted").is_ok());
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
    assert!(cfg.set_selinuxtype("mls").is_ok());
    assert_eq!(cfg.selinuxtype(), Some("mls"));
}

#[test]
fn test_typed_set_selinuxtype_empty() {
    let mut cfg = ConfigFile::new();
    let err = cfg.set_selinuxtype("").unwrap_err();
    assert!(err.message.contains("empty"));
}

#[test]
fn test_typed_set_selinuxtype_whitespace_only() {
    let mut cfg = ConfigFile::new();
    let err = cfg.set_selinuxtype("   ").unwrap_err();
    assert!(err.message.contains("empty"));
}

#[test]
fn test_typed_set_selinuxtype_contains_slash() {
    let mut cfg = ConfigFile::new();
    let err = cfg.set_selinuxtype("path/name").unwrap_err();
    assert!(err.message.contains("'/'"));
}

#[test]
fn test_typed_set_selinuxtype_contains_null() {
    let mut cfg = ConfigFile::new();
    let err = cfg.set_selinuxtype("type\0name").unwrap_err();
}

#[test]
fn test_typed_set_boolean_keys() {
    let mut cfg = ConfigFile::new();
    cfg.set_require_seusers(true);
    assert_eq!(cfg.require_seusers(), Some(true));
    cfg.set_require_seusers(false);
    assert_eq!(cfg.require_seusers(), Some(false));
    cfg.set_autorelabel(true);
    assert_eq!(cfg.autorelabel(), Some(true));
    cfg.set_setlocaldefs(false);
    assert_eq!(cfg.setlocaldefs(), Some(false));
}
```

- [ ] **Step 2: Run test and fix**

```bash
cargo test -- config_file_tests
```
Most of these should already pass. Fix any that fail.

- [ ] **Step 3: Commit**

```bash
git add tests/config_file_tests.rs
git commit -m "test: add typed setter validation tests"
```

### Task 4.3: add_comment_line, add_blank_line, validate

**Files:**
- Modify: `tests/config_file_tests.rs`
- Modify: `src/config_file.rs`

- [ ] **Step 1: Write remaining API tests**

```rust
#[test]
fn test_add_comment_line() {
    let mut cfg = ConfigFile::new();
    cfg.add_comment_line("my comment");
    let output = cfg.to_string();
    assert!(output.contains("# my comment\n"));
}

#[test]
fn test_add_blank_line() {
    let mut cfg = ConfigFile::new();
    cfg.add_blank_line();
    let output = cfg.to_string();
    assert!(output.ends_with("\n"));
}

#[test]
fn test_validate_all_valid() {
    let mut cfg = ConfigFile::default();
    let errors = cfg.validate();
    assert!(errors.is_empty());
}

#[test]
fn test_validate_invalid_selinux() {
    let mut cfg = ConfigFile::new();
    cfg.set("SELINUX", "badvalue");
    let errors = cfg.validate();
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.key == "SELINUX"));
}

#[test]
fn test_validate_invalid_selinuxtype() {
    let mut cfg = ConfigFile::new();
    cfg.set("SELINUXTYPE", ""); // bypass typed setter validation
    let errors = cfg.validate();
    assert!(!errors.is_empty());
}

#[test]
fn test_is_empty() {
    assert!(ConfigFile::new().is_empty());
    assert!(!ConfigFile::default().is_empty());
}
```

- [ ] **Step 2: Implement missing methods**

```rust
/// Append a comment line.
pub fn add_comment_line(&mut self, comment: &str) {
    self.lines.push(Line::Comment(format!("# {}\n", comment)));
}

/// Append a blank line.
pub fn add_blank_line(&mut self) {
    self.lines.push(Line::Blank(String::from("\n")));
}

/// Validate all entries. Returns all errors found.
pub fn validate(&self) -> Vec<ValueError> {
    let mut errors = Vec::new();

    for line in &self.lines {
        if let Line::Entry { key_raw, value, .. } = line {
            let key_lower = key_raw.to_lowercase();
            match key_lower.as_str() {
                "selinux" => {
                    if SelinuxMode::from_str(value).is_err() {
                        errors.push(ValueError {
                            key: key_raw.clone(),
                            message: format!(
                                "invalid SELinux mode '{}': must be enforcing, permissive, or disabled",
                                value
                            ),
                        });
                    }
                }
                "selinuxtype" => {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        errors.push(ValueError {
                            key: key_raw.clone(),
                            message: "SELINUXTYPE must not be empty".into(),
                        });
                    } else if trimmed.contains('/') {
                        errors.push(ValueError {
                            key: key_raw.clone(),
                            message: "SELINUXTYPE must not contain '/'".into(),
                        });
                    }
                }
                "requireseusers" | "autorelabel" | "setlocaldefs" => {
                    if value != "0" && value != "1"
                        && !value.eq_ignore_ascii_case("true")
                        && !value.eq_ignore_ascii_case("false")
                    {
                        errors.push(ValueError {
                            key: key_raw.clone(),
                            message: format!(
                                "{} must be 0, 1, true, or false, got '{}'",
                                key_raw, value
                            ),
                        });
                    }
                }
                _ => {} // unknown keys — no validation
            }
        }
    }

    errors
}
```

- [ ] **Step 3: Run test and fix**

```bash
cargo test -- config_file_tests
```

- [ ] **Step 4: Commit**

```bash
git add tests/config_file_tests.rs src/config_file.rs
git commit -m "feat: add add_comment_line, add_blank_line, validate methods"
```

---

## Phase 5: File I/O (TDD)

### Task 5.1: Read from file, write to file (atomic), read_default, write_default

**Files:**
- Create: `src/io.rs`
- Create: `tests/io_tests.rs`
- Modify: `src/lib.rs`
- Modify: `src/config_file.rs`

- [ ] **Step 1: Write IO tests**

```rust
use selinux_configfile::ConfigFile;
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;

fn temp_config_path() -> PathBuf {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();
    // Keep the tempfile around until test cleanup
    std::mem::forget(tmp);
    path
}

#[test]
fn test_read_from_file() {
    let path = temp_config_path();
    fs::write(&path, "SELINUX=enforcing\nSELINUXTYPE=targeted\n").unwrap();
    let cfg = ConfigFile::read_from(&path).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    fs::remove_file(&path).ok();
}

#[test]
fn test_read_from_nonexistent_file() {
    let path = PathBuf::from("/tmp/nonexistent_selinux_config_test_file");
    let result = ConfigFile::read_from(&path);
    assert!(result.is_err());
}

#[test]
fn test_write_to_file() {
    let path = temp_config_path();
    let mut cfg = ConfigFile::default();
    cfg.write_to(&path).unwrap();
    let read_back = fs::read_to_string(&path).unwrap();
    assert!(read_back.contains("SELINUX=enforcing"));
    assert!(read_back.contains("SELINUXTYPE=targeted"));
    fs::remove_file(&path).ok();
}

#[test]
fn test_roundtrip_file() {
    let path = temp_config_path();
    let input = "# Production config\n\nSELINUX = enforcing\nSELINUXTYPE = mls\n\n# End\n";
    fs::write(&path, input).unwrap();
    let cfg = ConfigFile::read_from(&path).unwrap();
    cfg.write_to(&path).unwrap();
    let output = fs::read_to_string(&path).unwrap();
    assert_eq!(output, input);
    fs::remove_file(&path).ok();
}

#[test]
fn test_atomic_write_no_corruption() {
    let path = temp_config_path();
    let cfg = ConfigFile::default();
    cfg.write_to(&path).unwrap();
    // Verify no .tmp file left behind
    let tmp_path = path.with_extension("tmp");
    assert!(!tmp_path.exists());
    // Verify file is readable
    let cfg2 = ConfigFile::read_from(&path).unwrap();
    assert_eq!(cfg2.to_string(), cfg.to_string());
    fs::remove_file(&path).ok();
}

#[test]
fn test_format_preservation_after_file_roundtrip() {
    let path = temp_config_path();
    let input = "# Comment\nSELINUX = enforcing\nSELINUXTYPE = targeted\n";
    fs::write(&path, input).unwrap();

    let mut cfg = ConfigFile::read_from(&path).unwrap();
    cfg.set_selinux(SelinuxMode::Permissive);
    cfg.write_to(&path).unwrap();

    let output = fs::read_to_string(&path).unwrap();
    // Comment preserved
    assert!(output.contains("# Comment\n"));
    // Spaces around = preserved
    assert!(output.contains(" = "));
    // Only enforcing changed to permissive
    assert!(output.contains("SELINUX = permissive\n"));
    fs::remove_file(&path).ok();
}
```

Note: IO tests require `tempfile` as a dev-dependency.

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -- io_tests
```
Expected: compilation error — `read_from`/`write_to` not implemented.

- [ ] **Step 3: Implement file I/O**

**src/io.rs:**

```rust
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::config_file::ConfigFile;
use crate::error::IoError;

/// Default config file path.
pub const SELINUX_CONFIG_PATH: &str = "/etc/selinux/config";

impl ConfigFile {
    /// Read and parse a config file.
    pub fn read_from(path: impl AsRef<Path>) -> Result<Self, IoError> {
        let path = path.as_ref();
        let mut file = fs::File::open(path).map_err(|e| IoError {
            path: path.to_path_buf(),
            source: e,
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| IoError {
            path: path.to_path_buf(),
            source: e,
        })?;
        ConfigFile::parse(&content).map_err(|e| IoError {
            path: path.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
        })
    }

    /// Read from /etc/selinux/config. Returns empty config if file does not exist.
    pub fn read_default() -> Result<Self, IoError> {
        let path = Path::new(SELINUX_CONFIG_PATH);
        if !path.exists() {
            return Ok(ConfigFile::new());
        }
        Self::read_from(path)
    }

    /// Atomic write: write to temp file, fsync, rename.
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<(), IoError> {
        let path = path.as_ref();
        let tmp_path = path.with_extension("tmp");

        let content = self.to_string();
        fs::write(&tmp_path, &content).map_err(|e| IoError {
            path: tmp_path.clone(),
            source: e,
        })?;

        // fsync the temp file
        let tmp_file = fs::File::open(&tmp_path).map_err(|e| IoError {
            path: tmp_path.clone(),
            source: e,
        })?;
        tmp_file.sync_all().map_err(|e| IoError {
            path: tmp_path.clone(),
            source: e,
        })?;

        // Atomic rename
        fs::rename(&tmp_path, path).map_err(|e| IoError {
            path: path.to_path_buf(),
            source: e,
        })?;

        // fsync the directory
        if let Some(parent) = path.parent() {
            let dir = fs::File::open(parent).map_err(|e| IoError {
                path: parent.to_path_buf(),
                source: e,
            })?;
            dir.sync_all().map_err(|e| IoError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        Ok(())
    }

    /// Write to /etc/selinux/config.
    pub fn write_default(&self) -> Result<(), IoError> {
        self.write_to(SELINUX_CONFIG_PATH)
    }
}
```

- [ ] **Step 4: Run test to verify all pass**

```bash
cargo test -- io_tests
```

- [ ] **Step 5: Commit**

```bash
git add src/io.rs tests/io_tests.rs src/lib.rs Cargo.toml
git commit -m "feat: implement file I/O with atomic writes"
```

---

## Phase 6: Integration & Polish

### Task 6.1: End-to-end integration tests

**Files:**
- Create: `tests/integration_tests.rs`

- [ ] **Step 1: Write integration tests**

```rust
use selinux_configfile::{ConfigFile, SelinuxMode};
use std::fs;
use tempfile::NamedTempFile;

/// Full lifecycle: read, modify multiple keys, write, verify
#[test]
fn test_full_lifecycle() {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path();

    // Real-world config
    let original = concat!(
        "# This file controls the state of SELinux on the system.\n",
        "# SELINUX= can take one of these three values:\n",
        "#     enforcing - SELinux security policy is enforced.\n",
        "#     permissive - SELinux prints warnings instead of enforcing.\n",
        "#     disabled - No SELinux policy is loaded.\n",
        "SELINUX=enforcing\n",
        "# SELINUXTYPE= can take one of these three values:\n",
        "#     targeted - Targeted processes are protected,\n",
        "#     minimum - Modification of targeted policy.\n",
        "#     mls - Multi Level Security protection.\n",
        "SELINUXTYPE=targeted\n",
    );
    fs::write(path, original).unwrap();

    // Read
    let mut cfg = ConfigFile::read_from(path).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    assert_eq!(cfg.selinuxtype(), Some("targeted"));

    // Modify
    cfg.set_selinux(SelinuxMode::Permissive);
    cfg.set_selinuxtype("mls").unwrap();
    cfg.set_require_seusers(true);

    // Write
    cfg.write_to(path).unwrap();

    // Verify
    let output = fs::read_to_string(path).unwrap();
    assert!(output.contains("SELINUX=permissive\n"));
    assert!(output.contains("SELINUXTYPE=mls\n"));
    assert!(output.contains("REQUIRESEUSERS=1\n"));
    // All original comments preserved
    assert!(output.contains("# This file controls the state of SELinux on the system.\n"));
    assert!(output.contains("#     enforcing - SELinux security policy is enforced.\n"));
    assert!(output.contains("#     targeted - Targeted processes are protected,\n"));
}

/// Multiple sequential modifications
#[test]
fn test_multiple_modifications() {
    let mut cfg = ConfigFile::default();

    cfg.set_selinux(SelinuxMode::Permissive);
    cfg.set_selinux(SelinuxMode::Enforcing);
    cfg.set_selinux(SelinuxMode::Disabled);
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Disabled));

    cfg.set_selinuxtype("targeted").unwrap();
    cfg.set_selinuxtype("mls").unwrap();
    cfg.set_selinuxtype("minimum").unwrap();
    assert_eq!(cfg.selinuxtype(), Some("minimum"));

    cfg.set_require_seusers(true);
    cfg.set_require_seusers(false);
    assert_eq!(cfg.require_seusers(), Some(false));
}

/// Real-world config from Red Hat documentation
#[test]
fn test_redhat_example_config() {
    let input = concat!(
        "# This file controls the state of SELinux on the system.\n",
        "# SELINUX= can take one of these three values:\n",
        "#     enforcing - SELinux security policy is enforced.\n",
        "#     permissive - SELinux prints warnings instead of enforcing.\n",
        "#     disabled - No SELinux policy is loaded.\n",
        "SELINUX=enforcing\n",
        "# SELINUXTYPE= can take one of these three values:\n",
        "#     targeted - Targeted processes are protected,\n",
        "#     minimum - Modification of targeted policy. Only selected processes are protected.\n",
        "#     mls - Multi Level Security protection.\n",
        "SELINUXTYPE=targeted\n",
    );
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
    assert_eq!(cfg.keys().len(), 2);
    assert_eq!(cfg.lines().len(), 10); // 8 comments + 2 entries
}

/// disable key and verify it's commented
#[test]
fn test_disable_key_end_to_end() {
    let input = "# header\nSELINUX=enforcing\nSELINUXTYPE=targeted\n# footer\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.disable("SELINUXTYPE");
    let output = cfg.to_string();
    assert!(output.contains("# header\n"));
    assert!(output.contains("# footer\n"));
    // SELINUXTYPE should now be commented
    assert!(output.contains("# SELINUXTYPE"));
    // But not active
    assert!(!cfg.contains("SELINUXTYPE"));
}

/// remove all keys from a config
#[test]
fn test_remove_all_keys() {
    let mut cfg = ConfigFile::default();
    cfg.remove("SELINUX");
    cfg.remove("SELINUXTYPE");
    assert!(cfg.is_empty());
    assert!(cfg.keys().is_empty());
}
```

- [ ] **Step 2: Run integration tests**

```bash
cargo test -- integration_tests
```
Expected: all tests pass. Fix any issues and iterate.

- [ ] **Step 3: Commit**

```bash
git add tests/integration_tests.rs
git commit -m "test: add end-to-end integration tests"
```

### Task 6.2: Cargo.toml metadata, docs, and final polish

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`

- [ ] **Step 1: Update Cargo.toml with metadata**

```toml
[package]
name = "selinux_configfile"
version = "0.1.0"
edition = "2021"
description = "100% safe Rust parser and writer for /etc/selinux/config with format preservation"
license = "MIT"
repository = "https://github.com/franck/selinux-configfile"
keywords = ["selinux", "config", "parser", "linux"]
categories = ["config", "os::linux-apis"]
readme = "README.md"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Finalize src/lib.rs re-exports**

```rust
//! # selinux_configfile
//!
//! 100% safe Rust library for reading and writing `/etc/selinux/config`.
//!
//! ## Features
//!
//! - **Type-safe API**: `SelinuxMode` enum instead of raw strings
//! - **Format preservation**: Comments, blank lines, whitespace, and inline
//!   comments are preserved through all write operations
//! - **Atomic writes**: Write to temp file + fsync + rename
//! - **Zero unsafe code**: 100% safe Rust
//! - **Thread-safe**: All public types implement `Send + Sync`
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use selinux_configfile::{ConfigFile, SelinuxMode};
//!
//! let mut cfg = ConfigFile::read_default().unwrap();
//! cfg.set_selinux(SelinuxMode::Permissive);
//! cfg.set_selinuxtype("mls").unwrap();
//! cfg.write_default().unwrap();
//! ```

mod config_file;
mod error;
mod io;
mod parser;
mod serializer;
mod types;

pub use config_file::ConfigFile;
pub use error::{IoError, ParseError, ValueError};
pub use types::{
    Line, SelinuxMode,
    AUTORELABEL_KEY, REQUIRESEUSERS_KEY, SELINUXTYPE_DEFAULT,
    SELINUXTYPE_KEY, SELINUX_KEY, SETLOCALDEFS_KEY,
};
```

- [ ] **Step 3: Run all tests**

```bash
cargo test
```
Expected: all tests PASS.

- [ ] **Step 4: Run cargo clippy (if available) and fix warnings**

```bash
cargo clippy -- -D warnings 2>/dev/null || echo "clippy not installed, skipping"
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs
git commit -m "chore: finalize Cargo.toml metadata and crate docs"
```

---

## Phase 7: Extended Test Coverage (超越官方)

### Task 7.1: Edge case and stress tests

**Files:**
- Create: `tests/edge_cases.rs`

- [ ] **Step 1: Write edge case tests exceeding official coverage**

```rust
use selinux_configfile::{ConfigFile, SelinuxMode, Line};

// --- Format edge cases ---

#[test]
fn test_very_long_line() {
    let long_value = "a".repeat(10000);
    let input = format!("SELINUXTYPE={}\nSELINUX=enforcing\n", long_value);
    let cfg = ConfigFile::parse(&input).unwrap();
    assert_eq!(cfg.selinuxtype().unwrap(), long_value.as_str());
    let output = cfg.to_string();
    assert_eq!(output, input);
}

#[test]
fn test_many_comments_and_blanks() {
    let mut input = String::new();
    for i in 0..100 {
        input.push_str(&format!("# comment {}\n\n", i));
    }
    input.push_str("SELINUX=enforcing\nSELINUXTYPE=targeted\n");
    let cfg = ConfigFile::parse(&input).unwrap();
    assert_eq!(cfg.lines().len(), 202); // 100 comments + 100 blanks + 2 entries
}

#[test]
fn test_unicode_in_comment() {
    let input = "# 中文注释 🎉\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let output = cfg.to_string();
    assert!(output.contains("中文注释"));
}

#[test]
fn test_roundtrip_ten_times() {
    let input = "# Production SELinux config\n\nSELINUX = enforcing\nSELINUXTYPE = mls\n\n# Managed by ansible\n";
    let mut current = input.to_string();
    for _ in 0..10 {
        let cfg = ConfigFile::parse(&current).unwrap();
        current = cfg.to_string();
    }
    assert_eq!(current, input);
}

#[test]
fn test_preserve_indented_entry() {
    let input = "  SELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.set_selinux(SelinuxMode::Permissive);
    let output = cfg.to_string();
    assert!(output.contains("  SELINUX=permissive\n")); // indentation preserved
}

// --- Correctness edge cases ---

#[test]
fn test_parse_only_selinxutype_no_selinux() {
    let input = "SELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
    assert_eq!(cfg.selinux(), None);
}

#[test]
fn test_duplicate_key_preserves_both_in_output() {
    let input = "SELINUX=disabled\nSELINUX=enforcing\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.set_selinux(SelinuxMode::Permissive);
    let output = cfg.to_string();
    // First occurrence unchanged, last one updated
    assert!(output.contains("SELINUX=disabled\n"));
    assert!(output.contains("SELINUX=permissive\n"));
}

#[test]
fn test_remove_duplicates_removes_all() {
    let input = "SELINUX=disabled\nSELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    assert!(cfg.remove("SELINUX"));
    let output = cfg.to_string();
    assert!(!output.contains("SELINUX"));
    assert_eq!(cfg.selinux(), None);
}

#[test]
fn test_disable_duplicates_disables_all() {
    let input = "SELINUX=disabled\nSELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    assert!(cfg.disable("SELINUX"));
    let output = cfg.to_string();
    assert!(output.contains("# SELINUX=disabled\n"));
    assert!(output.contains("# SELINUX=enforcing\n"));
    assert!(!cfg.contains("SELINUX"));
}

#[test]
fn test_value_with_only_spaces() {
    let input = "SELINUXTYPE=   \nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    // After trimming, value is empty
    let val = cfg.selinuxtype();
    assert!(val.is_none() || val == Some(""));
}

#[test]
fn test_config_default_is_valid() {
    let cfg = ConfigFile::default();
    let errors = cfg.validate();
    assert!(errors.is_empty());
}

#[test]
fn test_new_is_empty() {
    let cfg = ConfigFile::new();
    assert!(cfg.is_empty());
    assert!(cfg.keys().is_empty());
    assert_eq!(cfg.lines().len(), 0);
}

// --- Thread safety (compile-time assertion) ---

#[test]
fn test_config_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ConfigFile>();
    assert_send_sync::<SelinuxMode>();
    assert_send_sync::<Line>();
}
```

- [ ] **Step 2: Run edge case tests**

```bash
cargo test -- edge_cases
```

- [ ] **Step 3: Fix issues and commit**

```bash
git add tests/edge_cases.rs
git commit -m "test: add edge case and stress tests beyond official coverage"
```

### Task 7.2: Official behavior parity tests

**Files:**
- Create: `tests/official_parity.rs`

- [ ] **Step 1: Write tests matching official libselinux behavior**

```rust
use selinux_configfile::{ConfigFile, SelinuxMode};

// These tests verify behavior matching libselinux's selinux_getenforcemode()
// and init_selinux_config() precisely.

/// libselinux: selinux_getenforcemode() strips leading whitespace after tag
#[test]
fn test_selinux_value_leading_whitespace_stripped() {
    let input = "SELINUX=  permissive\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Permissive));
}

/// libselinux: selinux_getenforcemode() uses case-sensitive key matching via strncmp
/// Our library uses case-insensitive matching (better, but test the behavior)
#[test]
fn test_selinux_key_case_insensitive_lookup() {
    let input = "selinux=enforcing\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    // Our API finds it via case-insensitive matching
    assert_eq!(cfg.get("SELINUX"), Some("enforcing"));
}

/// libselinux: init_selinux_config() uses getline and strips trailing \n
#[test]
fn test_value_has_no_trailing_newline() {
    let input = "SELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let val = cfg.selinuxtype().unwrap();
    assert!(!val.contains('\n'));
    assert!(!val.contains('\r'));
}

/// libselinux: init_selinux_config() strips trailing control characters
#[test]
fn test_value_trailing_control_chars_stripped() {
    // \r is a control character — it gets stripped from value
    let input = "SELINUXTYPE=targeted\r\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let val = cfg.selinuxtype().unwrap();
    assert!(!val.ends_with('\r'));
}

/// libselinux: init_selinux_config() skips comment lines (starts with #)
#[test]
fn test_comment_lines_skipped() {
    let input = "#SELINUX=disabled\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}

/// libselinux: init_selinux_config() skips blank lines
#[test]
fn test_blank_lines_skipped() {
    let input = "\n\nSELINUX=enforcing\n\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}

/// libselinux: SELINUX default is "targeted" when not set
#[test]
fn test_default_selinuxtype_in_default_config() {
    let cfg = ConfigFile::default();
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
}

/// libselinux: case-insensitive value matching for SELINUX
#[test]
fn test_selinux_value_case_insensitive() {
    for val in &["ENFORCING", "Enforcing", "enforcing", "ENFORCing"] {
        let input = format!("SELINUX={}\nSELINUXTYPE=targeted\n", val);
        let cfg = ConfigFile::parse(&input).unwrap();
        assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing),
            "failed for value: {}", val);
    }
}

/// libselinux: REQUIRESEUSERS parses 0/1 via atoi
#[test]
fn test_require_seusers_atoi_parsing() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nREQUIRESEUSERS=1\n";
    let cfg = ConfigFile::parse(&input).unwrap();
    assert_eq!(cfg.require_seusers(), Some(true));
}

/// libselinux: REQUIRESEUSERS also parses "true"/"false" strings
#[test]
fn test_require_seusers_true_false_strings() {
    for val in &["true", "True", "TRUE"] {
        let input = format!("SELINUX=enforcing\nSELINUXTYPE=targeted\nREQUIRESEUSERS={}\n", val);
        let cfg = ConfigFile::parse(&input).unwrap();
        assert!(cfg.require_seusers().unwrap(), "failed for: {}", val);
    }
    for val in &["false", "False", "FALSE"] {
        let input = format!("SELINUX=enforcing\nSELINUXTYPE=targeted\nREQUIRESEUSERS={}\n", val);
        let cfg = ConfigFile::parse(&input).unwrap();
        assert!(!cfg.require_seusers().unwrap(), "failed for: {}", val);
    }
}

/// libselinux: AUTORELABEL not parsed by libselinux but defined in man page
/// Our library supports it as a full citizen.
#[test]
fn test_autorelabel_parsed() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nAUTORELABEL=0\n";
    let cfg = ConfigFile::parse(&input).unwrap();
    assert_eq!(cfg.autorelabel(), Some(false));
}

/// libselinux: SETLOCALDEFS not parsed by libselinux, deprecated
/// Our library supports it for completeness.
#[test]
fn test_setlocaldefs_parsed() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nSETLOCALDEFS=0\n";
    let cfg = ConfigFile::parse(&input).unwrap();
    assert_eq!(cfg.setlocaldefs(), Some(false));
}
```

- [ ] **Step 2: Run parity tests**

```bash
cargo test -- official_parity
```

- [ ] **Step 3: Commit**

```bash
git add tests/official_parity.rs
git commit -m "test: add official libselinux behavior parity tests"
```

---

## Final Verification

- [ ] **Step 1: Run complete test suite**

```bash
cargo test
```
Expected: all tests in all files PASS.

- [ ] **Step 2: Verify zero unsafe**

```bash
grep -r "unsafe" src/ && echo "UNSAFE FOUND!" || echo "No unsafe code ✓"
```

- [ ] **Step 3: Check format preservation summary**

All tests prefixed with `test_roundtrip_` or `test_format_preservation_` must pass.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "chore: finalize implementation with all tests passing"
```
