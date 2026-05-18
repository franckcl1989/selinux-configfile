use selinux_configfile::{ConfigFile, SelinuxMode};

#[test]
fn test_roundtrip_minimal() {
    let input = "SELINUX=enforcing\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_with_comments() {
    let input = "# config header\nSELINUX=enforcing\n# inline note\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_with_blank_lines() {
    let input = "\nSELINUX=enforcing\n\nSELINUXTYPE=targeted\n\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_with_spaces_around_equals() {
    let input = "SELINUX = enforcing\nSELINUXTYPE = targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_mixed_formatting() {
    let input = "# SELinux config\n\nSELINUX=permissive\nSELINUXTYPE = mls\n";
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_format_preservation_after_modify() {
    let input = "# config\nSELINUX = enforcing\nSELINUXTYPE = targeted\n# end\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.set_selinux(SelinuxMode::Permissive);
    let output = cfg.to_string();
    assert!(output.contains("# config\n"));
    assert!(output.contains("SELINUX = permissive\n"));
    assert!(output.contains("# end\n"));
    assert!(output.contains(" = "));
}

#[test]
fn test_roundtrip_inline_comment_preserved() {
    let input = "SELINUX=enforcing  # production mode\nSELINUXTYPE=targeted\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.set_selinux(SelinuxMode::Disabled);
    let output = cfg.to_string();
    assert!(output.contains("# production mode"));
    assert!(output.contains("SELINUX=disabled  # production mode\n"));
}

#[test]
fn test_roundtrip_with_raw_line() {
    let input = "SELINUX=enforcing\nthis line has no equals\nSELINUXTYPE=targeted\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let output = cfg.to_string();
    assert!(output.contains("this line has no equals\n"));
}

#[test]
fn test_roundtrip_with_all_five_keys() {
    let input = concat!(
        "SELINUX=enforcing\n",
        "SELINUXTYPE=targeted\n",
        "REQUIRESEUSERS=1\n",
        "AUTORELABEL=0\n",
        "SETLOCALDEFS=0\n",
    );
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.to_string(), input);
}

#[test]
fn test_roundtrip_crlf_normalized_to_lf() {
    let input = "SELINUX=enforcing\r\nSELINUXTYPE=targeted\r\n";
    let cfg = ConfigFile::parse(input).unwrap();
    let output = cfg.to_string();
    assert!(!output.contains('\r'));
    assert!(output.contains("SELINUX=enforcing\n"));
}
