# MCP Fallback Reference

When `lin` doesn't cover an operation, use the Linear MCP server (`mcp__linear__*`).

The official MCP server is at `mcp.linear.app` (HTTP Streams, SSE deprecated 2026).

**Token cost warning:** The official MCP server injects ~19,659 tokens of tool schemas into context before a single operation. For high-frequency operations, always prefer `lin` to avoid this compounding across a session.

---

## What the official MCP server covers

Issues: create, update, get, list, search, archive  
Projects: list, get, create, update, labels, milestones, project updates  
Initiatives: create, edit, initiative updates  
Teams: list, get  
Users: list, get  
Comments: create  
Images: load  
URL resources: load Linear URLs as context

---

## Gaps — use `lin` instead

The official MCP server does **not** support:

| Operation | Use `lin` instead |
|-----------|-------------------|
| Cycles / sprints (any operation) | `lin cycles *` |
| Issue relations (blocks, blocked-by, duplicates) | `lin relations *` |
| Label creation / bulk apply | `lin labels *` |
| Documents (create, read, update) | `lin docs *` |
| Notifications | `lin notifications *` |
| Attachments | `lin attachments *` |
| Customers / customer needs | `lin customers *` |
| Views | `lin views *` |
| Bulk operations of any kind | `lin` + shell loop over `--json` output |
| Deletion of any entity | `lin * delete` |
| Workload / team analytics | `lin teams workload *` |
| Cross-team aggregate queries | multiple `lin` calls piped together |

---

## Operations with no API support (anywhere)

These require the Linear web UI:

- OAuth app management
- SSO / SCIM configuration
- Reminders (not in GraphQL API)

---

## How to decide

```
Does lin have a command for this?
  YES → use lin (faster, far less context cost)
  NO  → check if MCP covers it (see above)
  NEITHER → use Linear web UI
  NOT SURE → run `lin <noun> --help` to check
```

---

## Community MCP servers (if installed)

`tacticlaunch/mcp-linear` — 42 tools (vs. official ~21). Adds: archive/duplicate/transfer issues, relations management, subscribe to issues, add/remove labels, add issues to cycles, workflow state management, active cycle info. If this server is available, it fills most gaps — but still has higher token overhead than `lin`.

---

## Comparison summary

| Feature | Official MCP | lin |
|---------|-------------|-----|
| Issues | ✓ | ✓ |
| Projects | ✓ | ✓ |
| Initiatives | ✓ | ✓ |
| Teams | ✓ (list/get only) | ✓ (+ workload, states, members) |
| Cycles | ✗ | ✓ |
| Relations | ✗ | ✓ |
| Labels (CRUD + apply) | ✗ | ✓ |
| Documents | ✗ | ✓ |
| Notifications | ✗ | ✓ |
| Attachments | ✗ | ✓ |
| Customers | ✗ | ✓ |
| Views | ✗ | ✓ |
| Bulk operations | ✗ | ✓ (via --json + shell) |
| Deletion | ✗ | ✓ |
| Token overhead | ~19,659 | 0 |
