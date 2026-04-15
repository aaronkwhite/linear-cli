# Changelog

All notable changes to linear-cli will be documented in this file.

Versioning follows [CalVer](https://calver.org/) with format `YYYY.MM.PATCH`.

## [2026.4.16] — 2026-04-14

### Added
- `lin issues list --all-teams` — query issues across all teams
- `lin issues list --created-after <DATE>` / `--updated-after <DATE>` — date filters
- `lin issues create --description-file <PATH>` — read description from a file
- `lin issues update --description-file <PATH>` — update description from a file
- `lin issues comment --body-file <PATH>` — read comment body from a file
- `lin issues start <ID>` — create/switch to the git branch Linear suggests for an issue
- `lin issues pr <ID>` — create a GitHub PR linked to an issue (requires `gh`)
- `lin auth login/list/default/whoami` — multi-workspace authentication
- `--workspace <NAME>` global flag — override the default workspace for a single command

### Changed
- Config format now supports named workspaces; legacy `[auth]` format auto-migrates
- `issues comment` body is now optional when using `--body-file`

## [2026.4.14] — 2026-04-14

### Added
- `lin api '<query>'` — raw GraphQL passthrough with auth injected automatically
- `lin issues update --team <TEAM>` — move issues between teams
- `lin projects update --add-team <TEAM>` — add a team to a project
- `lin cycles add --cycle <id>` — add issue to a specific cycle, not just the active one

### Fixed
- `lin projects get` now accepts UUID and URL slugs in addition to project names
- `lin projects issues --json` and `lin projects search --json` now emit consistent `{"issues": [...]}` and `{"projects": [...]}` shapes

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
