# lin-cli Open Source Readiness Design Spec

**Date**: 2026-04-02
**Status**: Approved
**GitHub**: aaronkwhite/lin-cli
**Distribution**: crates.io + Homebrew tap + GitHub releases

## Goals

1. Clean code quality — zero warnings on `cargo fmt`, `cargo clippy`, `cargo build`
2. Complete crates.io metadata for discoverability
3. Community files for contributor onboarding
4. CI/CD pipeline: test on push, release on tag
5. Binary distribution: GitHub releases (4 targets) + Homebrew tap
6. Updated README with badges and install methods

## 1. Code Quality Cleanup

- Run `cargo fmt` to fix all formatting violations
- Run `cargo clippy --fix` to resolve auto-fixable warnings
- Remove dead code:
  - `src/graphql/issues.rs` — unused cynic query structs (IssueById, Issue, IssueByIdVariables)
  - `src/graphql/mod.rs` — unused scalar types (DateTime, TimelessDate, JSONObject, JSON, Duration, UUID, DateTimeOrDuration, TimelessDateOrDuration) — keep only if needed for future typed queries
  - `src/output/interactive.rs` — unused `multi_select()` function
  - `src/output/table.rs` — unused `print_table_with_status()` function
  - `src/output/detail.rs` — unused `cyan` import
  - `src/client/cache.rs` — `email` and `active` fields on CachedUser, `state_type` on CachedState (keep if used for deserialization, suppress warning if needed)
- Target: zero compiler warnings

## 2. Cargo.toml Metadata

```toml
[package]
name = "lin-cli"
version = "2026.4.0"
edition = "2021"
description = "Linear CLI — manage issues, projects, cycles, and more from the terminal"
license = "MIT"
repository = "https://github.com/aaronkwhite/lin-cli"
homepage = "https://github.com/aaronkwhite/lin-cli"
authors = ["Aaron K White"]
readme = "README.md"
keywords = ["cli", "linear", "graphql", "issue-tracking"]
categories = ["command-line-utilities", "development-tools"]
exclude = ["docs/superpowers/", ".github/"]
```

Note: `schemas/linear.graphql` must NOT be excluded — it's needed at compile time by build.rs.

## 3. Community Files

### CONTRIBUTING.md
- Development setup: clone, `cargo build`, `cargo test`
- Code style: `cargo fmt` and `cargo clippy -- -D warnings` must pass
- Testing: new commands need smoke tests in `tests/cli_smoke.rs`, API-touching code needs integration tests
- PR process: fork, branch from `main`, PR with description, CI must pass
- Issue labels: `bug`, `feature`, `docs`, `good first issue`

### CODE_OF_CONDUCT.md
- Contributor Covenant v2.1

### SECURITY.md
- Directs vulnerability reports to private email, not public issues
- Expected response timeline

### .github/ISSUE_TEMPLATE/bug.md
- Description, steps to reproduce, expected vs actual behavior
- `lin --version` output, OS, shell

### .github/ISSUE_TEMPLATE/feature.md
- Use case, proposed solution, alternatives considered

### .github/PULL_REQUEST_TEMPLATE.md
- Checklist: description, tests, `cargo fmt` + `cargo clippy` clean, CHANGELOG updated

## 4. CI/CD — GitHub Actions

### .github/workflows/ci.yml
- Triggers: push to `main`, pull requests to `main`
- Matrix: `ubuntu-latest`, `macos-latest`
- Steps: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`

### .github/workflows/release.yml
- Triggers: tag push matching `v*`
- Build matrix (4 targets):
  - `x86_64-unknown-linux-musl` (static Linux x86)
  - `aarch64-unknown-linux-musl` (static Linux ARM)
  - `x86_64-apple-darwin` (macOS Intel)
  - `aarch64-apple-darwin` (macOS Apple Silicon)
- Creates GitHub Release with all 4 binaries attached
- Publishes to crates.io using `CARGO_REGISTRY_TOKEN` secret
- Generates Homebrew formula and pushes to tap repo

### .github/workflows/audit.yml
- Triggers: weekly cron (Sunday)
- Runs `cargo audit` for dependency vulnerability scanning

## 5. Homebrew Tap

Separate repository: `aaronkwhite/homebrew-tap`

The release workflow generates a formula that:
- Downloads the correct binary for the user's platform (macOS Intel or Apple Silicon)
- Installs to the Homebrew prefix

User install: `brew install aaronkwhite/tap/lin`

The formula is auto-generated and pushed by the release workflow. No manual formula maintenance.

## 6. README Updates

Add to top of README.md:
- Badges: CI status, crates.io version, crates.io downloads, license
- Install methods section expanded:
  - Homebrew: `brew install aaronkwhite/tap/lin`
  - Cargo: `cargo install lin-cli`
  - From source: `cargo install --path .`
  - Pre-built binaries: link to GitHub releases
- Link to CONTRIBUTING.md
- Link to CHANGELOG.md
