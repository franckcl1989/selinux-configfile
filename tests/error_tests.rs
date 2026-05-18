use selinux_configfile::{ParseError, ValueError};

#[test]
fn test_parse_error_display() {
    let err = ParseError { line: 5, message: String::from("malformed key") };
    let s = err.to_string();
    assert!(s.contains("line 5"), "expected line number in: {}", s);
    assert!(s.contains("malformed key"), "expected message in: {}", s);
}

#[test]
fn test_parse_error_trait() {
    let err = ParseError { line: 5, message: String::from("test") };
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_value_error_display() {
    let err = ValueError { key: String::from("SELINUX"), message: String::from("invalid value") };
    let s = err.to_string();
    assert!(s.contains("SELINUX"), "expected key in: {}", s);
    assert!(s.contains("invalid value"), "expected message in: {}", s);
}

#[test]
fn test_value_error_trait() {
    let err = ValueError { key: String::from("SELINUX"), message: String::from("test") };
    let _: &dyn std::error::Error = &err;
}
