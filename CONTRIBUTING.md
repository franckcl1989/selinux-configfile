# Contributing

## Reporting bugs

Open an issue on [GitHub](https://github.com/franck/selinux-configfile/issues).
Include:

- A minimal config file that triggers the bug
- Expected behavior vs actual behavior
- Rust version (`rustc --version`)

## Development

```bash
git clone https://github.com/franck/selinux-configfile.git
cd selinux-configfile
```

### Running tests

```bash
cargo test --all-features
cargo test --no-default-features
```

### Code quality

```bash
cargo clippy --all-features -- -D warnings
cargo fmt --check
cargo doc --all-features --no-deps -- -D warnings
```

### Conventions

- **TDD**: Write a failing test first, then the implementation.
- **No `unsafe`**: The crate uses `#![forbid(unsafe_code)]`.
- **Format preservation is paramount**: Any change to parsing or
  serialization must preserve comments, whitespace, and inline comments
  through a read-modify-write cycle.

## Code of Conduct

This project follows the
[Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
