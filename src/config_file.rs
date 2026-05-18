use crate::error::ValueError;
use crate::types::{
    Line, SelinuxMode, AUTORELABEL_KEY, REQUIRESEUSERS_KEY, SELINUXTYPE_DEFAULT, SELINUXTYPE_KEY,
    SELINUX_KEY, SETLOCALDEFS_KEY,
};
use crate::parser;
use std::collections::HashSet;

/// A parsed `/etc/selinux/config` file with format-preserving lines.
#[derive(Debug, Clone)]
pub struct ConfigFile {
    pub(crate) lines: Vec<Line>,
}

impl ConfigFile {
    /// Create an empty config file (no lines at all).
    pub fn new() -> Self {
        ConfigFile { lines: Vec::new() }
    }

    /// Create a default config: `SELINUX=enforcing`, `SELINUXTYPE=targeted`.
    pub fn default() -> Self {
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

    /// Parse a config file string, preserving all formatting.
    pub fn parse(input: &str) -> Result<Self, crate::error::ParseError> {
        parser::parse(input)
    }

    /// Return all lines (comments, blanks, raws, entries) in order.
    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    /// True when there are no [`Line::Entry`] lines at all.
    pub fn is_empty(&self) -> bool {
        !self.lines.iter().any(|l| matches!(l, Line::Entry { .. }))
    }

    /// Check if a key exists (case-insensitive).
    pub fn contains(&self, key: &str) -> bool {
        self.lines.iter().any(|line| match line {
            Line::Entry { key_raw, .. } => key_raw.eq_ignore_ascii_case(key),
            _ => false,
        })
    }

    /// Generic getter: case-insensitive, last-wins for duplicate keys.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.lines
            .iter()
            .filter_map(|line| {
                if let Line::Entry { key_raw, value, .. } = line {
                    if key_raw.eq_ignore_ascii_case(key) {
                        return Some(value.as_str());
                    }
                }
                None
            })
            .last()
    }

    /// Get the SELinux mode.
    pub fn selinux(&self) -> Option<SelinuxMode> {
        self.get(SELINUX_KEY)
            .and_then(|v| SelinuxMode::from_str(v).ok())
    }

    /// Get the SELinux policy type (raw string).
    pub fn selinuxtype(&self) -> Option<&str> {
        self.get(SELINUXTYPE_KEY)
    }

    /// Get REQUIRESEUSERS as a boolean (1=true, 0=false).
    pub fn require_seusers(&self) -> Option<bool> {
        self.get_bool(REQUIRESEUSERS_KEY)
    }

    /// Get AUTORELABEL as a boolean (1=true, 0=false).
    pub fn autorelabel(&self) -> Option<bool> {
        self.get_bool(AUTORELABEL_KEY)
    }

    /// Get SETLOCALDEFS as a boolean (1=true, 0=false).
    pub fn setlocaldefs(&self) -> Option<bool> {
        self.get_bool(SETLOCALDEFS_KEY)
    }

