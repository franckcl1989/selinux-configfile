//! The [`ConfigFile`] type — the main high-level API for reading, modifying,
//! validating, and writing SELinux config files.

use std::collections::HashSet;

use crate::error::ValueError;
use crate::parser;
use crate::types::{
    AUTORELABEL_KEY, Line, REQUIRESEUSERS_KEY, SELINUX_KEY, SELINUXTYPE_DEFAULT, SELINUXTYPE_KEY,
    SETLOCALDEFS_KEY, SelinuxMode,
};

/// A parsed `/etc/selinux/config` file with format-preserving lines.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConfigFile {
    pub(crate) lines: Vec<Line>,
}

impl ConfigFile {
    /// Create an empty config file (no lines at all).
    pub fn new() -> Self {
        ConfigFile { lines: Vec::new() }
    }

    /// Parse a config file string, preserving all formatting.
    pub fn parse(input: &str) -> Result<Self, crate::error::ParseError> {
        parser::parse(input)
    }

    /// Return all lines (comments, blanks, raws, entries) in order.
    #[must_use]
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// True when there are no [`Line::Entry`] lines at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.lines.iter().any(|l| matches!(l, Line::Entry { .. }))
    }

    /// Check if a key exists (case-insensitive).
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.lines.iter().any(|line| match line {
            Line::Entry { key_raw, .. } => key_raw.eq_ignore_ascii_case(key),
            _ => false,
        })
    }

    /// Generic getter: case-insensitive, last-wins for duplicate keys.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.lines.iter().rev().find_map(|line| {
            if let Line::Entry { key_raw, value, .. } = line
                && key_raw.eq_ignore_ascii_case(key)
            {
                Some(value.as_str())
            } else {
                None
            }
        })
    }

    /// Get the SELinux mode.
    #[must_use]
    pub fn selinux(&self) -> Option<SelinuxMode> {
        self.get(SELINUX_KEY).and_then(|v| v.parse().ok())
    }

    /// Get the SELinux policy type (raw string).
    #[must_use]
    pub fn selinuxtype(&self) -> Option<&str> {
        self.get(SELINUXTYPE_KEY)
    }

    /// Get REQUIRESEUSERS as a boolean (1=true, 0=false).
    #[must_use]
    pub fn require_seusers(&self) -> Option<bool> {
        self.get_bool(REQUIRESEUSERS_KEY)
    }

    /// Get AUTORELABEL as a boolean (1=true, 0=false).
    #[must_use]
    pub fn autorelabel(&self) -> Option<bool> {
        self.get_bool(AUTORELABEL_KEY)
    }

    /// Get SETLOCALDEFS as a boolean (1=true, 0=false).
    #[must_use]
    pub fn setlocaldefs(&self) -> Option<bool> {
        self.get_bool(SETLOCALDEFS_KEY)
    }

    /// Get a key's value interpreted as a boolean.
    ///
    /// `"1"` / `"true"` → `Some(true)`, `"0"` / `"false"` → `Some(false)`,
    /// anything else → `None`.  Matching is case-insensitive.
    #[must_use]
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)
            .and_then(|v| match v.to_ascii_lowercase().as_str() {
                "1" | "true" => Some(true),
                "0" | "false" => Some(false),
                _ => None,
            })
    }

    // -- typed setters --

    /// Set the SELinux mode.
    pub fn set_selinux(&mut self, mode: SelinuxMode) {
        self.set_inner(SELINUX_KEY, &mode.to_string());
    }

    /// Set the SELinux policy type with validation:
    /// - Must be non-empty after trimming
    /// - Must not contain `/`
    /// - Must not contain ASCII control characters
    pub fn set_selinuxtype(&mut self, value: &str) -> Result<(), ValueError> {
        let errors = validate_selinuxtype_value(value);
        if let Some(e) = errors.into_iter().next() {
            return Err(e);
        }
        let trimmed = value.trim();
        self.set_inner(SELINUXTYPE_KEY, trimmed);
        Ok(())
    }

    /// Set REQUIRESEUSERS (`"1"` / `"0"`).
    pub fn set_require_seusers(&mut self, value: bool) {
        self.set_inner(REQUIRESEUSERS_KEY, if value { "1" } else { "0" });
    }

    /// Set AUTORELABEL (`"1"` / `"0"`).
    pub fn set_autorelabel(&mut self, value: bool) {
        self.set_inner(AUTORELABEL_KEY, if value { "1" } else { "0" });
    }

    /// Set SETLOCALDEFS (`"1"` / `"0"`).
    pub fn set_setlocaldefs(&mut self, value: bool) {
        self.set_inner(SETLOCALDEFS_KEY, if value { "1" } else { "0" });
    }

    // -- generic API methods --

    /// Generic setter.
    ///
    /// - Empty key → no-op
    /// - Known keys → normalized to canonical uppercase form
    /// - Unknown keys → caller's case is preserved
    /// - If the key already exists → updates the **last** matching entry in-place
    /// - If the key does not exist → appends a new `Entry` at the end
    pub fn set(&mut self, key: &str, value: &str) {
        if key.is_empty() {
            return;
        }
        let canonical = canonical_key_name(key);
        self.set_inner(&canonical, value);
    }

    /// Remove **all** entries matching `key` (case-insensitive).
    ///
    /// Returns `true` if any entries were removed.  Comments and blank lines
    /// are not affected.
    pub fn remove(&mut self, key: &str) -> bool {
        let len_before = self.lines.len();
        self.lines.retain(|line| match line {
            Line::Entry { key_raw, .. } => !key_raw.eq_ignore_ascii_case(key),
            _ => true,
        });
        self.lines.len() != len_before
    }

    /// Comment out **all** entries matching `key` (case-insensitive).
    ///
    /// Each matching `Entry` is converted to a `Comment` with `"# "` prepended.
    ///
    /// Returns `true` if any entries were disabled.
    pub fn disable(&mut self, key: &str) -> bool {
        let mut disabled = false;
        for line in self.lines.iter_mut() {
            if let Line::Entry {
                key_raw,
                value,
                raw_leading,
                raw_separator,
                raw_suffix,
            } = line
                && key_raw.eq_ignore_ascii_case(key)
            {
                let commented = format!(
                    "{}# {}{}{}{}",
                    raw_leading, key_raw, raw_separator, value, raw_suffix
                );
                *line = Line::Comment(commented);
                disabled = true;
            }
        }
        disabled
    }

    /// Return all unique keys in order of first appearance.
    ///
    /// Deduplication is case-insensitive.
    #[must_use]
    pub fn keys(&self) -> Vec<&str> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for line in &self.lines {
            if let Line::Entry { key_raw, .. } = line {
                let lower = key_raw.to_ascii_lowercase();
                if seen.insert(lower) {
                    result.push(key_raw.as_str());
                }
            }
        }
        result
    }

    /// Append a comment line (`# <comment>\n`).
    pub fn add_comment_line(&mut self, comment: &str) {
        self.lines.push(Line::Comment(format!("# {}\n", comment)));
    }

    /// Append a blank line (`\n`).
    pub fn add_blank_line(&mut self) {
        self.lines.push(Line::Blank(String::from("\n")));
    }

    /// Validate all entry values against known key rules.
    ///
    /// - `SELINUX`: must be one of `enforcing`, `permissive`, `disabled`
    /// - `SELINUXTYPE`: must be non-empty, no `/`, no ASCII control chars
    /// - `REQUIRESEUSERS` / `AUTORELABEL` / `SETLOCALDEFS`: 0/1/true/false
    /// - Unknown keys: skipped
    #[must_use]
    pub fn validate(&self) -> Vec<ValueError> {
        let mut errors = Vec::new();
        for line in &self.lines {
            if let Line::Entry { key_raw, value, .. } = line {
                let key_upper = key_raw.to_ascii_uppercase();
                match key_upper.as_str() {
                    "SELINUX" if value.parse::<SelinuxMode>().is_err() => {
                        errors.push(ValueError {
                            key: SELINUX_KEY.into(),
                            message: format!("invalid SELinux mode: '{}'", value),
                        });
                    }
                    "SELINUXTYPE" => {
                        errors.extend(validate_selinuxtype_value(value));
                    }
                    "REQUIRESEUSERS" | "AUTORELABEL" | "SETLOCALDEFS" => {
                        if let Some(e) = validate_boolean_value(key_raw, value) {
                            errors.push(e);
                        }
                    }
                    _ => {}
                }
            }
        }
        errors
    }

    // -- internal helpers --

    /// Update the last entry matching `key` (case-insensitive) with a new
    /// value, or append a new entry if none matches.
    #[doc(hidden)]
    pub(crate) fn set_inner(&mut self, key: &str, value: &str) {
        for line in self.lines.iter_mut().rev() {
            if let Line::Entry {
                key_raw, value: v, ..
            } = line
                && key_raw.eq_ignore_ascii_case(key)
            {
                *v = value.to_string();
                return;
            }
        }
        self.lines.push(Line::Entry {
            key_raw: key.to_string(),
            value: value.to_string(),
            raw_leading: String::new(),
            raw_separator: "=".to_string(),
            raw_suffix: "\n".to_string(),
        });
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        let mut cfg = ConfigFile::new();
        cfg.lines.push(Line::Entry {
            key_raw: SELINUX_KEY.to_string(),
            value: "enforcing".to_string(),
            raw_leading: String::new(),
            raw_separator: "=".to_string(),
            raw_suffix: "\n".to_string(),
        });
        cfg.lines.push(Line::Entry {
            key_raw: SELINUXTYPE_KEY.to_string(),
            value: SELINUXTYPE_DEFAULT.to_string(),
            raw_leading: String::new(),
            raw_separator: "=".to_string(),
            raw_suffix: "\n".to_string(),
        });
        cfg
    }
}

