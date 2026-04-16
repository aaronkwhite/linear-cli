---
name: lin
description: Use when doing anything in Linear — creating or updating issues, checking status, searching, managing projects, cycles, teams, labels, customers, docs, views, notifications, or relations. Triggers on issue identifiers like ENG-123, or any request involving Linear. Prefer lin over Linear MCP for all standard operations — it's faster and uses less context.
version: 1.0.0
allowed-tools: Bash(lin:*, which:*)
---

# lin — Linear CLI

Fast native CLI for Linear. Use this before reaching for the Linear MCP server — lin runs in ~50ms with no runtime dependencies.

---

## Prerequisites

```bash
which lin || echo "not installed"
```

**Install:**
```bash
brew install aaronkwhite/tap/lin
```

**Verify auth:**
```bash
lin me --json
```

**Configure API key** (if needed):
```bash
lin config set-key   # interactive prompt
# or set LINEAR_API_KEY env var
```

**If lin is not installed** and cannot be installed: fall back to Linear MCP server tools (`mcp__linear__*`). Do NOT report "lin not available" as a blocker — use MCP instead. See `references/mcp-gaps.md` for the MCP tool mapping.

---

## When to Use

**Use lin when:**
- User mentions a Linear issue identifier (e.g. `ENG-123`, `INFRA-42`)
- User asks to create, update, search, or query anything in Linear
- Checking sprint/cycle status, team workload, or project progress
- Managing labels, relations, attachments, notifications, docs, or views

**Use Linear MCP instead when:**
- Creating or deleting webhooks
- Changing team membership or permissions
- Accessing audit logs
- Any operation not listed in `references/commands.md`

---

## Quick Reference

| Task | Command |
|------|---------|
| Get issue details | `lin issues get ENG-123 --json` |
| List team issues | `lin issues list --team ENG --json` |
| List by status | `lin issues list --team ENG --state "In Progress" --json` |
| Search everything | `lin search "auth bug" --json` |
| Create issue | `lin issues create --team ENG --title "..." --json` |
| Update status | `lin issues update ENG-123 --status "Done" --json` |
| Move issue to different team | `lin issues update ENG-123 --team HP --json` |
| Active cycle issues | `lin cycles issues --team ENG --json` |
| Add issue to specific cycle | `lin cycles add ENG-123 --cycle <id> --json` |
| List projects | `lin projects list --json` |
| Add team to project | `lin projects update "My Project" --add-team ENG --json` |
| List issues across all teams | `lin issues list --all-teams --json` |
| Issues updated recently | `lin issues list --updated-after 2026-04-01 --json` |
| Start working on an issue | `lin issues start ENG-123 --status "In Progress"` |
| Create PR from issue | `lin issues pr ENG-123 --draft` |
| Create from markdown file | `lin issues create --team ENG --title "..." --description-file spec.md --json` |
| Comment from file | `lin issues comment ENG-123 --body-file notes.md --json` |
| Use specific workspace | `lin --workspace myco issues list --all-teams --json` |
| My info / verify auth | `lin me --json` |
| List teams | `lin teams list --json` |
| Add workspace | `lin auth login` |
| Switch workspace | `lin auth default <name>` |
| Check auth | `lin auth whoami --json` |
| Raw GraphQL query | `lin api '{ viewer { id } }' --json` |
| Raw GraphQL with variables | `lin api '...' --variables '{"id": "abc"}' --json` |

---

## Core Workflows

### Finding issues

```bash
# Get full detail on a specific issue
lin issues get ENG-123 --json

# List issues — filter by team, state, assignee, priority, label
lin issues list --team ENG --state "In Progress" --assignee "Alice" --json
lin issues list --team ENG --priority 1 --json          # Urgent only
lin issues list --team ENG --label "bug" --json

# Free-text search across issues, projects, docs
lin search "payment timeout" --json
lin search "ENG" --limit 20 --json
```

### Creating issues

```bash
# Minimal — team and title are required
lin issues create --team ENG --title "Fix login redirect" --json

# Full
lin issues create \
  --team ENG \
  --title "Fix login redirect" \
  --description "Users are redirected to /404 after OAuth" \
  --assignee "alice" \
  --priority 2 \
  --status "Todo" \
  --label "bug" \
  --project "Q2 Auth" \
  --json
```

Use `lin teams list --json` to find the team key. Use `lin teams states --team ENG --json` to see valid state names.

