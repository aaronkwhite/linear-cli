# lin — Linear CLI

[![CI](https://github.com/aaronkwhite/lin-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/aaronkwhite/lin-cli/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/lin-cli.svg)](https://crates.io/crates/lin-cli)
[![Downloads](https://img.shields.io/crates/d/lin-cli.svg)](https://crates.io/crates/lin-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast, native CLI for [Linear](https://linear.app). Manage issues, projects, cycles, and more from your terminal.

## Install

### Homebrew (macOS)

```bash
brew install aaronkwhite/tap/lin
```

### Cargo

```bash
cargo install lin-cli
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/aaronkwhite/lin-cli/releases) — available for macOS (Intel & Apple Silicon) and Linux (x86 & ARM).

### From Source

```bash
git clone https://github.com/aaronkwhite/lin-cli.git
cd lin-cli
cargo install --path .
```

## Setup

Get your API key from **Linear Settings > API > Personal API keys**, then:

```bash
export LINEAR_API_KEY="lin_api_..."
```

Or create a `.env` file:

```
LINEAR_API_KEY=lin_api_...
```

## Usage

```bash
lin --help                    # Show all commands
lin issues list --team ENG    # List issues for a team
lin issues get ENG-123        # Get issue details
lin projects list             # List projects
lin teams list                # List teams
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output raw JSON for scripting |
| `--debug` | Print GraphQL queries/responses to stderr |
| `--version` | Show version |

## Commands

| Group | Description |
|-------|-------------|
| `issues` | List, create, update, search, comment, archive, branch lookup |
| `projects` | List, create, update, search, archive, delete projects |
| `cycles` | List, create, manage, archive cycles and cycle issues |
| `initiatives` | Manage initiatives, status updates, link projects |
| `roadmap` | Project updates and milestones |
| `labels` | Create, manage, apply labels |
| `teams` | List teams, members, states, workload |
| `relations` | Issue dependencies and relations |
| `customers` | Customer management, needs, tiers |
| `views` | Custom views and their issues |
| `docs` | Documents: create, search, manage |
| `notifications` | View, read, archive, snooze notifications |
| `me` | Show authenticated user info |
| `attachments` | Manage issue attachments and links |
| `search` | Search across issues, projects, and documents |

## Why CLI over MCP?

| | CLI | MCP |
|---|---|---|
| Tokens per operation | ~1,300 | ~44,000 |
| Reliability | 100% | ~72% |
| Dependencies | Single binary | Schema injection |

## Development

```bash
cargo build                           # Build
cargo test                            # Run tests (smoke + unit)
LINEAR_API_KEY=... cargo test -- --ignored  # Run integration tests
```

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT
