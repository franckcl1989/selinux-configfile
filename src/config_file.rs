use crate::error::ValueError;
use crate::types::{
    Line, SelinuxMode, AUTORELABEL_KEY, REQUIRESEUSERS_KEY, SELINUXTYPE_DEFAULT, SELINUXTYPE_KEY,
    SELINUX_KEY, SETLOCALDEFS_KEY,
};
use crate::parser;

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
}
