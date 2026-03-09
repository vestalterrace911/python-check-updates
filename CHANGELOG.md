# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-03-09

### Added

- Five built-in color schemes for table output, selectable and persisted per user:
  - `default` - GitHub-style SemVer severity (`#D73A49` / `#0366D6` / `#28A745`)
  - `okabe-ito` - Color-blind safe Okabe–Ito palette (`#E69F00` / `#0072B2` / `#009E73`)
  - `traffic-light` - Classic red/yellow/green (`#E74C3C` / `#F1C40F` / `#2ECC71`)
  - `severity` - Monitoring/observability style (`#8E44AD` / `#3498DB` / `#95A5A6`)
  - `high-contrast` - Maximum distinction, color-blind safe (`#CC79A7` / `#0072B2` / `#F0E442`)
- All colors rendered with true-color (24-bit) escape codes for exact hex fidelity
- `--set-color-scheme` flag: run without a value to preview all schemes visually, or pass a scheme name to save it permanently
- Color scheme preference persisted to `~/.config/pycu/config.toml` (Linux/macOS) or `%APPDATA%\pycu\config.toml` (Windows)
- First-run interactive prompt to choose a color scheme on initial install
- `--uninstall` now also removes the `pycu/` config directory

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

[Unreleased]: https://github.com/Logic-py/python-check-updates/compare/1.1.0...HEAD

[1.1.0]: https://github.com/Logic-py/python-check-updates/compare/1.0.0...1.1.0
[1.0.0]: https://github.com/Logic-py/python-check-updates/releases/tag/1.0.0
