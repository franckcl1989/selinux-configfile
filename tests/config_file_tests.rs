use selinux_configfile::{ConfigFile, SelinuxMode, Line};

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
    assert_eq!(cfg.get("selinux"), Some("enforcing"));
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
    assert_eq!(cfg.to_string(), original);
}

#[test]
fn test_generic_set_case_insensitive_match() {
    let mut cfg = make_test_config();
    cfg.set("selinux", "permissive");
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Permissive));
}

#[test]
fn test_generic_set_known_key_normalized() {
    let mut cfg = ConfigFile::new();
    cfg.set("selinux", "enforcing");
    let output = cfg.to_string();
    assert!(output.contains("SELINUX=enforcing"), "expected canonical key, got: {}", output);
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
    assert!(output.contains("# header"));
    assert!(output.contains("# footer"));
    assert!(output.contains("SELINUX=enforcing"));
    assert!(!output.contains("SELINUXTYPE"));
}

#[test]
fn test_remove_all_duplicates() {
    let input = "SELINUX=disabled\nSELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    assert!(cfg.remove("SELINUX"));
    assert_eq!(cfg.selinux(), None);
    let output = cfg.to_string();
    assert!(!output.contains("SELINUX="), "output should not contain SELINUX= entries, got: {}", output);
}

// --- disable ---
#[test]
fn test_disable_existing_key() {
    let mut cfg = make_test_config();
    assert!(cfg.disable("SELINUXTYPE"));
    assert_eq!(cfg.selinuxtype(), None);
    let output = cfg.to_string();
    assert!(output.contains("# SELINUXTYPE"));
}

#[test]
fn test_disable_missing_key() {
    let mut cfg = make_test_config();
    assert!(!cfg.disable("NONEXISTENT"));
}

#[test]
fn test_disable_all_duplicates() {
    let input = "SELINUX=disabled\nSELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    assert!(cfg.disable("SELINUX"));
    assert!(!cfg.contains("SELINUX"));
    let output = cfg.to_string();
    assert!(output.contains("# SELINUX=disabled\n"));
    assert!(output.contains("# SELINUX=enforcing\n"));
}

// --- contains ---
#[test]
fn test_contains_existing() {
    let cfg = make_test_config();
    assert!(cfg.contains("SELINUX"));
    assert!(cfg.contains("selinux"));
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
    assert!(keys.iter().any(|k| k.eq_ignore_ascii_case("SELINUX")));
    assert!(keys.iter().any(|k| k.eq_ignore_ascii_case("SELINUXTYPE")));
}

#[test]
fn test_keys_no_duplicates() {
    let cfg = ConfigFile::parse("SELINUX=disabled\nSELINUX=enforcing\n").unwrap();
    let keys = cfg.keys();
    assert_eq!(keys.len(), 1);
}

// --- lines ---
#[test]
fn test_lines_iterator() {
    let cfg = make_test_config();
    let lines = cfg.lines();
    assert!(lines.len() >= 4);
    let has_comment = lines.iter().any(|l| matches!(l, Line::Comment(_)));
    let has_blank = lines.iter().any(|l| matches!(l, Line::Blank(_)));
    let has_entry = lines.iter().any(|l| matches!(l, Line::Entry { .. }));
    assert!(has_comment);
    assert!(has_blank);
    assert!(has_entry);
}

// --- add_comment_line ---
#[test]
fn test_add_comment_line() {
    let mut cfg = ConfigFile::new();
    cfg.add_comment_line("my comment");
    let output = cfg.to_string();
    assert!(output.contains("# my comment\n"));
}

// --- add_blank_line ---
#[test]
fn test_add_blank_line() {
    let mut cfg = ConfigFile::new();
    cfg.add_blank_line();
    let output = cfg.to_string();
    assert!(output.contains("\n"));
}

// --- validate ---
#[test]
fn test_validate_all_valid() {
    let cfg = ConfigFile::default();
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
fn test_validate_unknown_key_skipped() {
    let mut cfg = ConfigFile::new();
    cfg.set("CUSTOMKEY", "anything goes");
    let errors = cfg.validate();
    assert!(errors.is_empty());
}
