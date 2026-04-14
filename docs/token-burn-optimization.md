# Token Burn Optimization for AI/Agent Use

Research date: 2026-04-11. Sources: Speakeasy, DEV Community, OnlyCLI, apideck, InfoQ, Mintlify.

## What Token Burn Means

Token burn is the aggregate cost imposed on an AI's context window by CLI output. Every byte emitted becomes input tokens the model must process. In agentic loops this compounds: 10 calls to `lin issues list` across a session means 10x the output in context. Burn has two dimensions:

- **Per-call cost** — tokens a single command emits
- **Ambient cost** — tool schemas, help text, capability metadata loaded before the agent works at all

## Current Strengths

lin-cli is already well-positioned for AI use:

- Global `--json` flag (not per-command) ✓
- `--debug` goes to stderr only, never pollutes stdout ✓
- Interactive prompts auto-skip in non-interactive contexts ✓
- Destructive ops auto-confirm when non-interactive ✓
- Noun-verb command grammar (`lin issues list`) easy for agents to discover ✓

## Done

- [x] **Compact JSON by default** — `to_string_pretty` → `to_string`. Saves 20–40% tokens with no UX cost. Human-readable output is unchanged; only `--json` mode is affected. *(2026-04-11)*
- [x] **Strip GraphQL `data` envelope** — `{"data": {...}}` → `{...}` in `print_json`. Applies globally. *(2026-04-11)*

## Remaining (good CLI hygiene, not AI-specific)

- [ ] **Exit code taxonomy** — Shell scripts and agents alike can't distinguish "not found" from "network error" without parsing stderr text. Assign and document:
  - `2` = usage error (bad flag, missing required arg)
  - `3` = resource not found
  - `4` = auth failure / API key invalid
  - `5` = conflict / already exists
- [ ] **CLAUDE.md agent usage reference** — Compact command cheatsheet (~400 tokens) loaded once per Claude Code session. Prevents repeated `--help` calls. See `wrsmith108/linear-claude-skill` for a worked example.

## Related: linear-claude-skill Integration

`wrsmith108/linear-claude-skill` (v2.7.2) is a Claude Code skill explicitly designed to use lin-cli as a fast path:

> `lin` CLI (Optional) — Faster execution for status updates, search, and listings. ~50ms vs ~1s.

Tool selection hierarchy in that skill:
1. Linear MCP server (preferred for single ops)
2. **`lin` CLI** — fast path for status/search/list
3. TypeScript scripts — ops MCP doesn't cover
4. SDK patterns — bulk/loops
5. Raw GraphQL — last resort

Improvements to lin-cli's speed or feature surface directly benefit users of that skill.

### Patterns worth borrowing from that skill

- **PostToolUse hook** — scans edited files for `ENG-123` patterns after every Write/Edit, prompts Linear sync passively
- **Discovery-before-creation** enforced at the prompt layer, prevents duplicate issues
- **`content` vs `description` distinction** — Linear has two text fields: `description` (255 char limit, shows in lists) vs `content` (unlimited, shows in detail panel). Many integrations get this wrong.
- **Swarm/parallel agent sync** — coordinator stores pending sync state, spawns parallel sub-agents per issue batch via the `Task` tool, verifies results in AgentDB
