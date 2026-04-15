# schpet-inspired Features Design Spec

**Goal:** Bring the best developer-workflow ideas from schpet/linear-cli into lin, shipped across four incremental buckets.

**Execution order:** C (query power) -> D (content input) -> A (developer workflow) -> B (multi-workspace). Each bucket ships independently with its own commits and tests.

---

## Bucket C: Query Power

Add filtering flags to `issues list` for date-based and cross-team queries.

### New flags on `issues list`

| Flag | Type | Maps to |
|------|------|---------|
| `--created-after <DATE>` | `String` (YYYY-MM-DD) | `IssueFilter.createdAt: { gte: $date }` |
| `--updated-after <DATE>` | `String` (YYYY-MM-DD) | `IssueFilter.updatedAt: { gte: $date }` |
| `--all-teams` | `bool` | Omits team filter; queries all teams. Conflicts with `--team`. |

### Behavior

- Date flags add to the existing filter object. They compose with `--team`, `--state`, `--label`, etc.
- `--all-teams` and `--team` are mutually exclusive. If both are provided, exit with a clap error.
- `--all-teams` without other filters returns issues across all teams, ordered by `updatedAt`.
- Dates are passed as ISO strings directly to the GraphQL filter.

### Files changed

- `src/commands/issues.rs` — add fields to `List` variant, build filter conditionally
- `tests/cli_smoke.rs` — parse tests for new flags
- `tests/integration.rs` — integration test with `--created-after` and `--all-teams`

---

## Bucket D: Content Input from Files

Allow descriptions and comment bodies to be read from files. Essential for AI agents writing markdown content.

### New flags

| Command | Flag | Conflicts with |
|---------|------|----------------|
| `issues create` | `--description-file <PATH>` | `--description` |
| `issues update` | `--description-file <PATH>` | (no existing `--description` on update, so no conflict) |
| `issues comment` | `--body-file <PATH>` | (no existing `--body` flag; `body` is positional) |

### Behavior

- `--description-file` reads the file with `std::fs::read_to_string` and uses contents as the description.
- For `issues create`: if both `--description` and `--description-file` are provided, error via clap conflict.
- For `issues comment`: if `body` positional arg is `-` AND `--body-file` is provided, the file wins. Otherwise, if `body` is a real string, it's used directly. Actually simpler: make `body` optional on `comment`, add `--body-file`, require exactly one.
- File not found or read error -> anyhow error with path in message.

### Files changed

- `src/commands/issues.rs` — add `--description-file` to Create/Update, `--body-file` to Comment
- `tests/cli_smoke.rs` — parse tests

---

## Bucket A: Developer Workflow

Two new subcommands that bridge Linear issues with git/GitHub.

### `issues start <IDENTIFIER>`

1. Fetch issue from Linear API — needs `branchName` and `state.name`
2. Create and switch to git branch: `git checkout -b <branchName>`
3. If `--status <STATUS>` is provided, update the issue status (e.g., "In Progress")
4. Print: `Switched to branch <branchName>` (or `Branch <branchName> already exists, switched to it` if it exists)
5. JSON mode: emit `{"branch": "<branchName>", "identifier": "ENG-123"}`

**Edge cases:**
- Branch already exists locally: `git checkout <branchName>` (no `-b`)
- Not in a git repo: error with "Not a git repository"
- Dirty working tree: let git handle it (it will error on checkout if conflicts)

**Flags:**
- `--status <STATUS>` — optional, update issue status after branching
- `--print-only` — just print the branch name, don't run git commands (for scripting: `` git checkout -b $(lin issues start ENG-123 --print-only) ``)

### `issues pr <IDENTIFIER>`

1. Fetch issue title, URL, identifier from Linear API
2. Check `gh` is in PATH — if not, error: "gh CLI required for pr command (https://cli.github.com)"
3. Run: `gh pr create --title "<identifier>: <title>" --body "Resolves <issue_url>" --fill`
4. Pass through gh's stdout/stderr
5. JSON mode: capture gh output

**Flags:**
- `--draft` — passes `--draft` to gh
- `--base <BRANCH>` — passes `--base` to gh

### Implementation

- Both use `std::process::Command` for git/gh — no new Rust dependencies
- Non-interactive detection: `issues start` always runs (git is non-interactive). `issues pr` always runs (gh handles its own interactivity).

### Files changed

- `src/commands/issues.rs` — add `Start` and `Pr` variants + handlers
- `tests/cli_smoke.rs` — parse tests (can't integration-test git operations easily)

---

## Bucket B: Multi-Workspace

Support multiple Linear workspaces with named profiles.

### Config format

New `~/.config/lin/config.toml` format:

```toml
default_workspace = "my-company"

[workspaces.my-company]
api_key = "lin_api_..."

[workspaces.other-org]
api_key = "lin_api_..."
```

### Migration

Old format:
```toml
[auth]
api_key = "lin_api_..."
```

Auto-migrates on first load: old key becomes workspace named "default". The `[auth]` section is removed and replaced with the new format. Migration is transparent — no user action needed.

### New commands: `lin auth`

| Command | Behavior |
|---------|----------|
| `auth login` | Prompt for workspace name + API key, save to config |
| `auth list` | List workspaces, mark default with `*` |
| `auth default <name>` | Set default workspace |
| `auth whoami` | Show current workspace name + user (calls `viewer` query) |

### Global flag

`--workspace <name>` — overrides the default workspace for a single command. Added to `Cli` struct.

### API key resolution (updated)

1. `--workspace <name>` flag -> look up in config
2. `LINEAR_API_KEY` env var (always wins if set)
3. Config file default workspace
4. `.env` / `.env.local` files

### Files changed

- `src/config.rs` — new `WorkspaceConfig` struct, migration logic, workspace CRUD
- `src/client/mod.rs` — update `resolve_api_key` to accept workspace name
- `src/cli.rs` — add `--workspace` global flag, add `Auth` command
- `src/commands/auth.rs` — new file, login/list/default/whoami handlers
- `src/commands/mod.rs` — register auth module
- `src/main.rs` — wire up Auth dispatch, pass workspace to client

### No keyring

API keys stored in plaintext config file, same as `gh`, `aws`, `gcloud`. Keyring support (macOS Keychain, libsecret) would add platform-specific dependencies. Not worth the complexity for a developer CLI.

---

## Testing strategy

Each bucket adds:
- **Smoke tests** (cli_smoke.rs) — flag parsing, help text, mutual exclusion
- **Unit tests** — config migration (Bucket B), file reading (Bucket D)
- **Integration tests** — where safe (read-only queries with date filters, `auth whoami`)

Mutation-heavy commands (`issues start`, `issues pr`) are tested via smoke tests only — they require git state or `gh` which aren't available in CI.

---

## Out of scope

- Keyring/secure credential storage
- `jj` (Jujutsu) support — git only
- `linear schema` command — `lin api` with introspection query covers this
- Project-level `.linear.toml` config — workspace config is sufficient
