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
    let mode: SelinuxMode = "enforcing".parse().unwrap();
    assert_eq!(mode, SelinuxMode::Enforcing);
}

#[test]
fn test_constant_values() {
    assert_eq!(selinux_configfile::SELINUX_KEY, "SELINUX");
    assert_eq!(selinux_configfile::SELINUXTYPE_KEY, "SELINUXTYPE");
    assert_eq!(selinux_configfile::REQUIRESEUSERS_KEY, "REQUIRESEUSERS");
    assert_eq!(selinux_configfile::AUTORELABEL_KEY, "AUTORELABEL");
    assert_eq!(selinux_configfile::SETLOCALDEFS_KEY, "SETLOCALDEFS");
    assert_eq!(selinux_configfile::SELINUXTYPE_DEFAULT, "targeted");
}

#[test]
fn test_line_comment() {
    use selinux_configfile::Line;
    let line = Line::Comment(String::from("# SELinux configuration\n"));
    assert_eq!(line, Line::Comment(String::from("# SELinux configuration\n")));
}

#[test]
fn test_line_blank() {
    use selinux_configfile::Line;
    let line = Line::Blank(String::from("\n"));
    assert_eq!(line, Line::Blank(String::from("\n")));
}

#[test]
fn test_line_raw() {
    use selinux_configfile::Line;
    let line = Line::Raw(String::from("this is not key=value\n"));
    assert_eq!(line, Line::Raw(String::from("this is not key=value\n")));
}

#[test]
fn test_line_entry_construct() {
    use selinux_configfile::Line;
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
    use selinux_configfile::Line;
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
