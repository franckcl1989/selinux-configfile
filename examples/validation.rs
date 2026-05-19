//! Demonstrates selinux_configfile's validation capabilities.
//!
//! Type-safe setters validate input, and the `validate()` method checks
//! all existing values in a config.

use selinux_configfile::ConfigFile;

fn main() {
    // set_selinuxtype validates on the spot
    let mut cfg = ConfigFile::default();

    match cfg.set_selinuxtype("") {
        Ok(()) => println!("ok"),
        Err(e) => println!("Validation error: {}", e),
    }

    match cfg.set_selinuxtype("path/name") {
        Ok(()) => println!("ok"),
        Err(e) => println!("Validation error: {}", e),
    }

    cfg.set_selinuxtype("targeted").expect("valid");

    // Generic set bypasses type-safe validation
    cfg.set("SELINUX", "invalid_mode");

    // validate() catches the issue
    let errors = cfg.validate();
    if !errors.is_empty() {
        println!("Found {} validation error(s):", errors.len());
        for error in &errors {
            println!("  {}", error);
        }
    }
    assert!(!errors.is_empty());

    println!("Validation example complete.");
}
