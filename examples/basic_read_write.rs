//! Basic read-modify-write example for selinux_configfile.
//!
//! This example parses a config string, modifies values, and serializes
//! the result — demonstrating the core read/write workflow.

use selinux_configfile::{ConfigFile, SelinuxMode};

fn main() {
    // Parse an in-memory config
    let input = "\
# SELinux configuration
SELINUX = enforcing
SELINUXTYPE = targeted
";
    let mut cfg = ConfigFile::parse(input).expect("valid config");

    // Inspect values
    println!("Current mode: {:?}", cfg.selinux());
    println!("Current type: {:?}", cfg.selinuxtype());

    // Modify with type-safe API
    cfg.set_selinux(SelinuxMode::Permissive);
    cfg.set_selinuxtype("mls").expect("valid policy type");

    // Serialize — comments and formatting are preserved
    let output = cfg.to_string();
    println!("Modified config:\n{}", output);

    assert!(output.contains("# SELinux configuration"));
    assert!(output.contains("SELINUX = permissive"));
    assert!(output.contains("SELINUXTYPE = mls"));
}
