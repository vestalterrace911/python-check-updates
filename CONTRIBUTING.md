# Contributing

Thank you for your interest in contributing to pycu!

## Prerequisites

- [Rust](https://rustup.rs) **1.85 or later** (stable toolchain - edition 2024)
- `rustfmt` and `clippy` components:

  ```sh
  rustup component add rustfmt clippy
  ```

## Build

```sh
cargo build
```

## Test

```sh
cargo test --all-features
```

Tests are fully offline - no network calls are made. Fixtures live in `tests/fixtures/`.

To run a single test module:

```sh
cargo test parsers
cargo test version
```

## Code quality

CI enforces formatting, linting, and tests. Run these before pushing:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

All warnings are treated as errors. Fix them before opening a PR.

## Adding a new dependency file format

1. Create `src/parsers/<format>.rs` implementing the `DependencyParser` trait.
2. Register it in `src/parsers/mod.rs` - add detection logic to `detect_parser`.
3. Add a fixture file under `tests/fixtures/` and tests in your new parser module.

## Submitting a pull request

1. Fork the repository and create a branch off `main`.
2. Make your changes, add tests, ensure all checks above pass.
3. Open a pull request with a clear description of what changes and why.

For large changes, open an issue first to discuss the approach before investing time in an implementation.

## Reporting bugs

Use the [bug report template](https://github.com/Logic-py/python-check-updates/issues/new?template=bug_report.yml).

## Security vulnerabilities

Please do **not** open a public issue for security vulnerabilities.
See [SECURITY.md](SECURITY.md) for responsible disclosure instructions.
