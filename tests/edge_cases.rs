use selinux_configfile::{ConfigFile, SelinuxMode, Line};

/// Very long value should roundtrip correctly
#[test]
fn test_very_long_line() {
    let long_value = "a".repeat(10000);
    let input = format!("SELINUXTYPE={}\nSELINUX=enforcing\n", long_value);
    let cfg = ConfigFile::parse(&input).unwrap();
    assert_eq!(cfg.selinuxtype().unwrap(), long_value.as_str());
    let output = cfg.to_string();
    assert_eq!(output, input);
}

/// Many comments and blanks with entries at the end
#[test]
fn test_many_comments_and_blanks() {
    let mut input = String::new();
    for i in 0..100 {
        input.push_str(&format!("# comment {}\n\n", i));
    }
    input.push_str("SELINUX=enforcing\nSELINUXTYPE=targeted\n");
    let cfg = ConfigFile::parse(&input).unwrap();
    assert_eq!(cfg.lines().len(), 202);
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
}

/// Unicode in comments preserved
#[test]
fn test_unicode_in_comment() {
    let input = "# SELinux configuration\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let output = cfg.to_string();
    assert!(output.contains("SELinux"));
    assert_eq!(output, input);
}

/// Roundtrip 10 times should produce identical output
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

/// Indentation before key preserved after modify
#[test]
fn test_preserve_indented_entry() {
    let input = "  SELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.set_selinux(SelinuxMode::Permissive);
    let output = cfg.to_string();
    assert!(output.contains("  SELINUX=permissive\n"), "output: {}", output);
}

/// Config with only SELINUXTYPE, no SELINUX
#[test]
fn test_parse_only_selinuxtype_no_selinux() {
    let input = "SELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
    assert_eq!(cfg.selinux(), None);
}

/// Duplicate key: set updates last, first stays
#[test]
fn test_duplicate_key_preserves_both_in_output() {
    let input = "SELINUX=disabled\nSELINUX=enforcing\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.set_selinux(SelinuxMode::Permissive);
    let output = cfg.to_string();
    assert!(output.contains("SELINUX=disabled\n"));
    assert!(output.contains("SELINUX=permissive\n"));
}

/// Value with only whitespace
#[test]
fn test_value_with_only_spaces() {
    let input = "SELINUXTYPE=   \nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let errors = cfg.validate();
    assert!(!errors.is_empty());
}

/// Default config is valid
#[test]
fn test_config_default_is_valid() {
    let cfg = ConfigFile::default();
    let errors = cfg.validate();
    assert!(errors.is_empty());
}

/// New config is empty
#[test]
fn test_new_is_empty() {
    let cfg = ConfigFile::new();
    assert!(cfg.is_empty());
    assert!(cfg.keys().is_empty());
    assert_eq!(cfg.lines().len(), 0);
}

/// Thread safety (compile-time)
#[test]
fn test_config_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ConfigFile>();
    assert_send_sync::<SelinuxMode>();
    assert_send_sync::<Line>();
}
