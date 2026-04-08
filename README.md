# lin — Linear CLI

[![CI](https://github.com/aaronkwhite/linear-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/aaronkwhite/linear-cli/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/lincli.svg)](https://crates.io/crates/lincli)
[![Downloads](https://img.shields.io/crates/d/lincli.svg)](https://crates.io/crates/lincli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast, native CLI for [Linear](https://linear.app). Manage issues, projects, cycles, and more from your terminal.

## Install

### Homebrew (macOS)

```bash
brew install aaronkwhite/tap/lin
```

### Cargo

```bash
cargo install lincli
```

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/aaronkwhite/linear-cli/releases) — available for macOS (Intel & Apple Silicon) and Linux (x86 & ARM).

### From Source

```bash
git clone https://github.com/aaronkwhite/linear-cli.git
cd linear-cli
cargo install --path .
```

## Setup

Get your API key from **Linear Settings > API > Personal API keys**, then:

```bash
lin config set-token
```

Or set it via environment variable:

```bash
export LINEAR_API_KEY="lin_api_..."
```

## Usage

```bash
lin --help                                  # Show all commands
lin issues list --team ENG                  # List issues for a team
lin issues list --state "In Progress"       # Filter by state
lin issues list --label bug --label urgent  # Filter by labels
lin issues get ENG-123                      # Get issue details
lin projects list                           # List projects
lin teams list                              # List teams
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
| `config` | Manage API key and CLI configuration |
| `completions` | Generate shell completions (bash, zsh, fish, powershell) |

## Using with Claude Code

Add this to your project's `CLAUDE.md` to give Claude Code access to Linear:

```markdown
## Linear

Use the `lin` CLI for all Linear operations. Always use `--json` for structured output.

Examples:
- `lin issues list --team ENG --json` — list issues
- `lin issues list --state "In Progress" --json` — filter by state
- `lin issues get ENG-123 --json` — get issue details
- `lin issues create --team ENG --title "Title" --json` — create issue
- `lin issues update ENG-123 --status "Done" --json` — update status
- `lin issues comment ENG-123 "comment body" --json` — add comment
- `lin search "query" --json` — search across issues, projects, docs
- `lin projects list --json` — list projects
- `lin teams list --json` — list teams
- `lin me --json` — current user info

Run `lin <command> --help` for full flag details.
```

## Why CLI over MCP?

MCP servers inject the full JSON schema for every tool into the LLM's context window on every message — whether those tools are used or not. This creates a permanent tax on every interaction.

### Token Cost

| Metric | CLI | MCP | Difference |
|---|---|---|---|
| Idle context cost | 0 tokens | ~19,659 tokens | CLI loads nothing upfront |
| Simple query (e.g., repo info) | ~1,365 tokens | ~44,026 tokens | **32x** more expensive |
| PR details + review | ~1,648 tokens | ~32,279 tokens | **20x** more expensive |
| Complex multi-step task | ~8,750 tokens | ~37,402 tokens | **4x** more expensive |
| Monthly cost @ 10K ops | ~$3.20 | ~$55.20 | **17x** more expensive |

*Source: [Scalekit benchmark](https://www.scalekit.com/blog/mcp-vs-cli-use), 75 runs, Claude Sonnet 4, p < 0.05*

### Why the Gap Is So Large

MCP loads the entire API surface into context on every message. Linear's official MCP server injects 22 tool schemas (~19,659 tokens) before you ask a single question. Connect 3 MCP servers and you can lose **72% of your context window at idle**.

A CLI pays only for what it uses. `lin issues list --team ENG --json` costs ~15 tokens for the command string. The other 200+ subcommands cost nothing because they're never loaded.

### Reliability

| | CLI | MCP |
|---|---|---|
| Success rate | 100% | 72% |
| Failure modes | Exit code + stderr | TCP timeouts, schema injection failures, transport errors, silent tool dropping |
| Dependencies | Single binary in PATH | JSON-RPC, transport negotiation, schema validation |

*Source: [Scalekit benchmark](https://www.scalekit.com/blog/mcp-vs-cli-use), 25 runs per method*

### Industry Trend

- **Perplexity** dropped MCP internally, citing context window waste and authentication issues ([source](https://nevo.systems/blogs/news/perplexity-drops-mcp-protocol-72-percent-context-window-waste))
- **mcp2cli** (wraps MCP servers as CLIs) measured **96-99% token reduction** ([source](https://blog.orangecountyai.com/mcp2cli-one-cli-for-every-api-zero-wasted-tokens/))
- "MCP is dead, long live the CLI" hit the Hacker News front page ([discussion](https://news.ycombinator.com/item?id=47208398))

### When MCP Still Makes Sense

MCP works well for non-technical users in chat interfaces where CLI access isn't available, or for services that don't have a CLI wrapper. For developer tooling where you have shell access, CLI wins on every axis.

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
