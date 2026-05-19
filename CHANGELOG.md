# Changelog

All notable changes to `selinux_configfile` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.3] — 2026-05-19

### Changed

- **MSRV** bumped from 1.85 to **1.95**.
- CI matrix updated: test on `stable` and `1.95`.
- All code, tests, docs, and examples aligned with Rust 1.95.

## [0.1.2] — 2026-05-19

### Changed

- CI redesigned for project fit — Linux only, added examples, full feature
  coverage.
- Removed Windows runner and redundant build steps.

### Fixed

- Replaced let-chains with nested `if let` for Rust 1.85 compatibility.

## [0.1.1] — 2026-05-19

### Changed

- Renamed package from `selinux-configfile` back to `selinux_configfile`
  for Crates.io consistency.

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

[0.1.3]: https://github.com/franckcl1989/selinux-configfile/releases/tag/v0.1.3
[0.1.2]: https://github.com/franckcl1989/selinux-configfile/releases/tag/v0.1.2
[0.1.1]: https://github.com/franckcl1989/selinux-configfile/releases/tag/v0.1.1
[0.1.0]: https://github.com/franckcl1989/selinux-configfile/releases/tag/v0.1.0
