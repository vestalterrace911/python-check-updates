# pycu

[![CI](https://github.com/Logic-py/python-check-updates/actions/workflows/ci.yml/badge.svg)](https://github.com/Logic-py/python-check-updates/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/Logic-py/python-check-updates/graph/badge.svg)](https://codecov.io/gh/Logic-py/python-check-updates)
[![Latest release](https://img.shields.io/github/v/release/Logic-py/python-check-updates)](https://github.com/Logic-py/python-check-updates/releases/latest)
[![Crates.io](https://img.shields.io/crates/v/pycu.svg)](https://crates.io/crates/pycu)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

A fast CLI that checks your Python project dependencies against PyPI and reports which ones have newer versions
available. Inspired by [npm-check-updates](https://github.com/raineorshine/npm-check-updates).

![demo](assets/demo.gif)

```
fastapi        >=0.109.0  →  >=0.135.1
pydantic       >=1.10.0   →  >=2.12.5
uvicorn        >=0.20     →  >=0.34.0

3 packages can be updated.
```

## Why I built this

Every week I'd open my Python projects, manually scan through `pyproject.toml`, look up each package on PyPI, and check
whether I was falling behind. It was tedious, easy to miss something, and I kept thinking there has to be a better way.

There were existing tools, but none of them fit quite how I worked. I'd been spoiled
by [npm-check-updates](https://github.com/raineorshine/npm-check-updates) in the Node world: one command, instant table,
upgrade in place if you want. I wanted exactly that, but for Python something fast, dependency-file-aware, and with
in-place upgrades that actually respected my version constraints.

So I built pycu.

## Features

- Supports `pyproject.toml` (PEP 621 / uv), `pyproject.toml` (Poetry), and `requirements.txt`
- Concurrent PyPI lookups with configurable parallelism
- In-place upgrades with `--upgrade`
- Filter by bump level: major, minor, or patch only
- JSON output for scripting
- Self-updates via `--self-update`
- Color-coded output showing exactly which version component changed
- SHA-256 verified self-update downloads

## Installation

### Linux / macOS

```sh
curl -fsSL https://raw.githubusercontent.com/Logic-py/python-check-updates/main/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Logic-py/python-check-updates/main/install.ps1 | iex
```

### Manual download

Download the binary for your platform from
the [latest release](https://github.com/Logic-py/python-check-updates/releases/latest), extract it, and place it
somewhere on your `PATH`.

### From source

Requires [Rust](https://rustup.rs) 1.85 or later (edition 2024).

```sh
cargo install --git https://github.com/Logic-py/python-check-updates
```

## Uninstall

```sh
pycu --uninstall
```

## Usage

Run in a directory that contains a `pyproject.toml` or `requirements.txt`:

```sh
pycu
```

Or point to a specific file:

```sh
pycu --file path/to/requirements.txt
```

### Options

| Flag                | Short | Description                                                      |
|---------------------|-------|------------------------------------------------------------------|
| `--file <PATH>`     |       | Path to the dependency file (auto-detected if omitted)           |
| `--upgrade`         | `-u`  | Rewrite the file in-place with updated constraints               |
| `--target <LEVEL>`  | `-t`  | Only show `major`, `minor`, or `patch` bumps (default: `latest`) |
| `--json`            |       | Output results as JSON                                           |
| `--concurrency <N>` |       | Max concurrent PyPI requests (default: `10`)                     |
| `--self-update`     |       | Update pycu itself to the latest release                         |
| `--uninstall`       |       | Remove pycu from your system                                     |
| `--version`         |       | Print the version                                                |

### Examples

```sh
# Check all dependencies
pycu

# Upgrade the file in-place
pycu --upgrade

# Only show minor-level updates
pycu --target minor

# Check a specific requirements file
pycu --file requirements-dev.txt

# Machine-readable output
pycu --json
```

### JSON output

```json
[
  {
    "name": "fastapi",
    "current": ">=0.109.0",
    "latest": "0.135.1"
  }
]
```

## Supported formats

### pyproject.toml - PEP 621 / uv

```toml
[project]
dependencies = [
    "fastapi>=0.109.0",
    "pydantic>=1.10.0,<2.0.0",
]

[project.optional-dependencies]
dev = [
    "pytest~=7.3.0",
]

[dependency-groups]
dev = [
    "mypy>=0.19.1,<2.0.0",
]
```

### pyproject.toml - Poetry

```toml
[tool.poetry.dependencies]
fastapi = "^0.109.0"

[tool.poetry.group.dev.dependencies]
pytest = "~7.3.0"
```

### requirements.txt

```text
fastapi>=0.109.0
pydantic>=1.10.0,<2.0.0
pytest~=7.3.0  # dev
```

## Roadmap

- **Private registry support** - planned support for checking dependencies hosted on private PyPI-compatible
  registries (e.g. Artifact Registry, etc.)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
