# Anonymous Usage Analytics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add anonymous, opt-out PostHog analytics to lin with local event queuing and background flush.

**Architecture:** Standalone `src/analytics.rs` module owns all analytics concerns — opt-out checks, install ID, queue file, PostHog POST. Zero coupling to `LinearClient`. Events are appended to a JSONL queue file synchronously, then flushed to PostHog via a background `tokio::spawn` task that runs concurrently with (and after) the main command. `main.rs` orchestrates timing, event building, and flush lifecycle.

**Tech Stack:** Existing reqwest (for flush HTTP), new `uuid` crate with `v4` feature (install ID generation)

**Spec:** `docs/superpowers/specs/2026-04-21-analytics-design.md`

---

## File Structure

| File | Role |
|---|---|
| `src/analytics.rs` | **New.** PostHog token const, `is_enabled()`, install ID management, `Event` struct, `track()`, `flush()`, `command_name()` |
| `src/config.rs` | **Modify.** Add `analytics_enabled: Option<bool>` field, `set_analytics_enabled()` and `is_analytics_enabled()` helpers |
| `src/commands/config.rs` | **Modify.** Add `Analytics` subcommand with `On`/`Off`/`Status` sub-subcommands |
| `src/cli.rs` | **No change.** `ConfigCommand` lives in `commands/config.rs`, not here |
| `src/main.rs` | **Modify.** Add `mod analytics`, wrap command dispatch with timing, call `track()`, spawn `flush()`, await with timeout |
| `Cargo.toml` | **Modify.** Add `uuid` dependency |

---

### Task 1: Add `uuid` dependency and `analytics_enabled` config field

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/config.rs`

- [ ] **Step 1: Write failing test for analytics_enabled roundtrip**

Add to the `#[cfg(test)] mod tests` block in `src/config.rs`:

```rust
#[test]
fn test_analytics_enabled_roundtrip() {
    let config = Config {
        analytics_enabled: Some(false),
        ..Config::default()
    };
    let serialized = toml::to_string_pretty(&config).unwrap();
    assert!(serialized.contains("analytics_enabled = false"));
    let deserialized: Config = toml::from_str(&serialized).unwrap();
    assert_eq!(deserialized.analytics_enabled, Some(false));
}

#[test]
fn test_analytics_enabled_default_is_none() {
    let config = Config::default();
    assert_eq!(config.analytics_enabled, None);
}

#[test]
fn test_analytics_enabled_missing_from_toml() {
    let toml_str = r#"
default_workspace = "myco"

[workspaces.myco]
api_key = "lin_api_test"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.analytics_enabled, None);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib config::tests::test_analytics_enabled`
Expected: compilation error — `analytics_enabled` field does not exist on `Config`

- [ ] **Step 3: Add uuid to Cargo.toml**

Add to `[dependencies]` section in `Cargo.toml`:

```toml
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 4: Add analytics_enabled field to Config**

In `src/config.rs`, add to the `Config` struct:

```rust
/// Whether anonymous usage analytics are enabled (None = enabled, opt-out default)
#[serde(default, skip_serializing_if = "Option::is_none")]
pub analytics_enabled: Option<bool>,
```

- [ ] **Step 5: Add analytics config helpers**

In `src/config.rs`, add these two public functions after `set_api_key`:

```rust
/// Set analytics enabled/disabled in config.
pub fn set_analytics_enabled(enabled: bool) -> anyhow::Result<()> {
    let mut config = load().unwrap_or_default();
    config.analytics_enabled = Some(enabled);
    save(&config)
}

/// Check if analytics are enabled in config. Default is true (opt-out).
pub fn is_analytics_enabled() -> bool {
    load()
        .ok()
        .and_then(|c| c.analytics_enabled)
        .unwrap_or(true)
}
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --lib config::tests`
Expected: all config tests pass, including the three new ones

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml src/config.rs
git commit -m "feat(analytics): add uuid dep and analytics_enabled config field"
```

---

### Task 2: Create analytics.rs — opt-out check and install ID