// -- private helpers --

/// Normalize known key names to canonical uppercase form.
fn canonical_key_name(key: &str) -> String {
    if key.eq_ignore_ascii_case(SELINUX_KEY) {
        return SELINUX_KEY.into();
    }
    if key.eq_ignore_ascii_case(SELINUXTYPE_KEY) {
        return SELINUXTYPE_KEY.into();
    }
    if key.eq_ignore_ascii_case(REQUIRESEUSERS_KEY) {
        return REQUIRESEUSERS_KEY.into();
    }
    if key.eq_ignore_ascii_case(AUTORELABEL_KEY) {
        return AUTORELABEL_KEY.into();
    }
    if key.eq_ignore_ascii_case(SETLOCALDEFS_KEY) {
        return SETLOCALDEFS_KEY.into();
    }
    key.to_string()
}

/// Validate a SELINUXTYPE value: non-empty, no `/`, no ASCII control chars.
fn validate_selinuxtype_value(value: &str) -> Vec<ValueError> {
    let trimmed = value.trim();
    let mut errors = Vec::new();
    if trimmed.is_empty() {
        errors.push(ValueError {
            key: SELINUXTYPE_KEY.into(),
            message: "SELINUXTYPE value must not be empty".into(),
        });
    }
    if trimmed.contains('/') {
        errors.push(ValueError {
            key: SELINUXTYPE_KEY.into(),
            message: format!("SELINUXTYPE value must not contain '/': '{}'", trimmed),
        });
    }
    if trimmed.chars().any(|c| c.is_ascii_control()) {
        errors.push(ValueError {
            key: SELINUXTYPE_KEY.into(),
            message: format!(
                "SELINUXTYPE value contains control characters: '{}'",
                trimmed
            ),
        });
    }
    errors
}

/// Validate a boolean config value (0/1/true/false).
fn validate_boolean_value(key: &str, value: &str) -> Option<ValueError> {
    let lower = value.to_ascii_lowercase();
    if lower != "1" && lower != "0" && lower != "true" && lower != "false" {
        Some(ValueError {
            key: key.into(),
            message: format!("invalid boolean value: '{}'", value),
        })
    } else {
        None
    }
}
