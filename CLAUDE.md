# lin-cli

Rust CLI for Linear project management. Binary is `lin`. Wraps the Linear GraphQL API directly — no SDK.

## Build & Test

```bash
cargo build                    # dev build
cargo build --release          # release build
cargo test                     # unit + query validation tests
cargo run -- <command>         # run without installing
```

Tests in `tests/query_validation.rs` validate all embedded GraphQL queries against `schemas/linear.graphql`. Run them after any query change.

## Architecture

```
src/
  main.rs           # CLI entry, global --json / --debug flags
  client/mod.rs     # LinearClient: HTTP, auth, retry, cache
  client/cache.rs   # In-memory cache for teams/states/users
  commands/         # One file per noun (issues, projects, teams, ...)
  output/mod.rs     # print_json() — all JSON output goes through here
  output/detail.rs  # Human-readable issue/project detail views
  output/table.rs   # ASCII table printer
  output/interactive.rs  # fuzzy-select prompts (skipped when non-interactive)
  error.rs          # LinearError enum
  config.rs         # ~/.config/lin/config.toml
schemas/
  linear.graphql    # Full Linear schema snapshot — used for query validation
```

## Key Patterns

**GraphQL queries** are raw strings embedded in command files. Variables use `serde_json::json!()`. All queries are validated against the schema in CI via `tests/query_validation.rs`.

**JSON output** — `print_json()` in `output/mod.rs` handles all `--json` output. It strips the GraphQL `{"data": {...}}` envelope and emits compact JSON. Never call `serde_json::to_string_pretty` for user output.

**Non-interactive detection** — use `crate::output::interactive::is_interactive()` before any prompt. Prompts must be skipped and operations must auto-confirm when not interactive (agent/pipe use).

**Type rules for GraphQL variables:**
- Filter fields (`{ team: { id: { eq: $id } } }`) → `ID!`
- Direct lookup fields (`team(id: $id)`, `issue(id: $id)`) → `String!`
- Linear accepts both UUIDs and identifiers (e.g. `ENG-123`) for `String!` lookup fields

**API key resolution order:** `LINEAR_API_KEY` env var → `~/.config/lin/config.toml` → `.env` / `.env.local` in cwd or parents.

## Adding a Command

1. Add a new file `src/commands/<noun>.rs` with a `pub async fn execute()` 
2. Register in `src/commands/mod.rs`
3. Add the subcommand to the Clap enum in `src/main.rs`
4. Use `client.query_raw(query, Some(variables)).await?` for queries
5. Branch on `json` flag: `if json { crate::output::print_json(&result) } else { /* human output */ }`

## Schema Updates

To update `schemas/linear.graphql`, fetch the introspection schema from `https://api.linear.app/graphql` and convert to SDL. The query validation tests will catch any type mismatches in embedded queries.