**Files:**
- Create: `src/analytics.rs`
- Modify: `src/main.rs` (just add `mod analytics`)

- [ ] **Step 1: Create analytics.rs with is_enabled and install ID, plus tests**

Create `src/analytics.rs`:

```rust
use std::fs;
use std::path::Path;

/// Write-only PostHog project token. Safe to embed — cannot read data, only send events.
/// Alternative: use compile-time env var instead:
///   const POSTHOG_TOKEN: &str = env!("LIN_POSTHOG_TOKEN");
/// This requires LIN_POSTHOG_TOKEN set at build time (add to CI secrets).
const POSTHOG_TOKEN: &str = "phc_PLACEHOLDER";

const POSTHOG_BATCH_URL: &str = "https://app.posthog.com/batch/";

/// Check if analytics are enabled. Checks in order:
/// 1. DO_NOT_TRACK=1 env var → disabled
/// 2. Config analytics_enabled == Some(false) → disabled
/// 3. Otherwise → enabled
pub fn is_enabled() -> bool {
    if std::env::var("DO_NOT_TRACK").ok().as_deref() == Some("1") {
        return false;
    }
    crate::config::is_analytics_enabled()
}

/// Get or create the anonymous install ID. Returns (id, was_first_run).
/// Uses the default config dir.
fn get_or_create_install_id() -> Option<(String, bool)> {
    let dir = crate::config::config_dir()?;
    get_or_create_install_id_in(&dir)
}

/// Get or create install ID in a specific directory. Testable.
fn get_or_create_install_id_in(dir: &Path) -> Option<(String, bool)> {
    let path = dir.join("analytics_id");
    if let Ok(id) = fs::read_to_string(&path) {
        let id = id.trim().to_string();
        if !id.is_empty() {
            return Some((id, false));
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    fs::create_dir_all(dir).ok()?;
    fs::write(&path, &id).ok()?;
    Some((id, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_enabled_default() {
        temp_env::with_var_unset("DO_NOT_TRACK", || {
            // With no config file and no env var, should default to enabled
            // (is_analytics_enabled returns true when config can't be loaded or field is None)
            assert!(is_enabled());
        });
    }

    #[test]
    fn test_is_enabled_do_not_track() {
        temp_env::with_var("DO_NOT_TRACK", Some("1"), || {
            assert!(!is_enabled());
        });
    }

    #[test]
    fn test_is_enabled_do_not_track_other_values() {
        temp_env::with_var("DO_NOT_TRACK", Some("0"), || {
            assert!(is_enabled());
        });
        temp_env::with_var("DO_NOT_TRACK", Some("true"), || {
            assert!(is_enabled());
        });
    }

    #[test]
    fn test_install_id_created_on_first_run() {
        let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);

        let (id, first_run) = get_or_create_install_id_in(&dir).unwrap();
        assert!(first_run);
        assert_eq!(id.len(), 36); // UUID format

        // File should exist now
        let stored = fs::read_to_string(dir.join("analytics_id")).unwrap();
        assert_eq!(stored, id);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_install_id_loaded_on_second_run() {
        let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);

        let (id1, first) = get_or_create_install_id_in(&dir).unwrap();
        assert!(first);

        let (id2, second) = get_or_create_install_id_in(&dir).unwrap();
        assert!(!second);
        assert_eq!(id1, id2);

        let _ = fs::remove_dir_all(&dir);
    }
}
```

- [ ] **Step 2: Register the module in main.rs**

Add `mod analytics;` to the module declarations at the top of `src/main.rs` (after `mod cli;`):

```rust
mod analytics;
mod cli;
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test --lib analytics::tests`
Expected: all 5 tests pass

- [ ] **Step 4: Commit**

```bash
git add src/analytics.rs src/main.rs
git commit -m "feat(analytics): add opt-out check and install ID management"
```

---

### Task 3: Event tracking — queue write

**Files:**
- Modify: `src/analytics.rs`

- [ ] **Step 1: Add Event struct and track functions with tests**

Add to `src/analytics.rs`, after the install ID functions:

