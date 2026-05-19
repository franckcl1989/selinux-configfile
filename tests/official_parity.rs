use selinux_configfile::{ConfigFile, SelinuxMode};

/// libselinux: selinux_getenforcemode() strips leading whitespace after tag
#[test]
fn test_selinux_value_leading_whitespace_stripped() {
    let input = "SELINUX=  permissive\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Permissive));
}

/// libselinux: case-insensitive key lookup (our enhancement)
#[test]
fn test_selinux_key_case_insensitive_lookup() {
    let input = "selinux=enforcing\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
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
    let input = "SELINUXTYPE=targeted\r\nSELINUX=enforcing\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let val = cfg.selinuxtype().unwrap();
    assert!(!val.ends_with('\r'));
}

/// libselinux: init_selinux_config() skips comment lines
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

/// libselinux: SELINUXTYPE default is "targeted"
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
        assert_eq!(
            cfg.selinux(),
            Some(SelinuxMode::Enforcing),
            "failed for value: {}",
            val
        );
    }
}

/// libselinux: REQUIRESEUSERS parses 0/1 via atoi
#[test]
fn test_require_seusers_atoi_parsing() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nREQUIRESEUSERS=1\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.require_seusers(), Some(true));
}

/// libselinux: REQUIRESEUSERS also handles true/false strings
#[test]
fn test_require_seusers_true_false_strings() {
    for val in &["true", "True", "TRUE"] {
        let input = format!(
            "SELINUX=enforcing\nSELINUXTYPE=targeted\nREQUIRESEUSERS={}\n",
            val
        );
        let cfg = ConfigFile::parse(&input).unwrap();
        assert!(cfg.require_seusers().unwrap(), "failed for: {}", val);
    }
    for val in &["false", "False", "FALSE"] {
        let input = format!(
            "SELINUX=enforcing\nSELINUXTYPE=targeted\nREQUIRESEUSERS={}\n",
            val
        );
        let cfg = ConfigFile::parse(&input).unwrap();
        assert!(!cfg.require_seusers().unwrap(), "failed for: {}", val);
    }
}

/// libselinux: AUTORELABEL defined in man page, our library supports it
#[test]
fn test_autorelabel_parsed() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nAUTORELABEL=0\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.autorelabel(), Some(false));
}

/// libselinux: SETLOCALDEFS deprecated but our library supports it
#[test]
fn test_setlocaldefs_parsed() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\nSETLOCALDEFS=0\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.setlocaldefs(), Some(false));
}
