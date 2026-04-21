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
lin auth login                        # interactive — saves as a named workspace
```

Or set it directly:

```bash
export LINEAR_API_KEY="lin_api_..."    # environment variable (simplest)
lin config set-token                   # save to config file
```

**Multiple workspaces:**

```bash
lin auth login                        # add each workspace by name
lin auth list                         # see configured workspaces
lin auth default myco                 # switch default
lin --workspace other-org issues list # one-off override
```

## Usage

```bash
lin --help                                              # Show all commands
lin issues list --team ENG                              # List issues for a team
lin issues list --state "In Progress"                   # Filter by state
lin issues list --label bug --label urgent              # Filter by labels
lin issues get ENG-123                                  # Get issue details
lin issues update ENG-123 --team HP                     # Move issue to another team
lin projects list                                       # List projects
lin projects update "My Project" --add-team ENG         # Add team to project
lin cycles add ENG-123 --cycle <id>                     # Add issue to specific cycle
lin teams list                                          # List teams
lin issues start ENG-123                                # Create branch from issue
lin issues pr ENG-123 --draft                           # Create draft PR from issue
lin issues list --all-teams --updated-after 2026-04-01  # Cross-team date filter
lin auth login                                          # Add a workspace
lin api '{ viewer { id displayName } }'                 # Raw GraphQL query
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output raw JSON for scripting |
| `--debug` | Print GraphQL queries/responses to stderr |
| `--workspace <NAME>` | Use a specific workspace |
| `--version` | Show version |

## Commands

| Group | Description |
|-------|-------------|
| `auth` | Manage workspaces and authentication |
| `issues` | List, create, update, search, comment, start, pr, archive, branch lookup |
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
| `config` | Manage API key, CLI configuration, and analytics |
| `completions` | Generate shell completions (bash, zsh, fish, powershell) |
| `api` | Raw GraphQL passthrough — any query or mutation with auth injected |

## Using with Claude Code

### Install the skill (recommended)

A Claude Code skill ships with this repo that teaches Claude how to use `lin` for any Linear task — creating issues, querying status, managing projects and cycles, and more.

```bash
cp -r .claude/skills/lin ~/.claude/skills/lin
```

The skill handles tool selection (lin first, Linear MCP as fallback), includes a full command reference, and documents which operations require MCP vs. lin.

### Manual CLAUDE.md snippet

If you prefer a lightweight setup, add this to your project's `CLAUDE.md`:

```markdown
## Linear

Use the `lin` CLI for all Linear operations. Always use `--json` for structured output.

- `lin issues list --team ENG --json` — list issues
- `lin issues get ENG-123 --json` — get issue details
- `lin issues create --team ENG --title "Title" --json` — create issue
- `lin issues update ENG-123 --status "Done" --json` — update status
- `lin search "query" --json` — search across issues, projects, docs

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

## Analytics

lin collects anonymous usage statistics to help improve the tool. This is **opt-out** — enabled by default with a first-run notice.

**What's collected:** command name, flags used (not values), success/failure, execution time, lin version, OS/arch. Each install gets a random anonymous ID.

**What's never collected:** API keys, issue content, workspace names, identifiers, query text, or any personal information.

**Disable:**

```bash
lin config analytics off        # permanent
DO_NOT_TRACK=1 lin ...          # per-invocation (community standard)
```

**Check status:**

```bash
lin config analytics status
```

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