```rust
use std::io::Write;

pub struct Event {
    pub command: String,
    pub flags: Vec<String>,
    pub success: bool,
    pub duration_ms: u64,
}

/// Track a command execution event. Appends to the local queue file.
/// Prints first-run notice to stderr if this is the first invocation.
pub fn track(event: &Event) {
    if !is_enabled() {
        return;
    }
    let Some(dir) = crate::config::config_dir() else {
        return;
    };
    if track_to_dir(&dir, event) {
        eprintln!("lin: anonymous usage stats enabled. Disable: lin config analytics off");
    }
}

/// Write event to queue file in the given directory. Returns true if first run.
fn track_to_dir(dir: &Path, event: &Event) -> bool {
    let Some((install_id, first_run)) = get_or_create_install_id_in(dir) else {
        return false;
    };

    let payload = serde_json::json!({
        "event": "command_executed",
        "distinct_id": install_id,
        "properties": {
            "command": event.command,
            "flags": event.flags,
            "success": event.success,
            "duration_ms": event.duration_ms,
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        }
    });

    let queue_path = dir.join("analytics_queue.jsonl");
    let Ok(line) = serde_json::to_string(&payload) else {
        return false;
    };

    let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&queue_path)
    else {
        return false;
    };
    let _ = writeln!(file, "{line}");
    first_run
}
```

Add these tests to the existing `#[cfg(test)] mod tests` block:

```rust
#[test]
fn test_track_writes_to_queue() {
    let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
    let _ = fs::create_dir_all(&dir);

    let event = Event {
        command: "issues list".to_string(),
        flags: vec!["--json".to_string()],
        success: true,
        duration_ms: 150,
    };

    let first_run = track_to_dir(&dir, &event);
    assert!(first_run);

    let queue = fs::read_to_string(dir.join("analytics_queue.jsonl")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(queue.trim()).unwrap();
    assert_eq!(parsed["event"], "command_executed");
    assert_eq!(parsed["properties"]["command"], "issues list");
    assert_eq!(parsed["properties"]["flags"][0], "--json");
    assert_eq!(parsed["properties"]["success"], true);
    assert_eq!(parsed["properties"]["duration_ms"], 150);
    assert!(parsed["properties"]["version"].is_string());
    assert!(parsed["properties"]["os"].is_string());
    assert!(parsed["properties"]["arch"].is_string());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_track_appends_multiple_events() {
    let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
    let _ = fs::create_dir_all(&dir);

    for i in 0..3 {
        let event = Event {
            command: format!("command {i}"),
            flags: vec![],
            success: true,
            duration_ms: i * 100,
        };
        track_to_dir(&dir, &event);
    }

    let queue = fs::read_to_string(dir.join("analytics_queue.jsonl")).unwrap();
    let lines: Vec<&str> = queue.trim().lines().collect();
    assert_eq!(lines.len(), 3);

    // All lines should be valid JSON with the same distinct_id
    let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    let third: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(first["distinct_id"], third["distinct_id"]);

    let _ = fs::remove_dir_all(&dir);
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test --lib analytics::tests`
Expected: all 7 tests pass (5 from Task 2 + 2 new)

- [ ] **Step 3: Commit**

```bash
git add src/analytics.rs
git commit -m "feat(analytics): add event queue write"
```

---

### Task 4: Flush — read queue and POST to PostHog

**Files:**
- Modify: `src/analytics.rs`

- [ ] **Step 1: Add flush functions and test with wiremock**

Add to `src/analytics.rs`, after `track_to_dir`:

```rust
use std::time::Duration;

/// Flush pending analytics events to PostHog.
pub async fn flush() {
    let Some(dir) = crate::config::config_dir() else {
        return;
    };
    flush_dir(&dir, POSTHOG_BATCH_URL).await;
}

/// Flush events from a specific directory to a specific URL. Testable.
async fn flush_dir(dir: &Path, url: &str) {
    let queue_path = dir.join("analytics_queue.jsonl");

    let contents = match fs::read_to_string(&queue_path) {
        Ok(c) if !c.trim().is_empty() => c,
        _ => return,
    };

    let events: Vec<serde_json::Value> = contents
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    if events.is_empty() {
        return;
    }

    let batch_payload = serde_json::json!({
        "api_key": POSTHOG_TOKEN,
        "batch": events,
    });

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    let response = client.post(url).json(&batch_payload).send().await;

    if let Ok(resp) = response {
        if resp.status().is_success() {
            let _ = fs::write(&queue_path, "");
        }
    }
}
```

