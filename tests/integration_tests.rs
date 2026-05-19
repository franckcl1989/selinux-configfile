use selinux_configfile::{ConfigFile, SelinuxMode};
use std::fs;

/// Full lifecycle: read, modify multiple keys, write, verify
#[test]
fn test_full_lifecycle() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();

    // Real-world config from Red Hat documentation
    let original = concat!(
        "# This file controls the state of SELinux on the system.\n",
        "# SELINUX= can take one of these three values:\n",
        "#     enforcing - SELinux security policy is enforced.\n",
        "#     permissive - SELinux prints warnings instead of enforcing.\n",
        "#     disabled - No SELinux policy is loaded.\n",
        "SELINUX=enforcing\n",
        "# SELINUXTYPE= can take one of these three values:\n",
        "#     targeted - Targeted processes are protected,\n",
        "#     minimum - Modification of targeted policy.\n",
        "#     mls - Multi Level Security protection.\n",
        "SELINUXTYPE=targeted\n",
    );
    fs::write(&path, original).unwrap();

    let mut cfg = ConfigFile::read_from(&path).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    assert_eq!(cfg.selinuxtype(), Some("targeted"));

    cfg.set_selinux(SelinuxMode::Permissive);
    cfg.set_selinuxtype("mls").unwrap();
    cfg.set_require_seusers(true);
    cfg.write_to(&path).unwrap();

    let output = fs::read_to_string(&path).unwrap();
    assert!(
        output.contains("SELINUX=permissive\n"),
        "output: {}",
        output
    );
    assert!(output.contains("SELINUXTYPE=mls\n"));
    assert!(output.contains("REQUIRESEUSERS=1\n"));
    assert!(output.contains("# This file controls the state of SELinux on the system.\n"));
    assert!(output.contains("#     enforcing - SELinux security policy is enforced.\n"));
    assert!(output.contains("#     targeted - Targeted processes are protected,\n"));

    fs::remove_file(&path).ok();
}

/// Multiple sequential modifications
#[test]
fn test_multiple_modifications() {
    let mut cfg = ConfigFile::minimal();
    cfg.set_selinux(SelinuxMode::Permissive);
    cfg.set_selinux(SelinuxMode::Enforcing);
    cfg.set_selinux(SelinuxMode::Disabled);
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Disabled));

    cfg.set_selinuxtype("targeted").unwrap();
    cfg.set_selinuxtype("mls").unwrap();
    cfg.set_selinuxtype("minimum").unwrap();
    assert_eq!(cfg.selinuxtype(), Some("minimum"));

    cfg.set_require_seusers(true);
    cfg.set_require_seusers(false);
    assert_eq!(cfg.require_seusers(), Some(false));
}

/// Real-world config from Red Hat documentation
#[test]
fn test_redhat_example_config() {
    let input = concat!(
        "# This file controls the state of SELinux on the system.\n",
        "# SELINUX= can take one of these three values:\n",
        "#     enforcing - SELinux security policy is enforced.\n",
        "#     permissive - SELinux prints warnings instead of enforcing.\n",
        "#     disabled - No SELinux policy is loaded.\n",
        "SELINUX=enforcing\n",
        "# SELINUXTYPE= can take one of these three values:\n",
        "#     targeted - Targeted processes are protected,\n",
        "#     minimum - Modification of targeted policy. Only selected processes are protected.\n",
        "#     mls - Multi Level Security protection.\n",
        "SELINUXTYPE=targeted\n",
    );
    let cfg = ConfigFile::parse(input).unwrap();
    assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
    assert_eq!(cfg.selinuxtype(), Some("targeted"));
    assert_eq!(cfg.keys().len(), 2);
    assert_eq!(cfg.lines().len(), 11);
}

/// disable key and verify it's commented
#[test]
fn test_disable_key_end_to_end() {
    let input = "# header\nSELINUX=enforcing\nSELINUXTYPE=targeted\n# footer\n";
    let mut cfg = ConfigFile::parse(input).unwrap();
    cfg.disable("SELINUXTYPE");
    let output = cfg.to_string();
    assert!(output.contains("# header\n"));
    assert!(output.contains("# footer\n"));
    assert!(!cfg.contains("SELINUXTYPE"));
}

/// remove all keys from a config
#[test]
fn test_remove_all_keys() {
    let mut cfg = ConfigFile::minimal();
    cfg.remove("SELINUX");
    cfg.remove("SELINUXTYPE");
    assert!(cfg.is_empty());
    assert!(cfg.keys().is_empty());
}
