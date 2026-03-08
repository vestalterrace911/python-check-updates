# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-03-08

### Added

- Check Python dependencies for newer versions on PyPI
- Support for `pyproject.toml` (PEP 621 / uv), `pyproject.toml` (Poetry), and `requirements.txt`
- Support for PEP 735 `[dependency-groups]` (used by uv)
- Support for Poetry 1.2+ group dependencies (`[tool.poetry.group.*.dependencies]`)
- `--upgrade` / `-u` flag to rewrite the dependency file in-place with latest versions
- `--target` / `-t` flag to filter updates by bump level (`major`, `minor`, `patch`)
- `--json` flag for machine-readable output
- `--concurrency` flag to control the number of concurrent PyPI requests (default: 10)
- `--self-update` to update pycu itself to the latest release with SHA-256 checksum verification
- `--uninstall` to remove pycu from the system
- Color-coded output: red for major, blue for minor, green for patch bumps
- Compound constraint handling (`>=old,<bound` updated intelligently)
- Progress bar during PyPI lookups
- Install scripts for Linux/macOS (`install.sh`) and Windows (`install.ps1`)

[Unreleased]: https://github.com/Logic-py/python-check-updates/compare/1.0.0...HEAD

[1.0.0]: https://github.com/Logic-py/python-check-updates/releases/tag/1.0.0
