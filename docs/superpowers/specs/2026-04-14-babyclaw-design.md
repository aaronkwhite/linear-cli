# babyclaw — Design Spec

Date: 2026-04-14  
Branch: feature/babyclaw  
Status: Approved

## Background

User feedback identified six gaps where `lin` forced users to drop down to raw GraphQL. This spec covers all six.

---

## Item 1: `projects get` — Accept UUID and slug in addition to name

**Problem:** `get_project_id` does a fuzzy name match only. UUIDs and URL slugs (e.g. `my-project-abc123` from shared Linear links) fail with "Project not found."

**Solution:** Three-step resolution in `get_project_id`:
1. If input matches UUID format (`looks_like_uuid()` in `util.rs`) → return directly, no API call
2. Try fuzzy name match against cache (existing behavior)
3. If not found → query `projects(filter: { slugId: { eq: $input } })` as fallback

No CLI changes. `lin projects get <value>` accepts any of the three formats transparently.

**Schema:** `ProjectFilter.slugId: StringComparator` — confirmed in `schemas/linear.graphql`.

---

## Item 2: `lin projects update --add-team <TEAM>`

**Problem:** `projectUpdate` mutation supports `teamIds: [String!]` but the CLI has no way to add a team.

**Solution:** Add `--add-team <TEAM>` flag to `ProjectsCommand::Update`. Implementation:
1. Fetch the project's current `teams { nodes { id } }` 
2. Resolve the new team name → ID via `get_team_id`
3. Merge IDs, pass as `teamIds` in the `projectUpdate` input

Named `--add-team` not `--team` to make additive intent explicit and avoid confusion with `--team` filters on other commands.

---

## Item 3: `lin cycles add --cycle <ID>`

**Problem:** `cycles add` always fetches the active cycle. No way to target an upcoming or future cycle.

**Solution:** Add optional `--cycle <ID>` flag to `CyclesCommand::Add`. Make `--team` optional when `--cycle` is provided.

Logic:
- `--cycle` provided → use it directly, skip active cycle lookup
- `--team` only → existing behavior (look up active cycle for that team)
- Neither → error: "Provide --team (uses active cycle) or --cycle <id>"

---

## Item 4: `lin issues update --team <TEAM>`

**Problem:** `issueUpdate` mutation accepts `teamId` for moving issues between teams, but the CLI doesn't expose it.

**Solution:** Add `--team <TEAM>` flag to `IssuesCommand::Update`. Resolve team name → ID via `get_team_id`, pass as `teamId` in the mutation input. Straightforward.

---

## Item 5: Normalize JSON output nesting

**Problem:** Two commands have inconsistent shapes after the `data` envelope is stripped:
- `projects issues <name>` returns `{"issues": {"nodes": [...]}}` (double-wrapped)  
- `projects search <query>` returns `{"searchProjects": {"nodes": [...]}}`

**Solution:** In the `--json` branch of each:
- `projects issues` → extract `nodes` array, emit `{"issues": [...]}`
- `projects search` → extract `nodes` array, emit `{"projects": [...]}`

Consistent with how other list commands behave after envelope stripping.

---

## Item 6: `lin api` — GraphQL passthrough

**Problem:** Any gap in lin's command surface forces users to dig out their API key and use curl manually.

**Solution:** New `api` subcommand. 

```
lin api '<query or mutation>'
lin api '<query>' --variables '{"key": "value"}'
```

Implementation:
- New `src/commands/api.rs` with `ApiArgs { query: String, variables: Option<String> }`
- Calls `client.query_raw(query, variables)` directly
- Output always goes through `print_json` (gets envelope stripping for free)
- `--variables` accepts a JSON string, parsed into `serde_json::Value`
- No `--mutation` flag needed — Linear's API handles both query and mutation strings identically; the string itself declares which it is

**Error handling:** If `--variables` is not valid JSON, return a clear error before hitting the API.

---

## Files Changed

| File | Change |
|------|--------|
| `src/client/mod.rs` | UUID/slug fallback in `get_project_id` |
| `src/client/cache.rs` | No change |
| `src/commands/projects.rs` | `--add-team` on Update; normalize Issues and Search JSON output |
| `src/commands/cycles.rs` | Optional `--cycle` on Add |
| `src/commands/issues.rs` | `--team` on Update |
| `src/commands/api.rs` | New file |
| `src/commands/mod.rs` | Register `api` module |
| `src/main.rs` | Add `Api` subcommand variant |
| `.claude/skills/lin/SKILL.md` | Update quick reference |
| `.claude/skills/lin/references/commands.md` | Document new flags and `api` command |
