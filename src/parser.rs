use crate::error::ParseError;
use crate::types::Line;
use crate::config_file::ConfigFile;

/// Parse a `/etc/selinux/config` string into a [`ConfigFile`].
///
/// Accepts both `\n` and `\r\n` line endings.  Normalizes `\r\n` → `\n` in
/// the stored representation so that output always uses Unix line endings.
pub fn parse(input: &str) -> Result<ConfigFile, ParseError> {
    let input = input.replace("\r\n", "\n");
    let mut lines: Vec<Line> = Vec::new();
    let mut remaining = input.as_str();

    while !remaining.is_empty() {
        // ---- extract one line ----
        let (orig_line, line_content, has_newline) =
            if let Some(pos) = remaining.find('\n') {
                let orig = &remaining[..=pos]; // includes '\n'
                let content = &remaining[..pos];
                remaining = &remaining[pos + 1..];
                (orig.to_string(), content.to_string(), true)
            } else {
                let orig = remaining.to_string();
                let content = remaining.to_string();
                remaining = "";
                (orig, content, false)
            };

        // ---- classify and parse the line ----
        let trimmed = line_content.trim_start();
        let raw_leading: String = line_content
            .chars()
            .take(line_content.len() - trimmed.len())
            .collect();

        if trimmed.is_empty() {
            // blank or whitespace-only
            lines.push(Line::Blank(orig_line));
            continue;
        }

        if trimmed.starts_with('#') {
            // comment line
            lines.push(Line::Comment(orig_line));
            continue;
        }

        if let Some(eq_pos) = trimmed.find('=') {
            // ---- potential key=value entry ----
            let text_before_eq = &trimmed[..eq_pos];
            let text_after_eq = &trimmed[eq_pos + 1..];

            let key_raw = text_before_eq.trim_end().to_string();

            if key_raw.is_empty() {
                // empty key (e.g. "=value") → raw line
                lines.push(Line::Raw(orig_line));
                continue;
            }

            // ---- build raw_separator ----
            let left_sep = &text_before_eq[key_raw.len()..];
            let after_eq_trimmed = text_after_eq.trim_start();
            let right_sep_len = text_after_eq.len() - after_eq_trimmed.len();
            let right_sep = &text_after_eq[..right_sep_len];
            let raw_separator = format!("{}={}", left_sep, right_sep);

            // ---- parse value ----
            let newline_part = if has_newline { "\n" } else { "" };
            let (value, comment_suffix) =
                parse_value(after_eq_trimmed);

            let raw_suffix = format!("{}{}", comment_suffix, newline_part);

            lines.push(Line::Entry {
                key_raw,
                value,
                raw_leading,
                raw_separator,
                raw_suffix,
            });
        } else {
            // no '=' found → raw line
            lines.push(Line::Raw(orig_line));
        }
    }

    Ok(ConfigFile { lines })
}

/// Parse the value portion of a key=value line.
///
/// Given the text after `=` with leading whitespace already consumed into
/// `raw_separator`, this returns:
/// - `value`:           the logical value (trailing whitespace and inline
///                      comments stripped)
/// - `comment_suffix`:  everything that was stripped from the value to be
///                      stored in `raw_suffix` (inline comment + whitespace)
fn parse_value(value_body: &str) -> (String, String) {
    // Step 8a: strip trailing whitespace and ASCII control characters
    let trimmed_body = value_body
        .trim_end_matches(|c: char| c.is_ascii_whitespace() || c.is_ascii_control());
    let trailing_stripped = &value_body[trimmed_body.len()..];

    // Step 8b: find inline comment — '#' preceded by whitespace (or at start)
    if let Some(pos) = trimmed_body
        .match_indices('#')
        .find_map(|(p, _)| {
            if p == 0 || trimmed_body.as_bytes()[p - 1].is_ascii_whitespace() {
                Some(p)
            } else {
                None
            }
        })
    {
        // Step 8c: inline comment found
        let value_area = &trimmed_body[..pos];
        let value = value_area.trim_end().to_string();
        // The spaces between value and '#' (if any were trimmed)
        let between = &value_area[value.len()..];
        let comment_and_rest = &trimmed_body[pos..];
        let comment_suffix = format!("{}{}{}", between, comment_and_rest, trailing_stripped);
        (value, comment_suffix)
    } else {
        // Step 8d: no inline comment
        let value = trimmed_body.to_string();
        let comment_suffix = trailing_stripped.to_string();
        (value, comment_suffix)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_parse_value_plain() {
        let (val, suffix) = parse_value("enforcing");
        assert_eq!(val, "enforcing");
        assert_eq!(suffix, "");
    }

    #[test]
    fn test_parse_value_trailing_spaces() {
        let (val, suffix) = parse_value("enforcing   ");
        assert_eq!(val, "enforcing");
        assert_eq!(suffix, "   ");
    }

    #[test]
    fn test_parse_value_inline_comment() {
        let (val, suffix) = parse_value("enforcing  # comment");
        assert_eq!(val, "enforcing");
        assert!(suffix.contains("# comment"));
    }
}