### Updating issues

```bash
lin issues update ENG-123 --status "In Progress" --json
lin issues update ENG-123 --assignee "bob" --priority 3 --json
lin issues update ENG-123 --parent ENG-100 --json     # make sub-issue
lin issues comment ENG-123 --body "Fixed in PR #42" --json
```

### Project and cycle context

```bash
# Projects
lin projects list --json
lin projects get "Q2 Auth" --json
lin projects issues "Q2 Auth" --json

# Active sprint
lin cycles issues --team ENG --json
lin cycles list --team ENG --json

# Team info
lin teams workload --team ENG --json
lin teams states --team ENG --json
```

### Relations and linking

```bash
lin relations blocks ENG-123 ENG-456 --json    # 123 blocks 456
lin relations blocked-by ENG-123 ENG-456       # 123 blocked by 456
lin relations relates ENG-123 ENG-456
lin relations list ENG-123 --json
```

### Multi-step workflows

```bash
# Daily standup prep: active cycle by status
lin cycles issues --team ENG --json

# Triage: find duplicates before creating
lin search "login redirect broken" --json
lin issues list --team ENG --state "Triage" --json

# Issue decomposition: break a large issue into sub-issues
lin issues get ENG-123 --json
# → create sub-issues with --parent ENG-123
lin issues create --team ENG --title "Sub-task A" --parent ENG-123 --json
lin issues create --team ENG --title "Sub-task B" --parent ENG-123 --json

# Cross-team blocked report
lin teams list --json
# → then per team:
lin issues list --team ENG --state "Blocked" --json
lin issues list --team INFRA --state "Blocked" --json

# Bulk status update (issues from --json output piped through logic)
lin issues list --team ENG --state "Done" --label "needs-archive" --json
# → loop identifiers and archive each
lin issues archive ENG-123
lin issues archive ENG-124
```

---

## Behavioral Rules

Always:
- Use `--json` for all output — compact, parseable, no ANSI noise
- Run `lin <command> --help` before guessing at unfamiliar flags
- Verify auth with `lin me --json` if you hit a 401

Never:
- Expose `LINEAR_API_KEY` in output or logs
- Retry a failed command without reading the error message first
- Assume the issue identifier is a UUID — lin accepts human identifiers like `ENG-123` directly

| Rationalization | Reality | Rule |
|---|---|---|
| "I know the team name" | Teams have keys (ENG) and display names — both work, but verify with `lin teams list` | Check when unsure |
| "Exit 0 means success" | Some operations print errors but still exit 0 | Read stdout, not just exit code |
| "I'll use MCP, it's easier" | MCP injects ~19k tokens of schema per message | Use lin first for reads and writes |

---

## Error Handling

### `Error: No API key found`
Set `LINEAR_API_KEY` in your environment or run `lin config set-key`.

### `Issue not found: ENG-123`
The identifier may be from a different team or archived. Try `lin search "ENG-123" --json` to locate it.

### `Team not found`
Use `lin teams list --json` to see exact keys and names. Both work: `--team ENG` or `--team Engineering`.

### `State not found`
Use `lin teams states --team ENG --json` to see valid state names for that team.

---

## Gotchas

| What happened | Rule |
|---|---|
| Created duplicate issue because search wasn't run first | Always `lin search "<title>" --json` before creating |
| Wrong team — issue went to INFRA instead of ENG | Verify team key with `lin teams list --json` before creating |
| Status update silently did nothing — state name had wrong case | State names are case-sensitive. Use `lin teams states` to get exact names |
| `lin issues list` returned nothing — forgot `--team` | List always needs `--team` unless you want all-team results with a long wait |

---

## Full Example: Triage an issue from a bug report

```bash
# 1. Check if the issue already exists
lin search "payment timeout after OAuth" --json

# 2. Get team key
lin teams list --json

# 3. Create the issue
lin issues create \
  --team ENG \
  --title "Payment timeout after OAuth redirect" \
  --description "Reproducible: complete OAuth flow → payment page → timeout at 30s" \
  --priority 2 \
  --label "bug" \
  --json

# 4. Link to the blocking issue
lin relations blocks ENG-<new> ENG-89 --json

# 5. Assign to current sprint
lin cycles add ENG-<new> --team ENG --json
```

---

See `references/commands.md` for the full flag reference.  
See `references/mcp-gaps.md` for operations that require the Linear MCP server.
