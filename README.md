# selinux-configfile

[![Crates.io](https://img.shields.io/crates/v/selinux_configfile)](https://crates.io/crates/selinux_configfile)
[![docs.rs](https://img.shields.io/docsrs/selinux_configfile)](https://docs.rs/selinux_configfile)
[![CI](https://github.com/franckcl1989/selinux-configfile/actions/workflows/ci.yml/badge.svg)](https://github.com/franckcl1989/selinux-configfile/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/rustc-1.95+-blue.svg)](https://blog.rust-lang.org/2025/12/11/Rust-1.95.0.html)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

100% safe Rust parser and writer for `/etc/selinux/config` with format-preserving writes, type-safe API, and atomic file updates. Zero dependencies beyond `std` (`serde` optional).

## Features

- **Type-safe API** — `SelinuxMode` enum (`Enforcing` / `Permissive` / `Disabled`) instead of raw strings; typed getters and setters for all 5 standard keys
- **Format preservation** — comments, blank lines, indentation, spacing around `=`, and inline comments survive all read-modify-write cycles intact
- **Atomic writes** — write to temp file + `fsync` + `rename` ensures the config file is never left in a corrupt state
- **Zero unsafe** — `#![forbid(unsafe_code)]` ready; verified with `grep -r "unsafe" src/`
- **Thread-safe** — all public types implement `Send` and `Sync`
- **Generic key-value API** — supports unknown/custom keys beyond the standard SELinux set
- **Validation** — validates values against SELinux rules (`enforcing`/`permissive`/`disabled`, boolean keys, SELINUXTYPE restrictions)

## Quick Start

```rust
use selinux_configfile::{ConfigFile, SelinuxMode};

// Parse an in-memory string
let mut cfg = ConfigFile::parse(
    "SELINUX=enforcing\nSELINUXTYPE=targeted\n"
).unwrap();

assert_eq!(cfg.selinux(), Some(SelinuxMode::Enforcing));
assert_eq!(cfg.selinuxtype(), Some("targeted"));

// Modify values with type-safe setters
cfg.set_selinux(SelinuxMode::Permissive);
cfg.set_selinuxtype("mls").unwrap();

// Serialize — comments and formatting preserved
let output = cfg.to_string();
```

```rust
// Read, modify, and write the system config file
use selinux_configfile::{ConfigFile, SelinuxMode};

let mut cfg = ConfigFile::read_default().unwrap();
cfg.set_selinux(SelinuxMode::Permissive);
cfg.write_default().unwrap();
```

## API Overview

### Reading

| Method | Returns |
|--------|---------|
| `selinux()` | `Option<SelinuxMode>` |
| `selinuxtype()` | `Option<&str>` |
| `require_seusers()` | `Option<bool>` |
| `autorelabel()` | `Option<bool>` |
| `setlocaldefs()` | `Option<bool>` |
| `get(key)` | `Option<&str>` — any key, case-insensitive |
| `get_bool(key)` | `Option<bool>` — parses 0/1/true/false |
| `contains(key)` | `bool` |
| `keys()` | `Vec<&str>` — all unique keys |
| `is_empty()` | `bool` |
| `lines()` | `&[Line]` — all lines including comments |
| `validate()` | `Vec<ValueError>` — empty = valid |

### Writing

| Method | Description |
|--------|-------------|
| `set_selinux(mode)` | Set SELinux mode (type-safe) |
| `set_selinuxtype(s)` | Set policy type (validated) |
| `set_require_seusers(b)` | Set REQUIRESEUSERS |
| `set_autorelabel(b)` | Set AUTORELABEL |
| `set_setlocaldefs(b)` | Set SETLOCALDEFS |
| `set(key, value)` | Set any key (generic, case-insensitive) |
| `remove(key)` | Remove all entries for a key |
| `disable(key)` | Comment out all entries for a key |
| `add_comment_line(s)` | Append a `#` comment line |
| `add_blank_line()` | Append a blank line |

### File I/O

| Method | Description |
|--------|-------------|
| `read_from(path)` | Read and parse any path |
| `read_default()` | Read `/etc/selinux/config` |
| `write_to(path)` | Atomic write to any path |
| `write_default()` | Atomic write to `/etc/selinux/config` |

## Format Preservation

The library represents each config file as a sequence of lines:

- `Comment(String)` — `#` comment lines (preserved verbatim)
- `Blank(String)` — empty or whitespace-only lines
- `Raw(String)` — unrecognized lines (no `=`)
- `Entry { key_raw, value, raw_leading, raw_separator, raw_suffix }` — key=value with formatting metadata

When a value is modified, only the `value` field changes. All other fields — leading whitespace, separator style (`=` vs ` = ` vs `  =  `), trailing content (inline comments, whitespace) — are written back exactly as parsed.

```rust
let input = "SELINUX = enforcing  # production\n";
let mut cfg = ConfigFile::parse(input).unwrap();
cfg.set_selinux(SelinuxMode::Permissive);
assert_eq!(cfg.to_string(), "SELINUX = permissive  # production\n");
```

## Standard SELinux Config Keys

| Key | Required | Valid Values |
|-----|----------|--------------|
| `SELINUX` | Yes | `enforcing`, `permissive`, `disabled` |
| `SELINUXTYPE` | Yes | Policy name (e.g., `targeted`, `mls`, `minimum`) |
| `REQUIRESEUSERS` | No | `0` / `1` |
| `AUTORELABEL` | No | `0` / `1` |
| `SETLOCALDEFS` | No (deprecated) | `0` / `1` |

Based on `selinux_config(5)` and libselinux source.

## Installation

```bash
cargo add selinux_configfile
```

## License

MIT
