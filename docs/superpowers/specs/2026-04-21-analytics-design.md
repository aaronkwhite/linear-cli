# Anonymous Usage Analytics

## Overview

Add lightweight, anonymous usage analytics to lin using PostHog. Events are queued locally and flushed in the background on each invocation. No PII, no workspace content, no API keys are ever collected.

## Decisions

- **Opt-out for interactive sessions, opt-in for non-interactive** (agents / CI / piped). Non-interactive first-run does not mint a UUID or queue events — opt-in requires `lin config analytics on` from a TTY, or a pre-existing install ID.
- **First-run notice prints BEFORE any data is queued.** First interactive invocation mints the UUID and prints the notice but does not record the event itself. From the second invocation on, events are tracked normally.
- **Embedded write-only PostHog project token** in source (const in `src/analytics.rs`). Token is write-only by design — cannot read data. Build-time env var documented as an alternative.
- **Local queue + debounced flush.** Flush runs at most once per 10 minutes per machine (sentinel mtime). The common path skips the flush spawn entirely — no network, no 3-second wait.
- **Atomic flush via rename-then-process.** `analytics_queue.jsonl` is renamed to `analytics_queue.flushing.<pid>.<nanos>.jsonl` before reading, eliminating the race where a concurrent writer's events get truncated. On 5xx / network failure, flushing-file contents are appended back to the queue.
- **Queue size cap at 1000 events.** On overflow, the oldest events drop first (FIFO).
- **HTTP status handling:** 2xx → drop events (delivered). 4xx → drop events (poison pill). 5xx / network → retain for retry.
- **Opt-out cleanup.** `lin config analytics off` removes the queue file and install UUID. `lin config analytics on` mints a fresh UUID (does not re-correlate to the prior identity).
- **Standalone module** (`src/analytics.rs`) — no coupling to `LinearClient`.

## What We Track

Single event type: `command_executed`

| Property | Example | Notes |
|---|---|---|
| `command` | `"issues list"` | Subcommand name |
| `flags` | `["--json", "--all-teams"]` | Flag presence only, never values |
| `success` | `true` | Whether the command exited successfully |
| `duration_ms` | `342` | Wall-clock execution time |
| `version` | `"2026.4.16"` | lin version from Cargo |
| `os` | `"darwin"` | `std::env::consts::OS` |
| `arch` | `"aarch64"` | `std::env::consts::ARCH` |
| `schema_version` | `1` | Bumped on any breaking property change |

**Never collected:** API keys, issue content, workspace names, identifiers, query text, user identity.

## Identity

- `distinct_id` is a random UUIDv4 generated on first use
- Stored in `~/.config/lin/analytics_id` (plain text, separate from config.toml)
- Not derived from any user or workspace data

## Config

One new field in `Config`:

```rust
pub analytics_enabled: Option<bool>, // None = enabled (opt-out default)
```

`None` and `Some(true)` = enabled. `Some(false)` = disabled. Existing configs require no migration.

## Opt-Out Checks (evaluated in order)

1. `DO_NOT_TRACK` env var set to **any non-empty value** — disabled (consoledonottrack.com convention)
2. `analytics_enabled == Some(false)` in config — disabled
3. First run on a non-interactive session (no TTY on stderr) and no install ID exists — disabled (don't implicitly opt in agents / CI)
4. Otherwise — enabled

## Event Queue

- File: `~/.config/lin/analytics_queue.jsonl`
- One JSON object per line, each a complete PostHog event
- `analytics::track()` appends a line per invocation (after opt-out check passes and after the first run)
- Capped at **1000 events**; oldest dropped on overflow

## Flush

- Debounced by `~/.config/lin/analytics_last_flush` mtime — at most once per 10 minutes per machine. Most invocations skip the flush spawn entirely (no network call, no 3s wait, fast exit).
- When due: spawned via `tokio::spawn` and bounded by a 3-second timeout in `main.rs`.
- **Atomic rename:** `analytics_queue.jsonl` → `analytics_queue.flushing.<pid>.<nanos>.jsonl` before reading. Concurrent writers cannot lose events to a truncate-then-write race.
- POSTs to `https://app.posthog.com/batch/` using PostHog batch API.
- **Status handling:** 2xx → delete flushing file (delivered). 4xx → delete (poison pill, don't retry forever). 5xx / network error → append flushing file contents back to the queue.
- Own reqwest client with a 5-second timeout.

## First-Run Notice

Printed to stderr on the first **interactive** invocation, when the install ID is minted. The event from this invocation is not queued — the user must see the notice before any data is shipped.

```
lin: anonymous usage stats enabled. Disable: `lin config analytics off`, or set DO_NOT_TRACK=1.
```

In non-interactive sessions, no notice is printed and no UUID is minted; analytics stays off until the user runs `lin config analytics on` from a TTY.

## CLI Surface

Added to `lin config`:

- `lin config analytics off` — sets `analytics_enabled = Some(false)` and removes queue file + install UUID
- `lin config analytics on` — sets `analytics_enabled = Some(true)` and resets the install UUID (mints fresh on next run)
- `lin config analytics status` — shows enabled/disabled (notes `DO_NOT_TRACK` override), shows install ID if present

All respect `--json` flag.

## Files Changed

| File | Change |
|---|---|
| `src/analytics.rs` | New module: track, flush, install ID, opt-out checks, PostHog token |
| `src/config.rs` | Add `analytics_enabled: Option<bool>` to `Config` |
| `src/commands/config.rs` | Add `Analytics` subcommand with on/off/status |
| `src/main.rs` | Add `mod analytics`, wrap dispatch with timing, call track + spawn flush |

## Build-Time Token Alternative

If the embedded token ever needs to be rotated without a source change, switch to:

```rust
const POSTHOG_TOKEN: &str = env!("LIN_POSTHOG_TOKEN");
```

This requires the env var set at compile time (CI would need `LIN_POSTHOG_TOKEN` in secrets). Not needed now — the write-only token in source is fine for the current threat model.
