# Anonymous Usage Analytics

## Overview

Add lightweight, anonymous usage analytics to lin using PostHog. Events are queued locally and flushed in the background on each invocation. No PII, no workspace content, no API keys are ever collected.

## Decisions

- **Opt-out** with first-run stderr notice
- **Embedded write-only PostHog project token** in source (const in `src/analytics.rs`). Token is write-only by design — cannot read data. Document build-time env var as an alternative approach in a code comment for future reference.
- **Local queue + piggybacked flush** for reliable delivery
- **Standalone module** (`src/analytics.rs`) — no coupling to `LinearClient`

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

1. `DO_NOT_TRACK=1` env var — disabled
2. `analytics_enabled == Some(false)` in config — disabled
3. Otherwise — enabled

## Event Queue

- File: `~/.config/lin/analytics_queue.jsonl`
- One JSON object per line, each a complete PostHog event
- `analytics::track()` appends a line per invocation (after opt-out check passes)
- No size cap initially (~200 bytes/event, 10k events = ~2MB)

## Flush

- `analytics::flush()` spawned via `tokio::spawn` on every invocation, **after** `track()` returns (no race between write and read)
- Reads pending events from the queue file
- POSTs to `https://app.posthog.com/capture/` using PostHog batch API
- On success: truncates the queue file
- On failure: events stay queued for next run
- Uses its own reqwest client (short 5s timeout)
- `main.rs` awaits the flush handle with a 3s timeout before exiting — worst case adds 3s on network failure

## First-Run Notice

Printed to stderr once, when the install ID is first generated:

```
lin: anonymous usage stats enabled. Disable: lin config analytics off
```

## CLI Surface

Added to `lin config`:

- `lin config analytics off` — sets `analytics_enabled = Some(false)`, prints confirmation
- `lin config analytics on` — sets `analytics_enabled = Some(true)`, prints confirmation
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
