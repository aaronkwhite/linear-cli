# lin-cli: Rust Port Design Spec

**Date**: 2026-04-01
**Status**: Approved
**Ported from**: [linear-tools](~/src/linear-tools) (Python/Click)

## Goals

1. **Performance** — near-instant startup, fast API interactions via async HTTP
2. **Distribution** — single static binary, no runtime dependencies
3. **Full API parity** — all 11 command groups, 67 subcommands from the Python CLI
4. **Type safety** — compile-time validated GraphQL queries via cynic
5. **Rich output** — `gh`-style formatting with interactive features

## Versioning

**CalVer: `YYYY.MM.PATCH`** (e.g., `2026.04.0`)

Tracked in `Cargo.toml` version field and `CHANGELOG.md`.

## Project Structure

```
lin-cli/
├── Cargo.toml
├── CHANGELOG.md
├── README.md
├── schema.graphql              # Linear's GraphQL schema (downloaded)
├── src/
│   ├── main.rs                 # Entry point, tokio runtime
│   ├── cli.rs                  # Top-level clap App + global flags (--json, --debug)
│   ├── client/
│   │   ├── mod.rs              # LinearClient: auth, query execution, retry, pagination
│   │   └── cache.rs            # Cached lookups (teams, users, projects, states, labels)
│   ├── graphql/
│   │   ├── mod.rs              # Shared cynic schema registration + common fragments
│   │   ├── issues.rs           # Issue query/mutation structs
│   │   ├── projects.rs         # Project query/mutation structs
│   │   ├── cycles.rs
│   │   ├── roadmap.rs
│   │   ├── labels.rs
│   │   ├── teams.rs
│   │   ├── relations.rs
│   │   ├── customers.rs
│   │   ├── views.rs
│   │   ├── docs.rs
│   │   └── notifications.rs
│   ├── commands/
│   │   ├── mod.rs              # Re-exports all command modules
│   │   ├── issues.rs           # Command handlers (clap structs + execute logic)
│   │   ├── projects.rs
│   │   ├── cycles.rs
│   │   ├── roadmap.rs
│   │   ├── labels.rs
│   │   ├── teams.rs
│   │   ├── relations.rs
│   │   ├── customers.rs
│   │   ├── views.rs
│   │   ├── docs.rs
│   │   └── notifications.rs
│   ├── output/
│   │   ├── mod.rs              # Format dispatch (json vs human vs interactive)
│   │   ├── table.rs            # comfy-table wrappers
│   │   ├── detail.rs           # Key-value detail views
│   │   ├── color.rs            # ANSI colors, NO_COLOR support
│   │   └── interactive.rs      # dialoguer prompts (fuzzy select, confirm)
│   └── error.rs                # LinearError via thiserror, anyhow at boundaries
└── tests/
    ├── cli_smoke.rs            # No API key, test CLI parsing
    └── integration.rs          # Real API tests (gated on LINEAR_API_KEY)
```

### Key decisions

- **`graphql/` separate from `commands/`**: query definitions are data structures, command handlers are business logic. Keeps each file focused and allows fragment reuse across commands.
- **`output/` as its own module**: all formatting logic in one place, easy to add new output modes.
- **`client/cache.rs` split out**: caching logic (teams, users, projects, states, labels) is complex enough to warrant its own file.

## Core Architecture

### LinearClient

```rust
pub struct LinearClient {
    http: reqwest::Client,          // reusable connection pool
    api_key: String,
    debug: bool,
    cache: Cache,
}
```

- **Auth resolution** (priority order):
  1. Explicit `api_key` argument
  2. `LINEAR_API_KEY` environment variable
  3. `.env` in current directory or binary directory
  4. `.env.local` in current directory or binary directory
- **Query execution**: `async fn query<Q: cynic::QueryBuilder>(&self, vars) -> Result<Q::Response>` — generic over any cynic query type, automatic deserialization.
- **Retry**: exponential backoff on HTTP 429 (1s, 2s, 4s), max 3 retries.
- **Pagination**: generic async stream over any connection type using cynic's `pageInfo` + `nodes` pattern. Works across all resource types.
- **Debug mode**: prints GraphQL query text, variables, and response to stderr.

### Cache

```rust
pub struct Cache {
    teams: OnceCell<Vec<Team>>,
    users: OnceCell<Vec<User>>,
    projects: OnceCell<Vec<Project>>,
    states: DashMap<String, Vec<WorkflowState>>,  // keyed by team_key
    labels: DashMap<String, Vec<Label>>,           // keyed by team_key or "__workspace__"
}
```

`OnceCell` for one-time global lookups, `DashMap` for per-key caches. Lazy-fetch pattern — only hits the API when a lookup is first needed.

### Error Handling

```rust
#[derive(thiserror::Error, Debug)]
pub enum LinearError {
    #[error("GraphQL error: {0}")]
    GraphQL(String),
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("API key not found. Set LINEAR_API_KEY or add to .env file")]
    NoApiKey,
    #[error("{0}")]
    NotFound(String),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}
```

Commands use `anyhow::Result` at the boundary for easy `?` propagation. `LinearError` for typed matching when needed.

## CLI Structure

### Global Flags

| Flag | Purpose |
|------|---------|
| `--json` | Output raw JSON for scripting |
| `--debug` | Print GraphQL queries/responses to stderr |
| `--version` | Print CalVer version |

### Command Groups (full parity with Python CLI)

