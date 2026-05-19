# Changelog

All notable changes to `selinux_configfile` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] — 2026-05-19

### Added

- Initial release.
- Type-safe `SelinuxMode` enum (`Enforcing`, `Permissive`, `Disabled`).
- `ConfigFile` struct with full CRUD API.
- Format-preserving parser and serializer (comments, whitespace, inline
  comments survive all read-modify-write cycles).
- Atomic file writes (write temp + fsync + rename).
- Typed getters/setters for all 5 standard SELinux config keys.
- Generic key-value API (`get`/`set`/`remove`/`disable`) for unknown keys.
- Validation (`validate()`) against SELinux config rules.
- `serde` feature flag for `Serialize`/`Deserialize` support.
- Zero `unsafe` code; `#![forbid(unsafe_code)]` verified.
- Comprehensive test suite (121 tests).
- Full rustdoc coverage with `#![deny(missing_docs)]`.

[0.1.0]: https://github.com/franck/selinux-configfile/releases/tag/v0.1.0
