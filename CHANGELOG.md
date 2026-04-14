# Changelog

All notable changes to linear-cli will be documented in this file.

Versioning follows [CalVer](https://calver.org/) with format `YYYY.MM.PATCH`.

## [2026.4.12] — 2026-04-12

### Added
- Claude Code skill at `.claude/skills/lin/` — teaches Claude how to use `lin` for Linear tasks, with full command reference and MCP fallback guide
- `CLAUDE.md` — repo-level agent reference for contributors working on the codebase

### Changed
- `--json` output is now compact (no indentation) — reduces token cost for AI agent use
- `--json` output strips the GraphQL `{"data": {...}}` envelope — payloads are returned directly

### Fixed
- Security audit CI workflow now has `issues: write` permission so it can file advisories
- Updated `rand` to v0.9.3 (resolves RUSTSEC-2026-0097, unsound advisory)
- Unicode-safe truncation in GraphQL query validation test helper

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