    /// Helper: interpret a boolean config value.
    /// `"1"` → `Some(true)`, `"0"` → `Some(false)`, anything else → `None`.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| match v {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        })
    }

    // -- internal helpers --

    /// Update the last entry matching `key` (case-insensitive) with a new
    /// value, or append a new entry if none matches.
    pub(crate) fn set_inner(&mut self, key: &str, value: &str) {
        let mut found = false;
        for line in self.lines.iter_mut().rev() {
            if let Line::Entry {
                key_raw, value: v, ..
            } = line
            {
                if key_raw.eq_ignore_ascii_case(key) {
                    *v = value.to_string();
                    found = true;
                    break;
                }
            }
        }
        if !found {
            self.lines.push(Line::Entry {
                key_raw: key.to_string(),
                value: value.to_string(),
                raw_leading: String::new(),
                raw_separator: "=".to_string(),
                raw_suffix: "\n".to_string(),
            });
        }
    }

    /// Set SELINUXTYPE without validation (internal use).
    pub(crate) fn set_selinuxtype_raw(&mut self, value: &str) {
        self.set_inner(SELINUXTYPE_KEY, value);
    }

    // -- typed setters --

    /// Set the SELinux mode.
    pub fn set_selinux(&mut self, mode: SelinuxMode) {
        self.set_inner(SELINUX_KEY, &mode.to_string());
    }

    /// Set the SELinux policy type with validation:
    /// - Must be non-empty after trimming
    /// - Must not contain `/`
    /// - Must not contain ASCII control characters (0x00–0x1F, 0x7F)
    pub fn set_selinuxtype(&mut self, value: &str) -> Result<(), ValueError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ValueError {
                key: SELINUXTYPE_KEY.to_string(),
                message: "SELINUXTYPE value must not be empty".to_string(),
            });
        }
        if trimmed.contains('/') {
            return Err(ValueError {
                key: SELINUXTYPE_KEY.to_string(),
                message: format!(
                    "SELINUXTYPE value must not contain '/': '{}'",
                    trimmed
                ),
            });
        }
        if trimmed
            .chars()
            .any(|c| (c as u32) <= 0x1F || (c as u32) == 0x7F)
        {
            return Err(ValueError {
                key: SELINUXTYPE_KEY.to_string(),
                message: format!(
                    "SELINUXTYPE value contains control characters: '{}'",
                    trimmed
                ),
            });
        }
        self.set_inner(SELINUXTYPE_KEY, trimmed);
        Ok(())
    }

    /// Set REQUIRESEUSERS (`"1"` / `"0"`).
    pub fn set_require_seusers(&mut self, val: bool) {
        self.set_inner(REQUIRESEUSERS_KEY, if val { "1" } else { "0" });
    }

    /// Set AUTORELABEL (`"1"` / `"0"`).
    pub fn set_autorelabel(&mut self, val: bool) {
        self.set_inner(AUTORELABEL_KEY, if val { "1" } else { "0" });
    }

    /// Set SETLOCALDEFS (`"1"` / `"0"`).
    pub fn set_setlocaldefs(&mut self, val: bool) {
        self.set_inner(SETLOCALDEFS_KEY, if val { "1" } else { "0" });
    }

    // -- generic API methods --

    /// If `key` matches a known key (case-insensitive), return the canonical
    /// uppercase form (e.g., `"selinux"` → `"SELINUX"`). Otherwise return
    /// `key` as-is.
    pub fn canonical_key_name(key: &str) -> String {
        match key.to_uppercase().as_str() {
            "SELINUX" => SELINUX_KEY.to_string(),
            "SELINUXTYPE" => SELINUXTYPE_KEY.to_string(),
            "REQUIRESEUSERS" => REQUIRESEUSERS_KEY.to_string(),
            "AUTORELABEL" => AUTORELABEL_KEY.to_string(),
            "SETLOCALDEFS" => SETLOCALDEFS_KEY.to_string(),
            _ => key.to_string(),
        }
    }

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
        let canonical = Self::canonical_key_name(key);
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
    /// Each matching `Entry` is converted to a `Comment` with `"# "` prepended:
    /// `{raw_leading}# {key_raw}{raw_separator}{value}{raw_suffix}`
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
            {
                if key_raw.eq_ignore_ascii_case(key) {
                    let commented =
                        format!("{}# {}{}{}{}", raw_leading, key_raw, raw_separator, value, raw_suffix);
                    *line = Line::Comment(commented);
                    disabled = true;
                }
            }
        }
        disabled
    }

    /// Return all unique keys in order of first appearance.
    ///
    /// Deduplication is case-insensitive — `"SELINUX"` and `"selinux"` count as
    /// the same key; the form that appears first is kept.
    pub fn keys(&self) -> Vec<&str> {
        let mut seen = HashSet::new();
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

    /// Append a comment line (`# <comment>\n`).
    pub fn add_comment_line(&mut self, comment: &str) {
        self.lines
            .push(Line::Comment(format!("# {}\n", comment)));
    }

    /// Append a blank line (`\n`).
    pub fn add_blank_line(&mut self) {
        self.lines.push(Line::Blank(String::from("\n")));
    }

    /// Validate all entry values against known key rules.
    ///
    /// - `SELINUX`: must be one of `enforcing`, `permissive`, `disabled`
    ///   (case-insensitive)
    /// - `SELINUXTYPE`: must be non-empty, no `/`, no ASCII control characters
    /// - `REQUIRESEUSERS` / `AUTORELABEL` / `SETLOCALDEFS`: must be `0`, `1`,
    ///   `true`, or `false`
    /// - Unknown keys: skipped
    pub fn validate(&self) -> Vec<ValueError> {
        let mut errors = Vec::new();
        for line in &self.lines {
            if let Line::Entry { key_raw, value, .. } = line {
                match key_raw.to_uppercase().as_str() {
                    "SELINUX" => {
                        if let Err(e) = SelinuxMode::from_str(value) {
                            errors.push(e);
                        }
                    }
                    "SELINUXTYPE" => {
                        let trimmed = value.trim();
                        if trimmed.is_empty() {
                            errors.push(ValueError {
                                key: SELINUXTYPE_KEY.to_string(),
                                message: "SELINUXTYPE value must not be empty".to_string(),
                            });
                        }
                        if trimmed.contains('/') {
                            errors.push(ValueError {
                                key: SELINUXTYPE_KEY.to_string(),
                                message: format!(
                                    "SELINUXTYPE value must not contain '/': '{}'",
                                    trimmed
                                ),
                            });
                        }
                        if trimmed
                            .chars()
                            .any(|c| (c as u32) <= 0x1F || (c as u32) == 0x7F)
                        {
                            errors.push(ValueError {
                                key: SELINUXTYPE_KEY.to_string(),
                                message: format!(
                                    "SELINUXTYPE value contains control characters: '{}'",
                                    trimmed
                                ),
                            });
                        }
                    }
                    "REQUIRESEUSERS" | "AUTORELABEL" | "SETLOCALDEFS" => {
                        let lower = value.to_lowercase();
                        if lower != "1" && lower != "0" && lower != "true" && lower != "false" {
                            errors.push(ValueError {
                                key: key_raw.clone(),
                                message: format!("invalid boolean value: '{}'", value),
                            });
                        }
                    }
                    _ => {} // unknown keys are skipped
                }
            }
        }
        errors
    }
}
