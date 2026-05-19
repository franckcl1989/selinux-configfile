//! Demonstrates selinux_configfile's format preservation guarantee.
//!
//! Comments, whitespace around `=`, inline comments, and blank lines are
//! all preserved through a read-modify-write cycle.

use selinux_configfile::{ConfigFile, SelinuxMode};

fn main() {
    let original = "\
# Production SELinux configuration
# Last updated: 2026-01-01

SELINUX = enforcing  # must be enforcing for production
SELINUXTYPE = targeted

# End of file
";
    println!("=== Original ===");
    println!("{}", original);

    let mut cfg = ConfigFile::parse(original).expect("valid config");

    // Modify only the value of SELINUX
    cfg.set_selinux(SelinuxMode::Permissive);

    let output = cfg.to_string();

    println!("=== After set_selinux(Permissive) ===");
    println!("{}", output);

    // Verify format preservation
    assert_eq!(
        output,
        original.replace("enforcing", "permissive"),
        "Only 'enforcing' should have changed to 'permissive'"
    );

    println!("SUCCESS: Format preservation verified!");
}