Add this test to the `#[cfg(test)] mod tests` block:

```rust
#[tokio::test]
async fn test_flush_sends_and_clears_queue() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

    let server = MockServer::start().await;
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/batch/"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
    let _ = fs::create_dir_all(&dir);

    // Write some events to the queue
    let event = Event {
        command: "issues list".to_string(),
        flags: vec![],
        success: true,
        duration_ms: 100,
    };
    track_to_dir(&dir, &event);
    track_to_dir(&dir, &event);

    let queue_path = dir.join("analytics_queue.jsonl");
    assert!(fs::read_to_string(&queue_path).unwrap().lines().count() == 2);

    // Flush to the mock server
    let url = format!("{}/batch/", server.uri());
    flush_dir(&dir, &url).await;

    // Queue should be empty after successful flush
    let after = fs::read_to_string(&queue_path).unwrap();
    assert!(after.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_flush_keeps_queue_on_failure() {
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

    let server = MockServer::start().await;
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/batch/"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&server)
        .await;

    let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
    let _ = fs::create_dir_all(&dir);

    let event = Event {
        command: "teams list".to_string(),
        flags: vec![],
        success: true,
        duration_ms: 50,
    };
    track_to_dir(&dir, &event);

    let queue_path = dir.join("analytics_queue.jsonl");
    let before = fs::read_to_string(&queue_path).unwrap();

    let url = format!("{}/batch/", server.uri());
    flush_dir(&dir, &url).await;

    // Queue should still have the event
    let after = fs::read_to_string(&queue_path).unwrap();
    assert_eq!(before, after);

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_flush_noop_on_empty_queue() {
    let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
    let _ = fs::create_dir_all(&dir);

    // No queue file exists — flush should silently return
    flush_dir(&dir, "http://localhost:1/batch/").await;

    // No crash, no file created
    assert!(!dir.join("analytics_queue.jsonl").exists());

    let _ = fs::remove_dir_all(&dir);
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test --lib analytics::tests`
Expected: all 10 tests pass (7 from Task 3 + 3 new)

- [ ] **Step 3: Commit**

```bash
git add src/analytics.rs
git commit -m "feat(analytics): add queue flush to PostHog"
```

---

### Task 5: `lin config analytics` subcommand

**Files:**
- Modify: `src/commands/config.rs`

- [ ] **Step 1: Add Analytics subcommand variants**

In `src/commands/config.rs`, add the new types and update the enum and execute function.

Add import at the top:

```rust
use clap::{Args, Subcommand};
```

(Already imported — just verifying.)

Add to the `ConfigCommand` enum:

```rust
/// Manage anonymous usage analytics
Analytics(AnalyticsArgs),
```

Add the new structs after `ConfigCommand`:

```rust
#[derive(Args, Debug)]
pub struct AnalyticsArgs {
    #[command(subcommand)]
    pub command: AnalyticsCommand,
}

#[derive(Subcommand, Debug)]
pub enum AnalyticsCommand {
    /// Enable anonymous usage analytics
    On,
    /// Disable anonymous usage analytics
    Off,
    /// Show analytics status
    Status,
}
```

Add the match arm in `execute()`:

```rust
ConfigCommand::Analytics(args) => analytics_cmd(&args.command, json),
```

Add the handler function:

