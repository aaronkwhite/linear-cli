#!/usr/bin/env bash
# Pre-release preflight: run locally before tagging a release.
# Catches every class of failure we've hit in CI/Release.

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

echo "==> Checking working tree is clean"
if [ -n "$(git status --porcelain)" ]; then
  echo "ERROR: uncommitted changes. Commit or stash before releasing."
  git status --short
  exit 1
fi

echo "==> Verifying rust-toolchain.toml is set"
if ! grep -qE '^channel\s*=\s*"' rust-toolchain.toml; then
  echo "ERROR: rust-toolchain.toml missing 'channel' entry."
  exit 1
fi
CHANNEL=$(grep -E '^channel' rust-toolchain.toml | cut -d'"' -f2)
echo "    channel: $CHANNEL"

echo "==> Checking Cargo.toml version matches CHANGELOG.md"
CARGO_VER=$(grep -E '^version\s*=' Cargo.toml | head -1 | cut -d'"' -f2)
if ! grep -qE "^## \[$CARGO_VER\]" CHANGELOG.md; then
  echo "ERROR: CHANGELOG.md missing entry for version $CARGO_VER"
  exit 1
fi
echo "    version: $CARGO_VER"

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo clippy -- -D warnings"
cargo clippy --all-targets -- -D warnings

echo "==> cargo test"
cargo test

echo "==> cargo build --release"
cargo build --release

echo ""
echo "✓ All preflight checks passed."
echo "  Ready to tag v$CARGO_VER and push."
