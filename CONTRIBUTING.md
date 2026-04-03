# Contributing to lin

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

```bash
git clone https://github.com/aaronkwhite/lin-cli.git
cd lin-cli
cargo build
cargo test
```

You'll need a [Linear API key](https://linear.app/settings/api) to run integration tests:

```bash
export LINEAR_API_KEY="lin_api_..."
cargo test --test integration -- --ignored
```

## Code Style

All code must pass these checks (CI enforces them):

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

Run `cargo fmt` before committing. If clippy flags something, fix it rather than suppressing it unless there's a documented reason.

## Testing

- **New commands**: add smoke tests in `tests/cli_smoke.rs` (no API key needed)
- **API-touching code**: add integration tests in `tests/integration.rs` (gated on `LINEAR_API_KEY`)
- **Unit tests**: add inline `#[cfg(test)]` modules for pure logic (cache lookups, formatting, etc.)

## Pull Request Process

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Ensure `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` all pass
4. Update `CHANGELOG.md` if your change is user-facing
5. Open a PR with a clear description of what and why

## Issues

- **Bugs**: use the bug report template with `lin --version`, OS, and steps to reproduce
- **Features**: use the feature request template and describe the use case, not just the solution