| Group | Subcommands | Count |
|-------|-------------|-------|
| issues | get, list, search, create, update, comment, archive | 7 |
| projects | get, list, issues, create, update | 5 |
| cycles | list, get, issues, create, update, add, remove | 7 |
| roadmap | updates, post, milestones, create-milestone, update-milestone, delete-milestone, initiatives | 7 |
| labels | list, create, update, delete, apply, remove, usage | 7 |
| teams | list, get, members, states, workload | 5 |
| relations | list, blocks, blocked-by, relates, duplicate, remove | 6 |
| customers | list, create, update, delete, link, needs, tiers, create-tier | 8 |
| views | list, get, create, update, delete, issues | 6 |
| docs | list, get, search, create, update, delete | 6 |
| notifications | list, read, archive | 3 |
| **Total** | | **67** |

Each subcommand maps to a clap `#[derive(Args)]` struct with the same options/arguments as the Python version.

## GraphQL Layer (cynic)

### Schema Registration

Download Linear's public GraphQL schema and register it with cynic:

```rust
#[cynic::schema("linear")]
mod schema {}
```

Schema file: `schema.graphql` at project root, referenced in `.graphql_config.yml`.

**Schema acquisition**: Download from Linear's public introspection endpoint using `cynic-cli` (`cynic introspect -u https://api.linear.app/graphql -H "Authorization: <key>" -o schema.graphql`). The schema is committed to the repo so builds don't require an API key.

### Query Pattern

Each `graphql/*.rs` file defines:
1. **Query/Mutation structs** — derive `cynic::QueryFragment`, map to the shape of data needed
2. **Variable structs** — derive `cynic::QueryVariables` for parameterized queries
3. **Filter input types** — derive `cynic::InputObject` for Linear's filter inputs

### Fragment Reuse

Common fragments (e.g., `IssueSummary`, `IssueDetail`) defined in `graphql/mod.rs` and reused across commands that need them.

## Output & Interactivity

### Output Modes

Every command supports three output paths:

1. **Human-readable (default)**: rich formatting with colors, tables, detail views
2. **JSON (`--json`)**: full API response via `serde_json`, for scripting/piping
3. **Interactive (auto-detected)**: when connected to a TTY and a command benefits from it

### Rich Output

- **Tables**: `comfy-table` with box-drawing borders, header styling, column alignment. Priority indicators: `!!!` red, `!!` yellow, `!` cyan.
- **Detail views**: bold labels, indented values, dimmed metadata (dates, IDs).
- **Markdown**: `termimad` for rendering descriptions in the terminal.
- **Status badges**: colored inline labels for issue states, project health, cycle status.
- **`NO_COLOR` / non-TTY**: all ANSI codes stripped automatically via `console` crate.

### Interactive Features (dialoguer)

- **Fuzzy select**: when a required value (e.g., `--team`) is omitted at a TTY, offer fuzzy selection from cached list
- **Confirm prompts**: before destructive actions (archive, delete)
- **Multi-select**: for label application, bulk operations
- Non-TTY: missing required values produce clean errors (no interactive prompts)

## Testing Strategy

### Three tiers

**1. CLI Smoke Tests** (`tests/cli_smoke.rs`)
- No API key required, no network calls
- Uses `assert_cmd` + `predicates`
- Tests:
  - `lin --help` exits 0, lists all 11 command groups
  - `lin --version` prints CalVer version
  - `lin <group> --help` for every group
  - `lin <group> <subcommand> --help` for all 67 subcommands
  - Missing required args produce clean error (no panic/backtrace)
  - `--json` and `--debug` flags accepted at root level
  - Unknown subcommands produce helpful error

**2. Unit Tests** (inline `#[cfg(test)]` modules)
- `client/`: auth resolution (mock env vars, mock `.env` files), retry backoff, pagination cursor handling
- `graphql/`: cynic query structs compile and serialize correctly (partially covered by cynic's compile-time checks)
- `output/`: table formatting, color with/without `NO_COLOR`, detail formatting, JSON serialization
- `cache.rs`: lookup matching (case-insensitive, partial match for users/projects)

**3. Integration Tests** (`tests/integration.rs`)
- Gated behind `LINEAR_API_KEY` env var (skipped if absent)
- Real API calls against Linear workspace
- Core read operations per group (list, get)
- One write+cleanup test per group where safe (create, verify, archive/delete)

### Test Crates

| Crate | Purpose |
|-------|---------|
| `assert_cmd` | CLI binary invocation + assertions |
| `predicates` | Output matching (contains, regex) |
| `wiremock` | HTTP mock server for client unit tests |
| `temp-env` | Scoped env var manipulation for auth tests |

### CI

- Smoke + unit tests: every build, no secrets needed
- Integration tests: separate job, requires `LINEAR_API_KEY` secret

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` (derive) | CLI framework |
| `cynic` | GraphQL code generation from schema |
| `reqwest` (rustls-tls) | HTTP client (no openssl, fully static) |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `thiserror` | Typed error definitions |
| `anyhow` | Error propagation at binary boundary |
| `comfy-table` | Table formatting |
| `console` | ANSI colors, NO_COLOR, TTY detection |
| `dialoguer` | Interactive prompts, fuzzy select |
| `termimad` | Markdown rendering in terminal |
| `dotenvy` | .env file loading |
| `dashmap` | Concurrent per-key cache maps |
| `tokio::sync::OnceCell` | One-time async cache initialization |

Using `reqwest` with `rustls-tls` feature for a fully static binary with no system OpenSSL dependency.