```rust
fn analytics_cmd(cmd: &AnalyticsCommand, json: bool) -> anyhow::Result<()> {
    match cmd {
        AnalyticsCommand::Off => {
            crate::config::set_analytics_enabled(false)?;
            if json {
                crate::output::print_json(&serde_json::json!({ "analytics": false }));
            } else {
                println!(
                    "{} Analytics disabled",
                    style("✓").green().bold()
                );
            }
        }
        AnalyticsCommand::On => {
            crate::config::set_analytics_enabled(true)?;
            if json {
                crate::output::print_json(&serde_json::json!({ "analytics": true }));
            } else {
                println!(
                    "{} Analytics enabled",
                    style("✓").green().bold()
                );
            }
        }
        AnalyticsCommand::Status => {
            let enabled = crate::analytics::is_enabled();
            let do_not_track = std::env::var("DO_NOT_TRACK").ok().as_deref() == Some("1");
            let config_value = crate::config::load()
                .ok()
                .and_then(|c| c.analytics_enabled);
            let install_id_path = crate::config::config_dir()
                .map(|d| d.join("analytics_id"));
            let install_id = install_id_path
                .and_then(|p| std::fs::read_to_string(p).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            if json {
                crate::output::print_json(&serde_json::json!({
                    "enabled": enabled,
                    "config_value": config_value,
                    "do_not_track_override": do_not_track,
                    "install_id": install_id,
                }));
            } else {
                let status = if enabled { "enabled" } else { "disabled" };
                println!("Analytics: {}", style(status).bold());
                if do_not_track {
                    println!("  {} DO_NOT_TRACK=1 is set (overrides config)", style("!").yellow().bold());
                }
                if let Some(id) = install_id {
                    println!("  Install ID: {}", style(id).dim());
                }
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 2: Run the build to verify it compiles**

Run: `cargo build 2>&1`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/commands/config.rs
git commit -m "feat(analytics): add config analytics on/off/status subcommand"
```

---

### Task 6: Wire analytics into main.rs

**Files:**
- Modify: `src/main.rs`
- Modify: `src/analytics.rs` (add `command_name` function)

- [ ] **Step 1: Add command_name function to analytics.rs**

Add to `src/analytics.rs`, after the `flush` functions:

```rust
/// Extract the top-level command name from a Commands variant.
pub fn command_name(cmd: &crate::cli::Commands) -> &'static str {
    use crate::cli::Commands;
    match cmd {
        Commands::Api(_) => "api",
        Commands::Auth(_) => "auth",
        Commands::Issues(_) => "issues",
        Commands::Projects(_) => "projects",
        Commands::Cycles(_) => "cycles",
        Commands::Initiatives(_) => "initiatives",
        Commands::Roadmap(_) => "roadmap",
        Commands::Labels(_) => "labels",
        Commands::Teams(_) => "teams",
        Commands::Relations(_) => "relations",
        Commands::Customers(_) => "customers",
        Commands::Views(_) => "views",
        Commands::Docs(_) => "docs",
        Commands::Notifications(_) => "notifications",
        Commands::Me(_) => "me",
        Commands::Attachments(_) => "attachments",
        Commands::Search(_) => "search",
        Commands::Config(_) => "config",
        Commands::Completions { .. } => "completions",
    }
}
```

- [ ] **Step 2: Update main.rs with timing, tracking, and flush**

Replace the entire contents of `src/main.rs` with:

