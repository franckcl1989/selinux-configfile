//! Serialization of [`ConfigFile`] back to a string with exact format
//! preservation via the [`Display`](std::fmt::Display) trait.

use crate::config_file::ConfigFile;
use crate::types::Line;
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
                    write!(
                        f,
                        "{}{}{}{}{}",
                        raw_leading, key_raw, raw_separator, value, raw_suffix
                    )?;
                }
            }
        }
        Ok(())
    }
}
