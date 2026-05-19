use selinux_configfile::{ConfigFile, SelinuxMode, Line};

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

#[test]
fn test_parse_inline_comment() {
    let input = "SELINUX=enforcing  # this is a comment\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    if let Line::Entry { raw_suffix, .. } = &cfg.lines()[0] {
        assert!(raw_suffix.contains("# this is a comment"), "raw_suffix: {:?}", raw_suffix);
    } else {
        panic!("expected Entry");
    }
}

#[test]
fn test_parse_hash_in_value_not_comment() {
    // # without preceding whitespace is part of the value
    let input = "SELINUXTYPE=targeted#1\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let val = cfg.selinuxtype().unwrap();
    assert!(val.contains('#'), "expected # in value, got: {}", val);
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

/// Lenient parser: any text is valid (even binary). Validation is separate.
#[test]
fn test_parse_lenient_accepts_any_input() {
    assert!(ConfigFile::parse("").is_ok());
    assert!(ConfigFile::parse("random text without equals").is_ok());
    assert!(ConfigFile::parse(&"x".repeat(1_000_000)).is_ok());
}