```rust
mod analytics;
mod cli;
mod client;
mod commands;
mod config;
mod error;
mod graphql;
mod output;
mod util;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let ws = cli.workspace.as_deref();

    // Completions are a local operation — skip analytics, return early
    if let Commands::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        clap_complete::generate(*shell, &mut cmd, "lin", &mut std::io::stdout());
        return;
    }

    let start = Instant::now();

    let result = match &cli.command {
        Commands::Api(args) => commands::api::execute(args, cli.json, cli.debug, ws).await,
        Commands::Auth(args) => commands::auth::execute(args, cli.json, cli.debug, ws).await,
        Commands::Issues(args) => commands::issues::execute(args, cli.json, cli.debug, ws).await,
        Commands::Projects(args) => {
            commands::projects::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Cycles(args) => commands::cycles::execute(args, cli.json, cli.debug, ws).await,
        Commands::Initiatives(args) => {
            commands::initiatives::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Roadmap(args) => commands::roadmap::execute(args, cli.json, cli.debug, ws).await,
        Commands::Labels(args) => commands::labels::execute(args, cli.json, cli.debug, ws).await,
        Commands::Teams(args) => commands::teams::execute(args, cli.json, cli.debug, ws).await,
        Commands::Relations(args) => {
            commands::relations::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Customers(args) => {
            commands::customers::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Views(args) => commands::views::execute(args, cli.json, cli.debug, ws).await,
        Commands::Docs(args) => commands::docs::execute(args, cli.json, cli.debug, ws).await,
        Commands::Notifications(args) => {
            commands::notifications::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Me(args) => commands::me::execute(args, cli.json, cli.debug, ws).await,
        Commands::Attachments(args) => {
            commands::attachments::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Search(args) => commands::search::execute(args, cli.json, cli.debug, ws).await,
        Commands::Config(args) => commands::config::execute(args, cli.json, cli.debug, ws).await,
        Commands::Completions { .. } => unreachable!(),
    };

    let duration = start.elapsed();
    let success = result.is_ok();

    // Print error immediately so user sees it without waiting for flush
    if let Err(ref e) = result {
        eprintln!("Error: {e}");
    }

    // Track analytics event (sync — writes to local queue file)
    let mut flags = Vec::new();
    if cli.json {
        flags.push("--json".to_string());
    }
    if cli.debug {
        flags.push("--debug".to_string());
    }
    if cli.workspace.is_some() {
        flags.push("--workspace".to_string());
    }

    analytics::track(&analytics::Event {
        command: analytics::command_name(&cli.command).to_string(),
        flags,
        success,
        duration_ms: duration.as_millis() as u64,
    });

    // Flush analytics queue in background, wait up to 3s
    let flush_handle = tokio::spawn(analytics::flush());
    let _ = tokio::time::timeout(std::time::Duration::from_secs(3), flush_handle).await;

    if result.is_err() {
        std::process::exit(1);
    }
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test`
Expected: all tests pass (config, analytics, query validation, etc.)

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: no warnings

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/analytics.rs
git commit -m "feat(analytics): wire tracking and flush into main dispatch loop"
```

---

### Task 7: Set PostHog token and smoke test

**Files:**
- Modify: `src/analytics.rs` (replace placeholder token)

- [ ] **Step 1: Replace the placeholder PostHog token**

In `src/analytics.rs`, replace the `POSTHOG_TOKEN` const value:

```rust
const POSTHOG_TOKEN: &str = "phc_YOUR_ACTUAL_TOKEN_HERE";
```

(The user must create a PostHog project and paste the write-only project API key here.)

- [ ] **Step 2: Build and run a smoke test**

```bash
cargo build
./target/debug/lin config analytics status
```

Expected: Shows "Analytics: enabled" and an install ID (generated on first run). First run also prints `lin: anonymous usage stats enabled. Disable: lin config analytics off` to stderr.

- [ ] **Step 3: Verify queue file was created and flushed**

```bash
cat ~/.config/lin/analytics_queue.jsonl
```

Expected: file is empty (events were flushed to PostHog) or file doesn't exist (flushed and cleaned up).

- [ ] **Step 4: Test opt-out**

```bash
./target/debug/lin config analytics off
./target/debug/lin issues list 2>/dev/null  # (will fail without auth, that's ok)
cat ~/.config/lin/analytics_queue.jsonl
```

Expected: no new events written to queue after disabling. Re-enable with `lin config analytics on`.

- [ ] **Step 5: Test DO_NOT_TRACK**

```bash
./target/debug/lin config analytics on
DO_NOT_TRACK=1 ./target/debug/lin config analytics status
```

Expected: status shows "disabled" with "DO_NOT_TRACK=1 is set (overrides config)" notice.

- [ ] **Step 6: Run preflight**

```bash
./scripts/preflight.sh
```

Expected: fmt, clippy, tests, build all pass.

- [ ] **Step 7: Commit the real token**

```bash
git add src/analytics.rs
git commit -m "feat(analytics): set PostHog project token"
```
