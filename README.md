# lin — Linear CLI

A fast, native CLI for [Linear](https://linear.app). Manage issues, projects, cycles, and more from your terminal.

## Install

### From source

```bash
cargo install --path .
```

### From crates.io

```bash
cargo install lin-cli
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
| `issues` | List, create, update, search, comment, archive issues |
| `projects` | List, create, update projects and their issues |
| `cycles` | List, create, manage cycles and cycle issues |
| `roadmap` | Project updates, milestones, initiatives |
| `labels` | Create, manage, apply labels |
| `teams` | List teams, members, states, workload |
| `relations` | Issue dependencies and relations |
| `customers` | Customer management, needs, tiers |
| `views` | Custom views and their issues |
| `docs` | Documents: create, search, manage |
| `notifications` | View and manage notifications |

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

## License

MIT
