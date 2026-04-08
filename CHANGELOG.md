# Changelog

All notable changes to linear-cli will be documented in this file.

Versioning follows [CalVer](https://calver.org/) with format `YYYY.MM.PATCH`.

## [2026.4.2] — 2026-04-08

### Added
- `lin config set-token` / `get-token` / `path` for persistent API key management
- `lin completions` for shell completions (bash, zsh, fish, powershell)
- `--state`, `--label`, `--priority` filter flags on `lin issues list`
- `--label` can be specified multiple times to match any
- Pre-built binaries for Homebrew (no Rust compile required)

### Changed
- API key resolution order: env var > config file > .env file
- Upgraded to Rust 2024 edition

### Fixed
- Clippy warnings under Rust 2024 edition

## [2026.4.1] — 2026-04-08

### Changed
- Renamed crate to `lincli` for crates.io publishing
- Updated all repo references to `aaronkwhite/linear-cli`
- Added CI and release workflows

## [2026.4.0] — 2026-04-01

### Added
- Initial Rust port of lin CLI
- Full parity with Python version: 11 command groups, 67 subcommands
- Type-safe GraphQL queries via cynic
- Rich terminal output with colors, tables, and interactive prompts
- Single static binary distribution (no runtime dependencies)
