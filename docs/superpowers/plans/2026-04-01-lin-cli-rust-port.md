# lin-cli Rust Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Python `lin` CLI to Rust with full API parity (11 command groups, 67 subcommands), type-safe GraphQL via cynic, and rich `gh`-style output.

**Architecture:** clap derive for CLI parsing, cynic for compile-time validated GraphQL, reqwest+tokio for async HTTP, comfy-table+dialoguer+console for rich interactive output. LinearClient handles auth, retries, pagination, and caching.

**Tech Stack:** Rust, clap, cynic, reqwest (rustls-tls), tokio, serde, thiserror, anyhow, comfy-table, console, dialoguer, termimad, dotenvy

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Dependencies, CalVer version, binary config |
| `schema.graphql` | Linear's GraphQL schema (committed) |
| `.graphql_config.yml` | cynic schema registration |
| `src/main.rs` | Entry point, tokio runtime, error display |
| `src/cli.rs` | Top-level clap App, global flags, subcommand dispatch |
| `src/error.rs` | LinearError enum via thiserror |
| `src/client/mod.rs` | LinearClient: auth, query, retry, pagination |
| `src/client/cache.rs` | Cached lookups: teams, users, projects, states, labels |
| `src/graphql/mod.rs` | Schema module, shared fragments (IssueSummary, IssueDetail) |
| `src/graphql/issues.rs` | Issue queries and mutations |
| `src/graphql/projects.rs` | Project queries and mutations |
| `src/graphql/cycles.rs` | Cycle queries and mutations |
| `src/graphql/roadmap.rs` | Roadmap/milestone/initiative queries and mutations |
| `src/graphql/labels.rs` | Label queries and mutations |
| `src/graphql/teams.rs` | Team queries |
| `src/graphql/relations.rs` | Relation queries and mutations |
| `src/graphql/customers.rs` | Customer queries and mutations |
| `src/graphql/views.rs` | View queries and mutations |
| `src/graphql/docs.rs` | Document queries and mutations |
| `src/graphql/notifications.rs` | Notification queries and mutations |
| `src/commands/mod.rs` | Re-exports all command modules |
| `src/commands/issues.rs` | Issue command handlers |
| `src/commands/projects.rs` | Project command handlers |
| `src/commands/cycles.rs` | Cycle command handlers |
| `src/commands/roadmap.rs` | Roadmap command handlers |
| `src/commands/labels.rs` | Label command handlers |
| `src/commands/teams.rs` | Team command handlers |
| `src/commands/relations.rs` | Relation command handlers |
| `src/commands/customers.rs` | Customer command handlers |
| `src/commands/views.rs` | View command handlers |
| `src/commands/docs.rs` | Document command handlers |
| `src/commands/notifications.rs` | Notification command handlers |
| `src/output/mod.rs` | OutputContext, format dispatch (json/human/interactive) |
| `src/output/table.rs` | comfy-table wrappers |
| `src/output/detail.rs` | Key-value detail views |
| `src/output/color.rs` | ANSI color helpers, NO_COLOR, priority formatting |
| `src/output/interactive.rs` | dialoguer fuzzy select, confirm, multi-select |
| `tests/cli_smoke.rs` | CLI parsing tests (no API key) |
| `tests/integration.rs` | Real API integration tests |
| `README.md` | Installation, usage, commands reference |
| `CHANGELOG.md` | CalVer changelog |

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `README.md`
- Create: `CHANGELOG.md`
- Create: `.gitignore`

- [ ] **Step 1: Initialize git repo**

```bash
cd /Users/wookiedrool/src/lin-cli
git init
```

- [ ] **Step 2: Create .gitignore**

Create `.gitignore`:
```
/target
.env
.env.local
```

- [ ] **Step 3: Create Cargo.toml**

Create `Cargo.toml`:
```toml
[package]
name = "lin-cli"
version = "2026.4.0"
edition = "2021"
description = "Linear CLI — manage issues, projects, cycles, and more from the terminal"
license = "MIT"

[[bin]]
name = "lin"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
cynic = "3"
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
comfy-table = "7"
console = "0.15"
dialoguer = { version = "0.11", features = ["fuzzy-select"] }
termimad = "0.30"
dotenvy = "0.15"
dashmap = "6"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
wiremock = "0.6"
temp-env = "0.3"
```

- [ ] **Step 4: Create minimal main.rs**

Create `src/main.rs`:
```rust
fn main() {
    println!("lin-cli");
}
```

- [ ] **Step 5: Create README.md**

Create `README.md`:
```markdown
# lin — Linear CLI

A fast, native CLI for [Linear](https://linear.app). Manage issues, projects, cycles, and more from your terminal.

## Install

### From source

```bash
cargo install --path .
```

### From crates.io

```bash
cargo install lin-cli
```

## Setup

Get your API key from **Linear Settings > API > Personal API keys**, then:

```bash
export LINEAR_API_KEY="lin_api_..."
```

Or create a `.env` file:

```
LINEAR_API_KEY=lin_api_...
```

## Usage

```bash
lin --help                    # Show all commands
lin issues list --team ENG    # List issues for a team
lin issues get ENG-123        # Get issue details
lin projects list             # List projects
lin teams list                # List teams
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output raw JSON for scripting |
| `--debug` | Print GraphQL queries/responses to stderr |
| `--version` | Show version |

## Commands

| Group | Description |
|-------|-------------|
| `issues` | List, create, update, search, comment, archive issues |
| `projects` | List, create, update projects and their issues |
| `cycles` | List, create, manage cycles and cycle issues |
| `roadmap` | Project updates, milestones, initiatives |
| `labels` | Create, manage, apply labels |
| `teams` | List teams, members, states, workload |
| `relations` | Issue dependencies and relations |
| `customers` | Customer management, needs, tiers |
| `views` | Custom views and their issues |
| `docs` | Documents: create, search, manage |
| `notifications` | View and manage notifications |

## Why CLI over MCP?

| | CLI | MCP |
|---|---|---|
| Tokens per operation | ~1,300 | ~44,000 |
| Reliability | 100% | ~72% |
| Dependencies | Single binary | Schema injection |

## Development

```bash
cargo build                           # Build
cargo test                            # Run tests (smoke + unit)
LINEAR_API_KEY=... cargo test -- --ignored  # Run integration tests
```

## License

MIT
```

- [ ] **Step 6: Create CHANGELOG.md**

Create `CHANGELOG.md`:
```markdown
# Changelog

All notable changes to lin-cli will be documented in this file.

Versioning follows [CalVer](https://calver.org/) with format `YYYY.MM.PATCH`.

## [2026.4.0] — 2026-04-01

### Added
- Initial Rust port of lin CLI
- Full parity with Python version: 11 command groups, 67 subcommands
- Type-safe GraphQL queries via cynic
- Rich terminal output with colors, tables, and interactive prompts
- Single static binary distribution (no runtime dependencies)
```

- [ ] **Step 7: Verify it compiles**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo build
```
Expected: successful build.

- [ ] **Step 8: Commit**

```bash
git add .gitignore Cargo.toml src/main.rs README.md CHANGELOG.md
git commit -m "feat: scaffold lin-cli Rust project with CalVer versioning"
```

---

### Task 2: Error Types

**Files:**
- Create: `src/error.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create error.rs**

Create `src/error.rs`:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LinearError {
    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },

    #[error(
        "LINEAR_API_KEY not found.\n\
         Set it via environment variable or .env file.\n\
         Get your key from: Linear Settings > API > Personal API keys"
    )]
    NoApiKey,

    #[error("{entity} not found: {name}")]
    NotFound { entity: &'static str, name: String },

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```

- [ ] **Step 2: Wire into main.rs**

Replace `src/main.rs`:
```rust
mod error;

fn main() {
    println!("lin-cli");
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo build
```

- [ ] **Step 4: Commit**

```bash
git add src/error.rs src/main.rs
git commit -m "feat: add LinearError types via thiserror"
```

---

### Task 3: Output Module

**Files:**
- Create: `src/output/mod.rs`
- Create: `src/output/color.rs`
- Create: `src/output/table.rs`
- Create: `src/output/detail.rs`
- Create: `src/output/interactive.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write color tests**

Create `src/output/color.rs`:
```rust
use console::Style;
use std::env;

pub fn is_color_enabled() -> bool {
    env::var("NO_COLOR").is_err() && console::Term::stdout().is_term()
}

pub fn bold(text: &str) -> String {
    if is_color_enabled() {
        Style::new().bold().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn dim(text: &str) -> String {
    if is_color_enabled() {
        Style::new().dim().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn green(text: &str) -> String {
    if is_color_enabled() {
        Style::new().green().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn yellow(text: &str) -> String {
    if is_color_enabled() {
        Style::new().yellow().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn red(text: &str) -> String {
    if is_color_enabled() {
        Style::new().red().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn cyan(text: &str) -> String {
    if is_color_enabled() {
        Style::new().cyan().apply_to(text).to_string()
    } else {
        text.to_string()
    }
}

pub fn format_priority(priority: i32) -> String {
    match priority {
        1 => red("!!!"),
        2 => yellow("!!"),
        3 => cyan("!"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_priority_urgent() {
        // NO_COLOR likely set in test env or not a TTY, so just check content
        let result = format_priority(1);
        assert!(result.contains("!!!") || result == "!!!");
    }

    #[test]
    fn test_format_priority_high() {
        let result = format_priority(2);
        assert!(result.contains("!!") || result == "!!");
    }

    #[test]
    fn test_format_priority_medium() {
        let result = format_priority(3);
        assert!(result.contains("!") || result == "!");
    }

    #[test]
    fn test_format_priority_low() {
        assert_eq!(format_priority(4), "");
    }

    #[test]
    fn test_format_priority_none() {
        assert_eq!(format_priority(0), "");
    }

    #[test]
    fn test_no_color_returns_plain_text() {
        // In test environment (not a TTY), functions should return plain text
        let result = bold("hello");
        assert!(result.contains("hello"));
    }
}
```

- [ ] **Step 2: Run color tests**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test output::color
```
Expected: all tests pass.

- [ ] **Step 3: Create table.rs**

Create `src/output/table.rs`:
```rust
use comfy_table::{presets::UTF8_FULL_CONDENSED, Attribute, Cell, Color, ContentArrangement, Table};

/// Print a formatted table with optional headers.
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    if !headers.is_empty() {
        table.set_header(
            headers
                .iter()
                .map(|h| Cell::new(h).add_attribute(Attribute::Bold))
                .collect::<Vec<_>>(),
        );
    }

    for row in rows {
        table.add_row(row);
    }

    println!("{table}");
}

/// Print a table with colored status column (column index specified).
pub fn print_table_with_status(
    headers: &[&str],
    rows: &[Vec<String>],
    status_col: usize,
) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    if !headers.is_empty() {
        table.set_header(
            headers
                .iter()
                .map(|h| Cell::new(h).add_attribute(Attribute::Bold))
                .collect::<Vec<_>>(),
        );
    }

    for row in rows {
        let cells: Vec<Cell> = row
            .iter()
            .enumerate()
            .map(|(i, val)| {
                if i == status_col {
                    let color = match val.to_lowercase().as_str() {
                        s if s.contains("completed") || s.contains("done") => Color::Green,
                        s if s.contains("progress") || s.contains("started") => Color::Yellow,
                        s if s.contains("canceled") || s.contains("cancelled") => Color::Red,
                        _ => Color::Reset,
                    };
                    Cell::new(val).fg(color)
                } else {
                    Cell::new(val)
                }
            })
            .collect();
        table.add_row(cells);
    }

    println!("{table}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_table_no_panic() {
        // Smoke test: table formatting doesn't panic
        print_table(
            &["ID", "Name"],
            &[
                vec!["1".into(), "Alice".into()],
                vec!["2".into(), "Bob".into()],
            ],
        );
    }

    #[test]
    fn test_print_table_empty() {
        print_table(&[], &[]);
    }
}
```

- [ ] **Step 4: Create detail.rs**

Create `src/output/detail.rs`:
```rust
use super::color::{bold, cyan, dim, format_priority, green, red, yellow};
use serde_json::Value;

/// Print a key-value detail line: "  Label:  value"
pub fn print_detail(label: &str, value: &str, indent: usize) {
    let pad = " ".repeat(indent * 2);
    println!("{pad}  {}: {value}", bold(label));
}

/// Print a section header.
pub fn print_section(title: &str) {
    println!("\n  {}", bold(title));
    println!("  {}", "─".repeat(title.len()));
}

/// Format a user's display name, or "-" if missing.
pub fn format_user(user: Option<&Value>) -> String {
    user.and_then(|u| u.get("displayName").or_else(|| u.get("name")))
        .and_then(|v| v.as_str())
        .unwrap_or("-")
        .to_string()
}

/// Format a health status with color.
pub fn format_health(health: &str) -> String {
    match health {
        "onTrack" => green("On Track"),
        "atRisk" => yellow("At Risk"),
        "offTrack" => red("Off Track"),
        _ => health.to_string(),
    }
}

/// Print an issue summary line (one-liner).
pub fn print_issue_summary(issue: &Value) {
    let identifier = issue
        .get("identifier")
        .and_then(|v| v.as_str())
        .unwrap_or("???");

    let status = issue
        .pointer("/state/name")
        .and_then(|v| v.as_str())
        .unwrap_or("-");

    let title = issue
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let assignee = issue
        .pointer("/assignee/displayName")
        .and_then(|v| v.as_str())
        .map(|name| dim(&format!("@{name}")))
        .unwrap_or_default();

    let priority = issue
        .get("priority")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;

    println!(
        "  {:<10} {:<14} {}  {} {}",
        bold(identifier),
        status,
        title,
        assignee,
        format_priority(priority),
    );
}

/// Print full issue details.
pub fn print_issue_detail(issue: &Value) {
    let identifier = issue
        .get("identifier")
        .and_then(|v| v.as_str())
        .unwrap_or("???");
    let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("");

    println!("\n  {} {}", bold(identifier), bold(title));
    println!();

    if let Some(state) = issue.pointer("/state/name").and_then(|v| v.as_str()) {
        print_detail("Status", state, 0);
    }

    let priority = issue
        .get("priority")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    if priority > 0 {
        let label = match priority {
            1 => "Urgent",
            2 => "High",
            3 => "Medium",
            4 => "Low",
            _ => "None",
        };
        print_detail("Priority", &format!("{} {}", label, format_priority(priority)), 0);
    }

    let assignee = format_user(issue.get("assignee"));
    print_detail("Assignee", &assignee, 0);

    if let Some(team) = issue.pointer("/team/key").and_then(|v| v.as_str()) {
        print_detail("Team", team, 0);
    }

    if let Some(project) = issue.pointer("/project/name").and_then(|v| v.as_str()) {
        print_detail("Project", project, 0);
    }

    if let Some(estimate) = issue.get("estimate").and_then(|v| v.as_f64()) {
        print_detail("Estimate", &format!("{estimate}"), 0);
    }

    if let Some(due) = issue.get("dueDate").and_then(|v| v.as_str()) {
        print_detail("Due", due, 0);
    }

    if let Some(labels) = issue.pointer("/labels/nodes").and_then(|v| v.as_array()) {
        if !labels.is_empty() {
            let names: Vec<&str> = labels
                .iter()
                .filter_map(|l| l.get("name").and_then(|n| n.as_str()))
                .collect();
            print_detail("Labels", &names.join(", "), 0);
        }
    }

    if let Some(parent) = issue.get("parent") {
        if !parent.is_null() {
            let pid = parent.get("identifier").and_then(|v| v.as_str()).unwrap_or("?");
            let ptitle = parent.get("title").and_then(|v| v.as_str()).unwrap_or("");
            print_detail("Parent", &format!("{pid} {ptitle}"), 0);
        }
    }

    if let Some(created) = issue.get("createdAt").and_then(|v| v.as_str()) {
        print_detail("Created", &dim(created), 0);
    }
    if let Some(updated) = issue.get("updatedAt").and_then(|v| v.as_str()) {
        print_detail("Updated", &dim(updated), 0);
    }

    // Description
    if let Some(desc) = issue.get("description").and_then(|v| v.as_str()) {
        if !desc.is_empty() {
            print_section("Description");
            // Render markdown in terminal
            let skin = termimad::MadSkin::default();
            let rendered = skin.term_text(desc);
            for line in rendered.to_string().lines() {
                println!("  {line}");
            }
        }
    }

    // Comments preview
    if let Some(comments) = issue.pointer("/comments/nodes").and_then(|v| v.as_array()) {
        if !comments.is_empty() {
            print_section("Comments");
            for comment in comments.iter().take(5) {
                let user = comment
                    .pointer("/user/displayName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let date = comment
                    .get("createdAt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let body = comment
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                println!("  {} {}", bold(user), dim(date));
                for line in body.lines().take(5) {
                    println!("    {line}");
                }
                println!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_user_with_display_name() {
        let user = json!({"displayName": "Alice"});
        assert_eq!(format_user(Some(&user)), "Alice");
    }

    #[test]
    fn test_format_user_with_name_fallback() {
        let user = json!({"name": "Bob"});
        assert_eq!(format_user(Some(&user)), "Bob");
    }

    #[test]
    fn test_format_user_none() {
        assert_eq!(format_user(None), "-");
    }

    #[test]
    fn test_format_health() {
        assert!(format_health("onTrack").contains("On Track"));
        assert!(format_health("atRisk").contains("At Risk"));
        assert!(format_health("offTrack").contains("Off Track"));
    }

    #[test]
    fn test_print_issue_summary_no_panic() {
        let issue = json!({
            "identifier": "ENG-123",
            "title": "Fix bug",
            "state": {"name": "In Progress"},
            "assignee": {"displayName": "Alice"},
            "priority": 2
        });
        print_issue_summary(&issue);
    }

    #[test]
    fn test_print_issue_detail_no_panic() {
        let issue = json!({
            "identifier": "ENG-123",
            "title": "Fix bug",
            "state": {"name": "In Progress"},
            "priority": 2,
            "assignee": {"displayName": "Alice"},
            "team": {"key": "ENG"},
            "project": {"name": "Backend"},
            "estimate": 3.0,
            "dueDate": "2026-04-15",
            "createdAt": "2026-04-01",
            "updatedAt": "2026-04-01",
            "labels": {"nodes": [{"name": "bug"}]},
            "parent": null,
            "description": "Something is broken",
            "comments": {"nodes": []}
        });
        print_issue_detail(&issue);
    }
}
```

- [ ] **Step 5: Create interactive.rs**

Create `src/output/interactive.rs`:
```rust
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, MultiSelect};

/// Check if we're in an interactive TTY.
pub fn is_interactive() -> bool {
    Term::stdout().is_term()
}

/// Fuzzy select from a list of items. Returns the selected index.
pub fn fuzzy_select(prompt: &str, items: &[String]) -> anyhow::Result<usize> {
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()?;
    Ok(selection)
}

/// Multi-select from a list of items. Returns selected indices.
pub fn multi_select(prompt: &str, items: &[String]) -> anyhow::Result<Vec<usize>> {
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .interact()?;
    Ok(selections)
}

/// Confirm a destructive action. Returns true if user confirms.
pub fn confirm(prompt: &str) -> anyhow::Result<bool> {
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(false)
        .interact()?;
    Ok(confirmed)
}
```

- [ ] **Step 6: Create output/mod.rs**

Create `src/output/mod.rs`:
```rust
pub mod color;
pub mod detail;
pub mod interactive;
pub mod table;

use serde::Serialize;

/// Print a value as formatted JSON.
pub fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("Error formatting JSON: {e}"),
    }
}
```

- [ ] **Step 7: Wire into main.rs**

Replace `src/main.rs`:
```rust
mod error;
mod output;

fn main() {
    println!("lin-cli");
}
```

- [ ] **Step 8: Run all tests**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test
```
Expected: all output tests pass.

- [ ] **Step 9: Commit**

```bash
git add src/output/ src/main.rs
git commit -m "feat: add output module with color, table, detail, and interactive formatting"
```

---

### Task 4: GraphQL Schema Setup

**Files:**
- Create: `schema.graphql`
- Create: `.graphql_config.yml`
- Create: `src/graphql/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Download Linear's GraphQL schema**

```bash
cd /Users/wookiedrool/src/lin-cli
# Install cynic-cli if not present
cargo install cynic-cli 2>/dev/null || true
# Download schema (requires API key)
cynic introspect -u https://api.linear.app/graphql -H "Authorization: $LINEAR_API_KEY" -o schema.graphql
```

If `cynic introspect` is unavailable, use curl:
```bash
curl -s -X POST https://api.linear.app/graphql \
  -H "Content-Type: application/json" \
  -H "Authorization: $LINEAR_API_KEY" \
  -d '{"query":"{ __schema { types { name kind fields { name type { name kind ofType { name kind ofType { name kind ofType { name } } } } } } } }"}' \
  | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin), indent=2))" > schema_introspection.json
```

Alternatively, Linear publishes their schema. Check if available at their public API docs and download directly.

- [ ] **Step 2: Create .graphql_config.yml**

Create `.graphql_config.yml`:
```yaml
schema:
  linear:
    path: schema.graphql
```

- [ ] **Step 3: Create graphql/mod.rs with schema registration**

Create `src/graphql/mod.rs`:
```rust
#[cynic::schema("linear")]
pub mod schema {}

// Re-export all graphql modules
pub mod issues;
```

Note: We start with just `issues` and add modules as we build each command group.

- [ ] **Step 4: Create a minimal issues.rs to verify cynic compiles against the schema**

Create `src/graphql/issues.rs`:
```rust
use super::schema;

// Verify schema registration works by defining a minimal query.
// We'll expand this in the issues command task.

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueByIdVariables {
    pub id: String,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
#[cynic(graphql_type = "Query", variables = "IssueByIdVariables")]
pub struct IssueById {
    #[arguments(id: $id)]
    pub issue: Issue,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct Issue {
    pub id: cynic::Id,
    pub identifier: String,
    pub title: String,
    pub priority: f64,
}
```

- [ ] **Step 5: Wire into main.rs**

Replace `src/main.rs`:
```rust
mod error;
mod graphql;
mod output;

fn main() {
    println!("lin-cli");
}
```

- [ ] **Step 6: Verify it compiles**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo build
```
Expected: compiles successfully. This validates that cynic can read the schema and generate types.

- [ ] **Step 7: Commit**

```bash
git add schema.graphql .graphql_config.yml src/graphql/ src/main.rs
git commit -m "feat: add Linear GraphQL schema and cynic registration"
```

---

### Task 5: LinearClient — Auth + Query Execution

**Files:**
- Create: `src/client/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write auth resolution tests**

Add to the bottom of `src/client/mod.rs` (we'll write the full file):

Create `src/client/mod.rs`:
```rust
pub mod cache;

use crate::error::LinearError;
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::env;
use std::path::PathBuf;
use std::time::Duration;

const API_URL: &str = "https://api.linear.app/graphql";
const MAX_RETRIES: u32 = 3;
const BACKOFF_BASE_MS: u64 = 1000;

pub struct LinearClient {
    http: Client,
    api_key: String,
    debug: bool,
}

impl LinearClient {
    pub fn new(api_key: Option<String>, debug: bool) -> Result<Self, LinearError> {
        let api_key = match api_key {
            Some(key) => key,
            None => Self::resolve_api_key()?,
        };

        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(LinearError::Request)?;

        Ok(Self {
            http,
            api_key,
            debug,
        })
    }

    fn resolve_api_key() -> Result<String, LinearError> {
        // 1. Environment variable
        if let Ok(key) = env::var("LINEAR_API_KEY") {
            if !key.is_empty() {
                return Ok(key);
            }
        }

        // 2. .env and .env.local files
        for filename in &[".env", ".env.local"] {
            for dir in Self::search_dirs() {
                let path = dir.join(filename);
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    for line in contents.lines() {
                        let line = line.trim();
                        if let Some(val) = line.strip_prefix("LINEAR_API_KEY=") {
                            let val = val.trim().trim_matches('"').trim_matches('\'');
                            if !val.is_empty() {
                                return Ok(val.to_string());
                            }
                        }
                    }
                }
            }
        }

        Err(LinearError::NoApiKey)
    }

    fn search_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(cwd) = env::current_dir() {
            dirs.push(cwd);
        }
        if let Ok(exe) = env::current_exe() {
            if let Some(parent) = exe.parent() {
                dirs.push(parent.to_path_buf());
            }
        }
        dirs
    }

    /// Execute a cynic GraphQL query and return the deserialized response data.
    pub async fn query<ResponseData, Vars>(
        &self,
        operation: cynic::Operation<ResponseData, Vars>,
    ) -> Result<ResponseData, LinearError>
    where
        ResponseData: DeserializeOwned + 'static,
        Vars: serde::Serialize,
    {
        if self.debug {
            eprintln!("--- GraphQL Query ---\n{}", operation.query);
            if let Ok(vars) = serde_json::to_string_pretty(&operation.variables) {
                eprintln!("--- Variables ---\n{vars}");
            }
        }

        let body = serde_json::json!({
            "query": operation.query,
            "variables": operation.variables,
        });

        let mut last_err = None;

        for attempt in 0..=MAX_RETRIES {
            let response = self
                .http
                .post(API_URL)
                .header("Content-Type", "application/json")
                .header("Authorization", &self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(LinearError::Request)?;

            let status = response.status();

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
                let wait = Duration::from_millis(BACKOFF_BASE_MS * 2u64.pow(attempt));
                tokio::time::sleep(wait).await;
                continue;
            }

            let response_text = response.text().await.map_err(LinearError::Request)?;

            if self.debug {
                eprintln!("--- Response ---\n{response_text}");
            }

            if !status.is_success() {
                // Try to extract GraphQL error message
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    if let Some(errors) = parsed.get("errors").and_then(|e| e.as_array()) {
                        let msgs: Vec<&str> = errors
                            .iter()
                            .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                            .collect();
                        if !msgs.is_empty() {
                            return Err(LinearError::GraphQL(msgs.join("; ")));
                        }
                    }
                }
                last_err = Some(LinearError::Http {
                    status: status.as_u16(),
                    body: response_text,
                });
                continue;
            }

            // Parse the GraphQL response
            let gql_response: serde_json::Value =
                serde_json::from_str(&response_text).map_err(LinearError::Json)?;

            // Check for GraphQL errors
            if let Some(errors) = gql_response.get("errors").and_then(|e| e.as_array()) {
                if !errors.is_empty() {
                    let msgs: Vec<&str> = errors
                        .iter()
                        .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                        .collect();
                    return Err(LinearError::GraphQL(msgs.join("; ")));
                }
            }

            // Extract "data" field and deserialize into ResponseData
            let data = gql_response
                .get("data")
                .ok_or_else(|| LinearError::GraphQL("No 'data' field in response".into()))?;

            let result: ResponseData =
                serde_json::from_value(data.clone()).map_err(LinearError::Json)?;

            return Ok(result);
        }

        Err(last_err.unwrap_or(LinearError::GraphQL("Max retries exceeded".into())))
    }

    /// Execute a raw GraphQL query string, returning the raw JSON Value.
    /// Useful for commands that need the raw response for --json output.
    pub async fn query_raw(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, LinearError> {
        if self.debug {
            eprintln!("--- GraphQL Query ---\n{query}");
            if let Some(ref vars) = variables {
                eprintln!(
                    "--- Variables ---\n{}",
                    serde_json::to_string_pretty(vars).unwrap_or_default()
                );
            }
        }

        let mut body = serde_json::json!({"query": query});
        if let Some(vars) = &variables {
            body["variables"] = vars.clone();
        }

        let mut last_err = None;

        for attempt in 0..=MAX_RETRIES {
            let response = self
                .http
                .post(API_URL)
                .header("Content-Type", "application/json")
                .header("Authorization", &self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(LinearError::Request)?;

            let status = response.status();

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
                let wait = Duration::from_millis(BACKOFF_BASE_MS * 2u64.pow(attempt));
                tokio::time::sleep(wait).await;
                continue;
            }

            let response_text = response.text().await.map_err(LinearError::Request)?;

            if self.debug {
                eprintln!("--- Response ---\n{response_text}");
            }

            if !status.is_success() {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    if let Some(errors) = parsed.get("errors").and_then(|e| e.as_array()) {
                        let msgs: Vec<&str> = errors
                            .iter()
                            .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                            .collect();
                        if !msgs.is_empty() {
                            return Err(LinearError::GraphQL(msgs.join("; ")));
                        }
                    }
                }
                last_err = Some(LinearError::Http {
                    status: status.as_u16(),
                    body: response_text,
                });
                continue;
            }

            let gql_response: serde_json::Value =
                serde_json::from_str(&response_text).map_err(LinearError::Json)?;

            if let Some(errors) = gql_response.get("errors").and_then(|e| e.as_array()) {
                if !errors.is_empty() {
                    let msgs: Vec<&str> = errors
                        .iter()
                        .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                        .collect();
                    return Err(LinearError::GraphQL(msgs.join("; ")));
                }
            }

            return Ok(gql_response);
        }

        Err(last_err.unwrap_or(LinearError::GraphQL("Max retries exceeded".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_api_key_from_env() {
        temp_env::with_var("LINEAR_API_KEY", Some("lin_api_test123"), || {
            let key = LinearClient::resolve_api_key().unwrap();
            assert_eq!(key, "lin_api_test123");
        });
    }

    #[test]
    fn test_resolve_api_key_empty_env() {
        temp_env::with_var("LINEAR_API_KEY", Some(""), || {
            // Empty env should fall through
            // Without .env files, this should error
            let result = LinearClient::resolve_api_key();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_resolve_api_key_missing() {
        temp_env::with_var_unset("LINEAR_API_KEY", || {
            let result = LinearClient::resolve_api_key();
            assert!(result.is_err());
            let err = result.unwrap_err().to_string();
            assert!(err.contains("LINEAR_API_KEY not found"));
        });
    }

    #[test]
    fn test_new_with_explicit_key() {
        let client = LinearClient::new(Some("lin_api_explicit".into()), false);
        assert!(client.is_ok());
    }
}
```

- [ ] **Step 2: Create stub cache.rs**

Create `src/client/cache.rs`:
```rust
// Cache module — will be populated in Task 6
```

- [ ] **Step 3: Wire into main.rs**

Replace `src/main.rs`:
```rust
mod client;
mod error;
mod graphql;
mod output;

fn main() {
    println!("lin-cli");
}
```

- [ ] **Step 4: Run tests**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test client::
```
Expected: all 4 auth tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/client/ src/main.rs
git commit -m "feat: add LinearClient with auth resolution, query execution, and retry logic"
```

---

### Task 6: Client Cache

**Files:**
- Modify: `src/client/cache.rs`
- Modify: `src/client/mod.rs`

- [ ] **Step 1: Implement cache.rs**

Replace `src/client/cache.rs`:
```rust
use crate::error::LinearError;
use dashmap::DashMap;
use serde::Deserialize;
use tokio::sync::OnceCell;

#[derive(Debug, Deserialize, Clone)]
pub struct CachedTeam {
    pub id: String,
    pub name: String,
    pub key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CachedUser {
    pub id: String,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub active: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CachedProject {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CachedState {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CachedLabel {
    pub id: String,
    pub name: String,
}

pub struct Cache {
    pub teams: OnceCell<Vec<CachedTeam>>,
    pub users: OnceCell<Vec<CachedUser>>,
    pub projects: OnceCell<Vec<CachedProject>>,
    pub states: DashMap<String, Vec<CachedState>>,
    pub labels: DashMap<String, Vec<CachedLabel>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            teams: OnceCell::new(),
            users: OnceCell::new(),
            projects: OnceCell::new(),
            states: DashMap::new(),
            labels: DashMap::new(),
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve a team key or name to UUID. Case-insensitive.
pub fn find_team<'a>(teams: &'a [CachedTeam], key_or_name: &str) -> Result<&'a CachedTeam, LinearError> {
    let needle = key_or_name.to_lowercase();
    teams
        .iter()
        .find(|t| t.key.to_lowercase() == needle || t.name.to_lowercase() == needle)
        .ok_or_else(|| LinearError::NotFound {
            entity: "Team",
            name: key_or_name.to_string(),
        })
}

/// Resolve a user display name to UUID. Case-insensitive partial match.
pub fn find_user<'a>(users: &'a [CachedUser], name: &str) -> Result<&'a CachedUser, LinearError> {
    let needle = name.to_lowercase();
    users
        .iter()
        .find(|u| {
            let display = u
                .display_name
                .as_deref()
                .or(u.name.as_deref())
                .unwrap_or("")
                .to_lowercase();
            display == needle || display.contains(&needle)
        })
        .ok_or_else(|| LinearError::NotFound {
            entity: "User",
            name: name.to_string(),
        })
}

/// Resolve a project name to UUID. Case-insensitive partial match.
pub fn find_project<'a>(
    projects: &'a [CachedProject],
    name: &str,
) -> Result<&'a CachedProject, LinearError> {
    let needle = name.to_lowercase();
    projects
        .iter()
        .find(|p| p.name.to_lowercase().contains(&needle))
        .ok_or_else(|| LinearError::NotFound {
            entity: "Project",
            name: name.to_string(),
        })
}

/// Resolve a workflow state name to UUID. Case-insensitive exact match.
pub fn find_state<'a>(
    states: &'a [CachedState],
    state_name: &str,
) -> Result<&'a CachedState, LinearError> {
    let needle = state_name.to_lowercase();
    states
        .iter()
        .find(|s| s.name.to_lowercase() == needle)
        .ok_or_else(|| LinearError::NotFound {
            entity: "State",
            name: state_name.to_string(),
        })
}

/// Resolve label names to UUIDs. Case-insensitive exact match.
pub fn find_labels(
    labels: &[CachedLabel],
    names: &[&str],
) -> Result<Vec<String>, LinearError> {
    names
        .iter()
        .map(|name| {
            let needle = name.to_lowercase().trim().to_string();
            labels
                .iter()
                .find(|l| l.name.to_lowercase() == needle)
                .map(|l| l.id.clone())
                .ok_or_else(|| LinearError::NotFound {
                    entity: "Label",
                    name: name.to_string(),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_teams() -> Vec<CachedTeam> {
        vec![
            CachedTeam { id: "t1".into(), name: "Engineering".into(), key: "ENG".into() },
            CachedTeam { id: "t2".into(), name: "Design".into(), key: "DES".into() },
        ]
    }

    fn test_users() -> Vec<CachedUser> {
        vec![
            CachedUser {
                id: "u1".into(),
                name: Some("Alice Smith".into()),
                display_name: Some("Alice".into()),
                email: None,
                active: true,
            },
            CachedUser {
                id: "u2".into(),
                name: Some("Bob Jones".into()),
                display_name: None,
                email: None,
                active: true,
            },
        ]
    }

    #[test]
    fn test_find_team_by_key() {
        let teams = test_teams();
        let team = find_team(&teams, "ENG").unwrap();
        assert_eq!(team.id, "t1");
    }

    #[test]
    fn test_find_team_by_name_case_insensitive() {
        let teams = test_teams();
        let team = find_team(&teams, "engineering").unwrap();
        assert_eq!(team.id, "t1");
    }

    #[test]
    fn test_find_team_not_found() {
        let teams = test_teams();
        let result = find_team(&teams, "NOPE");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_user_by_display_name() {
        let users = test_users();
        let user = find_user(&users, "Alice").unwrap();
        assert_eq!(user.id, "u1");
    }

    #[test]
    fn test_find_user_partial_match() {
        let users = test_users();
        let user = find_user(&users, "ali").unwrap();
        assert_eq!(user.id, "u1");
    }

    #[test]
    fn test_find_user_falls_back_to_name() {
        let users = test_users();
        let user = find_user(&users, "Bob Jones").unwrap();
        assert_eq!(user.id, "u2");
    }

    #[test]
    fn test_find_labels_all_found() {
        let labels = vec![
            CachedLabel { id: "l1".into(), name: "bug".into() },
            CachedLabel { id: "l2".into(), name: "feature".into() },
        ];
        let ids = find_labels(&labels, &["bug", "feature"]).unwrap();
        assert_eq!(ids, vec!["l1", "l2"]);
    }

    #[test]
    fn test_find_labels_one_missing() {
        let labels = vec![CachedLabel { id: "l1".into(), name: "bug".into() }];
        let result = find_labels(&labels, &["bug", "nope"]);
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run cache tests**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test client::cache
```
Expected: all 8 cache tests pass.

- [ ] **Step 3: Commit**

```bash
git add src/client/cache.rs
git commit -m "feat: add client cache with team/user/project/state/label lookups"
```

---

### Task 7: CLI Structure + Clap

**Files:**
- Create: `src/cli.rs`
- Create: `src/commands/mod.rs`
- Create: `src/commands/issues.rs` (stub)
- Modify: `src/main.rs`

- [ ] **Step 1: Create cli.rs with top-level clap structure**

Create `src/cli.rs`:
```rust
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "lin",
    about = "lin — Linear CLI. Manage issues, projects, cycles, and more.",
    version
)]
pub struct Cli {
    /// Output raw JSON for scripting
    #[arg(long, global = true)]
    pub json: bool,

    /// Print GraphQL queries/responses to stderr
    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage issues
    Issues(crate::commands::issues::IssuesArgs),
    // Future command groups will be added here as they're implemented:
    // Projects, Cycles, Roadmap, Labels, Teams, Relations, Customers, Views, Docs, Notifications
}
```

- [ ] **Step 2: Create commands/mod.rs**

Create `src/commands/mod.rs`:
```rust
pub mod issues;
```

- [ ] **Step 3: Create stub commands/issues.rs**

Create `src/commands/issues.rs`:
```rust
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct IssuesArgs {
    #[command(subcommand)]
    pub command: IssuesCommand,
}

#[derive(Subcommand, Debug)]
pub enum IssuesCommand {
    /// Get issue details
    Get {
        /// Issue identifier (e.g., ENG-123)
        identifier: String,
    },
    /// List issues
    List {
        /// Filter by team key or name
        #[arg(long)]
        team: Option<String>,
        /// Filter by status name
        #[arg(long)]
        status: Option<String>,
        /// Filter by assignee name
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by priority (1=Urgent, 2=High, 3=Medium, 4=Low)
        #[arg(long)]
        priority: Option<i32>,
        /// Filter by label name
        #[arg(long)]
        label: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// Search issues by text
    Search {
        /// Search query
        query: String,
        /// Filter by team
        #[arg(long)]
        team: Option<String>,
        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,
    },
    /// Create a new issue
    Create {
        /// Team key or name (required)
        #[arg(long)]
        team: String,
        /// Issue title (required)
        #[arg(long)]
        title: String,
        /// Issue description
        #[arg(long)]
        description: Option<String>,
        /// Assignee name
        #[arg(long)]
        assignee: Option<String>,
        /// Priority (1=Urgent, 2=High, 3=Medium, 4=Low)
        #[arg(long)]
        priority: Option<i32>,
        /// Story points estimate
        #[arg(long)]
        estimate: Option<f64>,
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due_date: Option<String>,
        /// Label name
        #[arg(long)]
        label: Option<String>,
        /// Parent issue identifier
        #[arg(long)]
        parent: Option<String>,
        /// Project name
        #[arg(long)]
        project: Option<String>,
        /// Initial status
        #[arg(long)]
        status: Option<String>,
    },
    /// Update an issue
    Update {
        /// Issue identifier
        identifier: String,
        /// New status
        #[arg(long)]
        status: Option<String>,
        /// New assignee
        #[arg(long)]
        assignee: Option<String>,
        /// New priority
        #[arg(long)]
        priority: Option<i32>,
        /// New estimate
        #[arg(long)]
        estimate: Option<f64>,
        /// New due date
        #[arg(long)]
        due_date: Option<String>,
        /// Parent issue identifier
        #[arg(long)]
        parent: Option<String>,
        /// Project name
        #[arg(long)]
        project: Option<String>,
        /// Label name
        #[arg(long)]
        label: Option<String>,
        /// Milestone name
        #[arg(long)]
        milestone: Option<String>,
    },
    /// Add a comment to an issue
    Comment {
        /// Issue identifier
        identifier: String,
        /// Comment body
        body: String,
    },
    /// Archive an issue
    Archive {
        /// Issue identifier
        identifier: String,
    },
}

pub async fn execute(
    args: &IssuesArgs,
    json: bool,
    debug: bool,
) -> anyhow::Result<()> {
    let client = crate::client::LinearClient::new(None, debug)?;

    match &args.command {
        IssuesCommand::Get { identifier } => {
            // TODO: implement in Task 9
            println!("issues get {identifier} — not yet implemented");
        }
        IssuesCommand::List { team, status, assignee, priority, label, limit } => {
            println!("issues list — not yet implemented");
        }
        IssuesCommand::Search { query, team, limit } => {
            println!("issues search '{query}' — not yet implemented");
        }
        IssuesCommand::Create { team, title, .. } => {
            println!("issues create '{title}' in {team} — not yet implemented");
        }
        IssuesCommand::Update { identifier, .. } => {
            println!("issues update {identifier} — not yet implemented");
        }
        IssuesCommand::Comment { identifier, body } => {
            println!("issues comment {identifier} — not yet implemented");
        }
        IssuesCommand::Archive { identifier } => {
            println!("issues archive {identifier} — not yet implemented");
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Wire everything into main.rs**

Replace `src/main.rs`:
```rust
mod cli;
mod client;
mod commands;
mod error;
mod graphql;
mod output;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Issues(args) => commands::issues::execute(args, cli.json, cli.debug).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

- [ ] **Step 5: Verify it compiles and help works**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo build && ./target/debug/lin --help
```
Expected: prints help showing "issues" subcommand.

```bash
./target/debug/lin issues --help
```
Expected: prints help showing all issue subcommands.

- [ ] **Step 6: Commit**

```bash
git add src/cli.rs src/commands/ src/main.rs
git commit -m "feat: add clap CLI structure with issues command group"
```

---

### Task 8: Smoke Tests

**Files:**
- Create: `tests/cli_smoke.rs`

- [ ] **Step 1: Write smoke tests**

Create `tests/cli_smoke.rs`:
```rust
use assert_cmd::Command;
use predicates::prelude::*;

fn lin() -> Command {
    Command::cargo_bin("lin").unwrap()
}

#[test]
fn test_help() {
    lin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Linear CLI"));
}

#[test]
fn test_version() {
    lin()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("2026.4.0"));
}

#[test]
fn test_issues_help() {
    lin()
        .args(["issues", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("comment"))
        .stdout(predicate::str::contains("archive"));
}

#[test]
fn test_issues_get_help() {
    lin()
        .args(["issues", "get", "--help"])
        .assert()
        .success();
}

#[test]
fn test_issues_list_help() {
    lin()
        .args(["issues", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--team"))
        .stdout(predicate::str::contains("--status"))
        .stdout(predicate::str::contains("--assignee"))
        .stdout(predicate::str::contains("--priority"))
        .stdout(predicate::str::contains("--label"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_issues_create_help() {
    lin()
        .args(["issues", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--team"))
        .stdout(predicate::str::contains("--title"));
}

#[test]
fn test_issues_get_missing_arg() {
    lin()
        .args(["issues", "get"])
        .assert()
        .failure();
}

#[test]
fn test_unknown_command() {
    lin()
        .arg("nonexistent")
        .assert()
        .failure();
}

#[test]
fn test_json_flag_accepted() {
    lin()
        .args(["--json", "issues", "--help"])
        .assert()
        .success();
}

#[test]
fn test_debug_flag_accepted() {
    lin()
        .args(["--debug", "issues", "--help"])
        .assert()
        .success();
}
```

- [ ] **Step 2: Run smoke tests**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test --test cli_smoke
```
Expected: all 10 tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/cli_smoke.rs
git commit -m "test: add CLI smoke tests for help, flags, and argument validation"
```

---

### Task 9: Issues Command — Full Implementation

**Files:**
- Modify: `src/graphql/issues.rs`
- Modify: `src/commands/issues.rs`
- Modify: `src/client/mod.rs` (add cache population methods)

This is the largest task — it establishes the pattern for all subsequent command groups.

- [ ] **Step 1: Expand graphql/issues.rs with all queries and mutations**

This step depends on the actual Linear schema. Once the schema is downloaded (Task 4), the cynic derive macros will validate against it. The exact field names and types must match the schema.

Replace `src/graphql/issues.rs` with the full set of issue queries. The pattern for each query:

```rust
use super::schema;

// --- Variables ---

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueByIdVariables {
    pub id: String,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueListVariables {
    pub filter: Option<IssueFilter>,
    pub first: Option<i32>,
    pub after: Option<String>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueSearchVariables {
    pub term: String,
    pub first: Option<i32>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueCreateVariables {
    pub input: IssueCreateInput,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueUpdateVariables {
    pub id: String,
    pub input: IssueUpdateInput,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct CommentCreateVariables {
    pub input: CommentCreateInput,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct IssueArchiveVariables {
    pub id: String,
}

// --- Input types ---
// These must match Linear's schema exactly. The field names below are based
// on the Python CLI's usage. Adjust types after downloading schema.

#[derive(cynic::InputObject, Debug)]
pub struct IssueFilter {
    pub team: Option<TeamFilter>,
    pub state: Option<StateFilter>,
    pub assignee: Option<AssigneeFilter>,
    pub priority: Option<NumberComparator>,
    pub labels: Option<LabelFilter>,
}

// Additional input/filter types will be derived from the schema.
// cynic will error at compile time if any field is wrong.

// --- Query fragments ---

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
#[cynic(graphql_type = "Query", variables = "IssueByIdVariables")]
pub struct GetIssue {
    #[arguments(id: $id)]
    pub issue: IssueDetail,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
#[cynic(graphql_type = "Issue")]
pub struct IssueDetail {
    pub id: cynic::Id,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub state: WorkflowState,
    pub priority: f64,
    pub assignee: Option<User>,
    pub team: Team,
    pub project: Option<Project>,
    pub estimate: Option<f64>,
    pub due_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub labels: LabelConnection,
    pub parent: Option<ParentIssue>,
    #[arguments(first: 10)]
    pub comments: CommentConnection,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
#[cynic(graphql_type = "Issue")]
pub struct IssueSummary {
    pub id: cynic::Id,
    pub identifier: String,
    pub title: String,
    pub state: WorkflowState,
    pub assignee: Option<User>,
    pub priority: f64,
    pub team: TeamKey,
}

// Shared sub-fragments: WorkflowState, User, Team, Project, Label, Comment, etc.
// These are reused across all command groups.

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct WorkflowState {
    pub id: cynic::Id,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct User {
    pub id: cynic::Id,
    pub display_name: String,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct Team {
    pub id: cynic::Id,
    pub key: String,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct TeamKey {
    pub key: String,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct Project {
    pub id: cynic::Id,
    pub name: String,
}

// Connection types for pagination
#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct LabelConnection {
    pub nodes: Vec<Label>,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct Label {
    pub id: cynic::Id,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct CommentConnection {
    pub nodes: Vec<Comment>,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
pub struct Comment {
    pub id: cynic::Id,
    pub body: String,
    pub created_at: String,
    pub user: Option<User>,
}

#[derive(cynic::QueryFragment, Debug, serde::Serialize)]
#[cynic(graphql_type = "Issue")]
pub struct ParentIssue {
    pub id: cynic::Id,
    pub identifier: String,
    pub title: String,
}
```

**Note:** The exact types (especially `IssueFilter`, `IssueCreateInput`, `IssueUpdateInput`, `CommentCreateInput`, and their sub-types) must be derived from the downloaded schema. cynic will produce compile errors if any field name or type doesn't match. The implementer should use `cynic querygen` or read the schema to get exact types.

- [ ] **Step 2: Implement commands/issues.rs execute functions**

Replace the stub `execute` function in `src/commands/issues.rs` with full implementations. Each match arm should:
1. Build the cynic operation
2. Call `client.query(operation)` 
3. Format output (json mode: `print_json`, human mode: `print_issue_detail` / `print_issue_summary` / `print_table`)

The pattern for `get`:
```rust
IssuesCommand::Get { identifier } => {
    use crate::graphql::issues::*;
    let op = GetIssue::build(IssueByIdVariables {
        id: identifier.clone(),
    });
    let result = client.query(op).await?;
    if json {
        output::print_json(&result.issue);
    } else {
        // Convert to serde_json::Value for the detail printer
        let val = serde_json::to_value(&result.issue)?;
        output::detail::print_issue_detail(&val);
    }
}
```

Follow this pattern for all 7 subcommands: get, list, search, create, update, comment, archive.

- [ ] **Step 3: Add cache population methods to LinearClient**

Add these methods to `src/client/mod.rs`:

```rust
impl LinearClient {
    pub async fn get_teams(&self) -> Result<&[cache::CachedTeam], LinearError> {
        self.cache.teams.get_or_try_init(|| async {
            let result = self.query_raw(
                "query { teams { nodes { id name key } } }",
                None,
            ).await?;
            let nodes = result
                .pointer("/data/teams/nodes")
                .ok_or_else(|| LinearError::GraphQL("No teams data".into()))?;
            Ok(serde_json::from_value(nodes.clone())?)
        }).await.map(|v| v.as_slice())
    }

    pub async fn get_team_id(&self, key_or_name: &str) -> Result<String, LinearError> {
        let teams = self.get_teams().await?;
        Ok(cache::find_team(teams, key_or_name)?.id.clone())
    }

    pub async fn get_users(&self) -> Result<&[cache::CachedUser], LinearError> {
        self.cache.users.get_or_try_init(|| async {
            let result = self.query_raw(
                "query { users(first: 250) { nodes { id name displayName email active } } }",
                None,
            ).await?;
            let nodes = result
                .pointer("/data/users/nodes")
                .ok_or_else(|| LinearError::GraphQL("No users data".into()))?;
            Ok(serde_json::from_value(nodes.clone())?)
        }).await.map(|v| v.as_slice())
    }

    pub async fn get_user_id(&self, name: &str) -> Result<String, LinearError> {
        let users = self.get_users().await?;
        Ok(cache::find_user(users, name)?.id.clone())
    }

    pub async fn get_project_id(&self, name: &str) -> Result<String, LinearError> {
        let projects = self.cache.projects.get_or_try_init(|| async {
            let result = self.query_raw(
                "query { projects(first: 250) { nodes { id name } } }",
                None,
            ).await?;
            let nodes = result
                .pointer("/data/projects/nodes")
                .ok_or_else(|| LinearError::GraphQL("No projects data".into()))?;
            Ok(serde_json::from_value(nodes.clone())?)
        }).await?;
        Ok(cache::find_project(projects, name)?.id.clone())
    }

    pub async fn get_state_id(&self, team_key: &str, state_name: &str) -> Result<String, LinearError> {
        if !self.cache.states.contains_key(team_key) {
            let team_id = self.get_team_id(team_key).await?;
            let result = self.query_raw(
                "query($teamId: ID!) { workflowStates(filter: { team: { id: { eq: $teamId } } }) { nodes { id name type } } }",
                Some(serde_json::json!({"teamId": team_id})),
            ).await?;
            let nodes = result
                .pointer("/data/workflowStates/nodes")
                .ok_or_else(|| LinearError::GraphQL("No states data".into()))?;
            let states: Vec<cache::CachedState> = serde_json::from_value(nodes.clone())?;
            self.cache.states.insert(team_key.to_string(), states);
        }
        let entry = self.cache.states.get(team_key).unwrap();
        Ok(cache::find_state(entry.value(), state_name)?.id.clone())
    }

    pub async fn get_label_ids(&self, names: &[&str], team_key: Option<&str>) -> Result<Vec<String>, LinearError> {
        let cache_key = team_key.unwrap_or("__workspace__");
        if !self.cache.labels.contains_key(cache_key) {
            let (query, variables) = if let Some(tk) = team_key {
                let team_id = self.get_team_id(tk).await?;
                (
                    "query($teamId: ID!) { issueLabels(filter: { team: { id: { eq: $teamId } } }) { nodes { id name } } }",
                    Some(serde_json::json!({"teamId": team_id})),
                )
            } else {
                ("query { issueLabels { nodes { id name } } }", None)
            };
            let result = self.query_raw(query, variables).await?;
            let nodes = result
                .pointer("/data/issueLabels/nodes")
                .ok_or_else(|| LinearError::GraphQL("No labels data".into()))?;
            let labels: Vec<cache::CachedLabel> = serde_json::from_value(nodes.clone())?;
            self.cache.labels.insert(cache_key.to_string(), labels);
        }
        let entry = self.cache.labels.get(cache_key).unwrap();
        cache::find_labels(entry.value(), names)
    }
}
```

Also add `cache: Cache` to `LinearClient::new`:
```rust
pub fn new(api_key: Option<String>, debug: bool) -> Result<Self, LinearError> {
    // ... existing code ...
    Ok(Self {
        http,
        api_key,
        debug,
        cache: cache::Cache::new(),
    })
}
```

And add the `cache` field to the struct:
```rust
pub struct LinearClient {
    http: Client,
    api_key: String,
    debug: bool,
    cache: cache::Cache,
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo build
```

- [ ] **Step 5: Run all tests**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test
```

- [ ] **Step 6: Manual smoke test with real API**

```bash
export LINEAR_API_KEY="..."
./target/debug/lin issues list --team ENG --limit 5
./target/debug/lin issues get ENG-123
./target/debug/lin --json issues list --team ENG --limit 2
```

- [ ] **Step 7: Commit**

```bash
git add src/graphql/issues.rs src/commands/issues.rs src/client/
git commit -m "feat: implement issues command group with all 7 subcommands"
```

---

### Task 10: Projects Command Group

**Files:**
- Create: `src/graphql/projects.rs`
- Create: `src/commands/projects.rs`
- Modify: `src/cli.rs` (add Projects variant)
- Modify: `src/commands/mod.rs`
- Modify: `src/graphql/mod.rs`

- [ ] **Step 1: Define clap args for projects**

Create `src/commands/projects.rs`:
```rust
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct ProjectsArgs {
    #[command(subcommand)]
    pub command: ProjectsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ProjectsCommand {
    /// Get project details
    Get {
        /// Project name (partial match)
        name: String,
    },
    /// List projects
    List {
        /// Filter by status (backlog/planned/started/paused/completed/canceled)
        #[arg(long)]
        status: Option<String>,
        /// Filter by team
        #[arg(long)]
        team: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// List issues in a project
    Issues {
        /// Project name
        name: String,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// Create a project
    Create {
        /// Project name
        #[arg(long)]
        name: String,
        /// Team key
        #[arg(long)]
        team: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Lead name
        #[arg(long)]
        lead: Option<String>,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: Option<String>,
        /// Target date (YYYY-MM-DD)
        #[arg(long)]
        target_date: Option<String>,
    },
    /// Update a project
    Update {
        /// Project name
        name: String,
        /// New status
        #[arg(long)]
        status: Option<String>,
        /// New lead
        #[arg(long)]
        lead: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New start date
        #[arg(long)]
        start_date: Option<String>,
        /// New target date
        #[arg(long)]
        target_date: Option<String>,
    },
}

pub async fn execute(
    args: &ProjectsArgs,
    json: bool,
    debug: bool,
) -> anyhow::Result<()> {
    let client = crate::client::LinearClient::new(None, debug)?;

    match &args.command {
        ProjectsCommand::Get { name } => {
            let project_id = client.get_project_id(name).await?;
            let result = client.query_raw(
                r#"query($id: String!) {
                    project(id: $id) {
                        id name description state
                        lead { displayName }
                        startDate targetDate
                        teams { nodes { key name } }
                        members { nodes { displayName } }
                        issues { nodes { id } }
                        projectUpdates(first: 3) {
                            nodes { body health createdAt user { displayName } }
                        }
                    }
                }"#,
                Some(serde_json::json!({"id": project_id})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else {
                let project = &result["data"]["project"];
                crate::output::detail::print_detail("Name", project["name"].as_str().unwrap_or("-"), 0);
                crate::output::detail::print_detail("Status", project["state"].as_str().unwrap_or("-"), 0);
                if let Some(lead) = project.pointer("/lead/displayName").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Lead", lead, 0);
                }
                if let Some(start) = project["startDate"].as_str() {
                    crate::output::detail::print_detail("Start", start, 0);
                }
                if let Some(target) = project["targetDate"].as_str() {
                    crate::output::detail::print_detail("Target", target, 0);
                }
                if let Some(teams) = project.pointer("/teams/nodes").and_then(|v| v.as_array()) {
                    let names: Vec<&str> = teams.iter().filter_map(|t| t["key"].as_str()).collect();
                    crate::output::detail::print_detail("Teams", &names.join(", "), 0);
                }
                if let Some(count) = project.pointer("/issues/nodes").and_then(|v| v.as_array()) {
                    crate::output::detail::print_detail("Issues", &count.len().to_string(), 0);
                }
            }
        }
        ProjectsCommand::List { status, team, limit } => {
            // Build filter
            let mut filter = serde_json::Map::new();
            if let Some(s) = status {
                filter.insert("state".into(), serde_json::json!({"eq": s}));
            }
            let variables = serde_json::json!({
                "first": limit,
                "filter": if filter.is_empty() { serde_json::Value::Null } else { serde_json::Value::Object(filter) },
            });
            let result = client.query_raw(
                r#"query($first: Int!, $filter: ProjectFilter) {
                    projects(first: $first, filter: $filter) {
                        nodes { id name state lead { displayName } startDate targetDate }
                    }
                }"#,
                Some(variables),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result.pointer("/data/projects/nodes").and_then(|v| v.as_array());
                if let Some(projects) = nodes {
                    let headers = &["Name", "Status", "Lead", "Start", "Target"];
                    let rows: Vec<Vec<String>> = projects.iter().map(|p| {
                        vec![
                            p["name"].as_str().unwrap_or("-").to_string(),
                            p["state"].as_str().unwrap_or("-").to_string(),
                            p.pointer("/lead/displayName").and_then(|v| v.as_str()).unwrap_or("-").to_string(),
                            p["startDate"].as_str().unwrap_or("-").to_string(),
                            p["targetDate"].as_str().unwrap_or("-").to_string(),
                        ]
                    }).collect();
                    crate::output::table::print_table(headers, &rows);
                }
            }
        }
        ProjectsCommand::Issues { name, status, limit } => {
            let project_id = client.get_project_id(name).await?;
            let mut filter = serde_json::json!({"project": {"id": {"eq": project_id}}});
            if let Some(s) = status {
                filter["state"] = serde_json::json!({"name": {"eq": s}});
            }
            let result = client.query_raw(
                r#"query($filter: IssueFilter!, $first: Int!) {
                    issues(filter: $filter, first: $first, orderBy: updatedAt) {
                        nodes {
                            id identifier title
                            state { name }
                            assignee { displayName }
                            priority
                            team { key }
                        }
                    }
                }"#,
                Some(serde_json::json!({"filter": filter, "first": limit})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else if let Some(issues) = result.pointer("/data/issues/nodes").and_then(|v| v.as_array()) {
                for issue in issues {
                    crate::output::detail::print_issue_summary(issue);
                }
            }
        }
        ProjectsCommand::Create { name, team, description, lead, start_date, target_date } => {
            let team_id = client.get_team_id(team).await?;
            let mut input = serde_json::json!({"name": name, "teamIds": [team_id]});
            if let Some(d) = description { input["description"] = serde_json::json!(d); }
            if let Some(l) = lead {
                let uid = client.get_user_id(l).await?;
                input["leadId"] = serde_json::json!(uid);
            }
            if let Some(s) = start_date { input["startDate"] = serde_json::json!(s); }
            if let Some(t) = target_date { input["targetDate"] = serde_json::json!(t); }
            let result = client.query_raw(
                r#"mutation($input: ProjectCreateInput!) {
                    projectCreate(input: $input) {
                        success
                        project { id name state }
                    }
                }"#,
                Some(serde_json::json!({"input": input})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else {
                println!("{}", crate::output::color::green(&format!("Created project: {name}")));
            }
        }
        ProjectsCommand::Update { name, status, lead, description, start_date, target_date } => {
            let project_id = client.get_project_id(name).await?;
            let mut input = serde_json::Map::new();
            if let Some(s) = status { input.insert("state".into(), serde_json::json!(s)); }
            if let Some(l) = lead {
                let uid = client.get_user_id(l).await?;
                input.insert("leadId".into(), serde_json::json!(uid));
            }
            if let Some(d) = description { input.insert("description".into(), serde_json::json!(d)); }
            if let Some(s) = start_date { input.insert("startDate".into(), serde_json::json!(s)); }
            if let Some(t) = target_date { input.insert("targetDate".into(), serde_json::json!(t)); }
            let result = client.query_raw(
                r#"mutation($id: String!, $input: ProjectUpdateInput!) {
                    projectUpdate(id: $id, input: $input) {
                        success
                        project { id name state }
                    }
                }"#,
                Some(serde_json::json!({"id": project_id, "input": input})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else {
                println!("{}", crate::output::color::green(&format!("Updated project: {name}")));
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 2: Create graphql/projects.rs stub**

Create `src/graphql/projects.rs`:
```rust
// Project GraphQL types — using raw queries via client.query_raw() for now.
// Will migrate to cynic typed queries as patterns stabilize.
```

- [ ] **Step 3: Register in cli.rs, commands/mod.rs, graphql/mod.rs**

Add to `src/commands/mod.rs`:
```rust
pub mod issues;
pub mod projects;
```

Add to `src/graphql/mod.rs`:
```rust
pub mod projects;
```

Add to `src/cli.rs` Commands enum:
```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage issues
    Issues(crate::commands::issues::IssuesArgs),
    /// Manage projects
    Projects(crate::commands::projects::ProjectsArgs),
}
```

Add to `src/main.rs` match:
```rust
let result = match &cli.command {
    Commands::Issues(args) => commands::issues::execute(args, cli.json, cli.debug).await,
    Commands::Projects(args) => commands::projects::execute(args, cli.json, cli.debug).await,
};
```

- [ ] **Step 4: Add smoke tests for projects**

Append to `tests/cli_smoke.rs`:
```rust
#[test]
fn test_projects_help() {
    lin()
        .args(["projects", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("issues"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"));
}
```

- [ ] **Step 5: Run all tests**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test
```

- [ ] **Step 6: Commit**

```bash
git add src/commands/projects.rs src/graphql/projects.rs src/cli.rs src/commands/mod.rs src/graphql/mod.rs src/main.rs tests/cli_smoke.rs
git commit -m "feat: implement projects command group with all 5 subcommands"
```

---

### Task 11: Cycles Command Group

**Files:**
- Create: `src/commands/cycles.rs`
- Create: `src/graphql/cycles.rs`
- Modify: `src/cli.rs`, `src/commands/mod.rs`, `src/graphql/mod.rs`, `src/main.rs`
- Modify: `tests/cli_smoke.rs`

Follow the same pattern as Task 10. Subcommands:
- `list --team --type[current/previous/next/all]`
- `get <cycle_id>`
- `issues --team`
- `create --team --name --start --end`
- `update <cycle_id> --name --start --end`
- `add <issue_id> --team`
- `remove <issue_id>`

- [ ] **Step 1: Create commands/cycles.rs with all 7 subcommands**

Create `src/commands/cycles.rs`:
```rust
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct CyclesArgs {
    #[command(subcommand)]
    pub command: CyclesCommand,
}

#[derive(Subcommand, Debug)]
pub enum CyclesCommand {
    /// List cycles for a team
    List {
        #[arg(long)]
        team: String,
        /// Filter: current, previous, next, all
        #[arg(long, default_value = "all")]
        r#type: String,
    },
    /// Get cycle details
    Get { cycle_id: String },
    /// List issues in current cycle
    Issues {
        #[arg(long)]
        team: String,
    },
    /// Create a cycle
    Create {
        #[arg(long)]
        team: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
    },
    /// Update a cycle
    Update {
        cycle_id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
    },
    /// Add issue to current cycle
    Add {
        issue_id: String,
        #[arg(long)]
        team: String,
    },
    /// Remove issue from its cycle
    Remove { issue_id: String },
}

pub async fn execute(args: &CyclesArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = crate::client::LinearClient::new(None, debug)?;

    match &args.command {
        CyclesCommand::List { team, r#type } => {
            let team_id = client.get_team_id(team).await?;
            let result = client.query_raw(
                r#"query($teamId: ID!) {
                    team(id: $teamId) {
                        cycles(orderBy: createdAt) {
                            nodes {
                                id number name startsAt endsAt isActive
                                issues { nodes { id } }
                            }
                        }
                    }
                }"#,
                Some(serde_json::json!({"teamId": team_id})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else if let Some(cycles) = result.pointer("/data/team/cycles/nodes").and_then(|v| v.as_array()) {
                let now = chrono::Utc::now().to_rfc3339();
                let filtered: Vec<&serde_json::Value> = cycles.iter().filter(|c| {
                    match r#type.as_str() {
                        "current" => c["isActive"].as_bool().unwrap_or(false),
                        "previous" => !c["isActive"].as_bool().unwrap_or(false) && c["endsAt"].as_str().map(|e| e < now.as_str()).unwrap_or(false),
                        "next" => !c["isActive"].as_bool().unwrap_or(false) && c["startsAt"].as_str().map(|s| s > now.as_str()).unwrap_or(false),
                        _ => true,
                    }
                }).collect();
                let headers = &["#", "Name", "Start", "End", "Status", "Issues"];
                let rows: Vec<Vec<String>> = filtered.iter().map(|c| {
                    let status = if c["isActive"].as_bool().unwrap_or(false) {
                        "Active".to_string()
                    } else if c["endsAt"].as_str().map(|e| e < now.as_str()).unwrap_or(false) {
                        "Past".to_string()
                    } else {
                        "Upcoming".to_string()
                    };
                    let issue_count = c.pointer("/issues/nodes").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
                    vec![
                        c["number"].as_i64().map(|n| n.to_string()).unwrap_or("-".into()),
                        c["name"].as_str().unwrap_or("-").to_string(),
                        c["startsAt"].as_str().unwrap_or("-").chars().take(10).collect(),
                        c["endsAt"].as_str().unwrap_or("-").chars().take(10).collect(),
                        status,
                        issue_count.to_string(),
                    ]
                }).collect();
                crate::output::table::print_table(headers, &rows);
            }
        }
        CyclesCommand::Get { cycle_id } => {
            let result = client.query_raw(
                r#"query($id: String!) {
                    cycle(id: $id) {
                        id number name startsAt endsAt isActive
                        team { key name }
                        issues(first: 100) {
                            nodes {
                                id identifier title
                                state { name }
                                assignee { displayName }
                                priority
                                team { key }
                            }
                        }
                    }
                }"#,
                Some(serde_json::json!({"id": cycle_id})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else {
                let cycle = &result["data"]["cycle"];
                crate::output::detail::print_detail("Cycle", &format!("#{}", cycle["number"]), 0);
                if let Some(name) = cycle["name"].as_str() {
                    crate::output::detail::print_detail("Name", name, 0);
                }
                crate::output::detail::print_detail("Start", cycle["startsAt"].as_str().unwrap_or("-"), 0);
                crate::output::detail::print_detail("End", cycle["endsAt"].as_str().unwrap_or("-"), 0);
                let status = if cycle["isActive"].as_bool().unwrap_or(false) { "Active" } else { "Inactive" };
                crate::output::detail::print_detail("Status", status, 0);
                if let Some(issues) = cycle.pointer("/issues/nodes").and_then(|v| v.as_array()) {
                    println!("\n  Issues ({}):", issues.len());
                    for issue in issues {
                        crate::output::detail::print_issue_summary(issue);
                    }
                }
            }
        }
        CyclesCommand::Issues { team } => {
            let team_id = client.get_team_id(team).await?;
            let result = client.query_raw(
                r#"query($teamId: String!) {
                    team(id: $teamId) {
                        activeCycle {
                            issues(first: 100) {
                                nodes {
                                    id identifier title
                                    state { name }
                                    assignee { displayName }
                                    priority
                                    team { key }
                                }
                            }
                        }
                    }
                }"#,
                Some(serde_json::json!({"teamId": team_id})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else if let Some(issues) = result.pointer("/data/team/activeCycle/issues/nodes").and_then(|v| v.as_array()) {
                for issue in issues {
                    crate::output::detail::print_issue_summary(issue);
                }
                if issues.is_empty() {
                    println!("  No issues in current cycle.");
                }
            } else {
                println!("  No active cycle for team {team}.");
            }
        }
        CyclesCommand::Create { team, name, start, end } => {
            let team_id = client.get_team_id(team).await?;
            let mut input = serde_json::json!({"teamId": team_id, "startsAt": start, "endsAt": end});
            if let Some(n) = name { input["name"] = serde_json::json!(n); }
            let result = client.query_raw(
                r#"mutation($input: CycleCreateInput!) { cycleCreate(input: $input) { success cycle { id number } } }"#,
                Some(serde_json::json!({"input": input})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else {
                println!("{}", crate::output::color::green("Cycle created."));
            }
        }
        CyclesCommand::Update { cycle_id, name, start, end } => {
            let mut input = serde_json::Map::new();
            if let Some(n) = name { input.insert("name".into(), serde_json::json!(n)); }
            if let Some(s) = start { input.insert("startsAt".into(), serde_json::json!(s)); }
            if let Some(e) = end { input.insert("endsAt".into(), serde_json::json!(e)); }
            let result = client.query_raw(
                r#"mutation($id: String!, $input: CycleUpdateInput!) { cycleUpdate(id: $id, input: $input) { success } }"#,
                Some(serde_json::json!({"id": cycle_id, "input": input})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else {
                println!("{}", crate::output::color::green("Cycle updated."));
            }
        }
        CyclesCommand::Add { issue_id, team } => {
            let team_id = client.get_team_id(team).await?;
            // Get active cycle
            let cycle_result = client.query_raw(
                r#"query($teamId: String!) { team(id: $teamId) { activeCycle { id } } }"#,
                Some(serde_json::json!({"teamId": team_id})),
            ).await?;
            let cycle_id = cycle_result.pointer("/data/team/activeCycle/id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("No active cycle for team {team}"))?;
            let result = client.query_raw(
                r#"mutation($id: String!, $input: IssueUpdateInput!) { issueUpdate(id: $id, input: $input) { success } }"#,
                Some(serde_json::json!({"id": issue_id, "input": {"cycleId": cycle_id}})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else {
                println!("{}", crate::output::color::green(&format!("Added {issue_id} to current cycle.")));
            }
        }
        CyclesCommand::Remove { issue_id } => {
            let result = client.query_raw(
                r#"mutation($id: String!, $input: IssueUpdateInput!) { issueUpdate(id: $id, input: $input) { success } }"#,
                Some(serde_json::json!({"id": issue_id, "input": {"cycleId": null}})),
            ).await?;
            if json {
                crate::output::print_json(&result);
            } else {
                println!("{}", crate::output::color::green(&format!("Removed {issue_id} from cycle.")));
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 2: Create graphql/cycles.rs stub**

Create `src/graphql/cycles.rs`:
```rust
// Cycle GraphQL types — using raw queries via client.query_raw() for now.
```

- [ ] **Step 3: Register in cli.rs, mod files, main.rs**

Add `Cycles` variant to `Commands` enum, add to `commands/mod.rs`, `graphql/mod.rs`, and `main.rs` match arm. Same pattern as Task 10 Step 3.

- [ ] **Step 4: Add Cargo.toml dependency for chrono**

Add to `[dependencies]` in `Cargo.toml`:
```toml
chrono = "0.4"
```

- [ ] **Step 5: Add smoke tests**

Append to `tests/cli_smoke.rs`:
```rust
#[test]
fn test_cycles_help() {
    lin()
        .args(["cycles", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("issues"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("remove"));
}
```

- [ ] **Step 6: Run all tests and commit**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test
git add -A && git commit -m "feat: implement cycles command group with all 7 subcommands"
```

---

### Task 12: Remaining Command Groups (Roadmap, Labels, Teams, Relations, Customers, Views, Docs, Notifications)

Each follows the exact same pattern as Tasks 10-11. For each command group:

1. Create `src/commands/<group>.rs` with clap Args + Subcommand enums + execute function
2. Create `src/graphql/<group>.rs` stub
3. Register in `cli.rs` (Commands enum), `commands/mod.rs`, `graphql/mod.rs`, `main.rs`
4. Add smoke tests
5. Commit

Below are the clap definitions for each. The `execute` function bodies follow the same pattern: build query → call `client.query_raw()` → format output.

#### 12a: Roadmap

- [ ] **Step 1: Create commands/roadmap.rs**

Create `src/commands/roadmap.rs`:
```rust
use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct RoadmapArgs {
    #[command(subcommand)]
    pub command: RoadmapCommand,
}

#[derive(Subcommand, Debug)]
pub enum RoadmapCommand {
    /// List project status updates
    Updates {
        project: String,
        #[arg(long, default_value = "10")]
        limit: i32,
    },
    /// Post a project status update
    Post {
        project: String,
        body: String,
        /// Health: onTrack, atRisk, offTrack
        #[arg(long, default_value = "onTrack")]
        health: String,
    },
    /// List project milestones
    Milestones { project: String },
    /// Create a milestone
    CreateMilestone {
        project: String,
        name: String,
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    /// Update a milestone
    UpdateMilestone {
        milestone_id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a milestone
    DeleteMilestone { milestone_id: String },
    /// List initiatives
    Initiatives {
        #[arg(long, default_value = "20")]
        limit: i32,
    },
}

pub async fn execute(args: &RoadmapArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = crate::client::LinearClient::new(None, debug)?;
    match &args.command {
        RoadmapCommand::Updates { project, limit } => {
            let pid = client.get_project_id(project).await?;
            let result = client.query_raw(
                r#"query($id: String!) { project(id: $id) { projectUpdates(first: 50) { nodes { body health createdAt user { displayName } } } } }"#,
                Some(serde_json::json!({"id": pid})),
            ).await?;
            if json { crate::output::print_json(&result); }
            else if let Some(updates) = result.pointer("/data/project/projectUpdates/nodes").and_then(|v| v.as_array()) {
                for u in updates.iter().take(*limit as usize) {
                    let health = u["health"].as_str().unwrap_or("-");
                    let user = u.pointer("/user/displayName").and_then(|v| v.as_str()).unwrap_or("?");
                    let date = u["createdAt"].as_str().unwrap_or("").chars().take(10).collect::<String>();
                    println!("  {} {} {} {}", crate::output::detail::format_health(health), crate::output::color::dim(&date), crate::output::color::dim(user), "");
                    if let Some(body) = u["body"].as_str() {
                        for line in body.lines().take(3) { println!("    {line}"); }
                    }
                    println!();
                }
            }
        }
        RoadmapCommand::Post { project, body, health } => {
            let pid = client.get_project_id(project).await?;
            let result = client.query_raw(
                r#"mutation($input: ProjectUpdateCreateInput!) { projectUpdateCreate(input: $input) { success } }"#,
                Some(serde_json::json!({"input": {"projectId": pid, "body": body, "health": health}})),
            ).await?;
            if json { crate::output::print_json(&result); }
            else { println!("{}", crate::output::color::green("Update posted.")); }
        }
        RoadmapCommand::Milestones { project } => {
            let pid = client.get_project_id(project).await?;
            let result = client.query_raw(
                r#"query($id: String!) { project(id: $id) { projectMilestones { nodes { id name targetDate description sortOrder } } } }"#,
                Some(serde_json::json!({"id": pid})),
            ).await?;
            if json { crate::output::print_json(&result); }
            else if let Some(milestones) = result.pointer("/data/project/projectMilestones/nodes").and_then(|v| v.as_array()) {
                let headers = &["Name", "Target Date", "Description"];
                let rows: Vec<Vec<String>> = milestones.iter().map(|m| vec![
                    m["name"].as_str().unwrap_or("-").to_string(),
                    m["targetDate"].as_str().unwrap_or("-").to_string(),
                    m["description"].as_str().unwrap_or("-").chars().take(50).collect(),
                ]).collect();
                crate::output::table::print_table(headers, &rows);
            }
        }
        RoadmapCommand::CreateMilestone { project, name, date, description } => {
            let pid = client.get_project_id(project).await?;
            let mut input = serde_json::json!({"projectId": pid, "name": name});
            if let Some(d) = date { input["targetDate"] = serde_json::json!(d); }
            if let Some(desc) = description { input["description"] = serde_json::json!(desc); }
            let result = client.query_raw(
                r#"mutation($input: ProjectMilestoneCreateInput!) { projectMilestoneCreate(input: $input) { success } }"#,
                Some(serde_json::json!({"input": input})),
            ).await?;
            if json { crate::output::print_json(&result); }
            else { println!("{}", crate::output::color::green(&format!("Milestone '{name}' created."))); }
        }
        RoadmapCommand::UpdateMilestone { milestone_id, name, date, description } => {
            let mut input = serde_json::Map::new();
            if let Some(n) = name { input.insert("name".into(), serde_json::json!(n)); }
            if let Some(d) = date { input.insert("targetDate".into(), serde_json::json!(d)); }
            if let Some(desc) = description { input.insert("description".into(), serde_json::json!(desc)); }
            let result = client.query_raw(
                r#"mutation($id: String!, $input: ProjectMilestoneUpdateInput!) { projectMilestoneUpdate(id: $id, input: $input) { success } }"#,
                Some(serde_json::json!({"id": milestone_id, "input": input})),
            ).await?;
            if json { crate::output::print_json(&result); }
            else { println!("{}", crate::output::color::green("Milestone updated.")); }
        }
        RoadmapCommand::DeleteMilestone { milestone_id } => {
            let result = client.query_raw(
                r#"mutation($id: String!) { projectMilestoneDelete(id: $id) { success } }"#,
                Some(serde_json::json!({"id": milestone_id})),
            ).await?;
            if json { crate::output::print_json(&result); }
            else { println!("{}", crate::output::color::green("Milestone deleted.")); }
        }
        RoadmapCommand::Initiatives { limit } => {
            let result = client.query_raw(
                r#"query($first: Int!) { initiatives(first: $first) { nodes { id name status targetDate projects { nodes { id name } } } } }"#,
                Some(serde_json::json!({"first": limit})),
            ).await?;
            if json { crate::output::print_json(&result); }
            else if let Some(initiatives) = result.pointer("/data/initiatives/nodes").and_then(|v| v.as_array()) {
                let headers = &["Name", "Status", "Target", "Projects"];
                let rows: Vec<Vec<String>> = initiatives.iter().map(|i| vec![
                    i["name"].as_str().unwrap_or("-").to_string(),
                    i["status"].as_str().unwrap_or("-").to_string(),
                    i["targetDate"].as_str().unwrap_or("-").to_string(),
                    i.pointer("/projects/nodes").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0).to_string(),
                ]).collect();
                crate::output::table::print_table(headers, &rows);
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 2: Register roadmap, add smoke tests, commit**

Same pattern. Commit message: `"feat: implement roadmap command group with 7 subcommands"`

#### 12b: Labels

- [ ] **Step 3: Create commands/labels.rs**

Create `src/commands/labels.rs` with 8 subcommands: `list`, `create`, `update`, `delete`, `apply`, `remove`, `usage`. Follow exact same pattern — clap Args, execute function with `query_raw`, output formatting.

Key implementation notes:
- `apply` must merge new labels with existing ones (fetch current labels, add new, update)
- `remove` must filter out specified labels and update with the remainder
- `usage` lists issues with a specific label using `IssueFilter`

- [ ] **Step 4: Register labels, add smoke tests, commit**

Commit message: `"feat: implement labels command group with 7 subcommands"`

#### 12c: Teams

- [ ] **Step 5: Create commands/teams.rs**

5 subcommands: `list`, `get`, `members`, `states`, `workload`.

Key implementation notes:
- `states` sorts by type (triage→backlog→unstarted→started→completed→canceled) then position
- `workload` groups issues by assignee then status, shows distribution table

- [ ] **Step 6: Register teams, add smoke tests, commit**

Commit message: `"feat: implement teams command group with 5 subcommands"`

#### 12d: Relations

- [ ] **Step 7: Create commands/relations.rs**

6 subcommands: `list`, `blocks`, `blocked-by`, `relates`, `duplicate`, `remove`.

Key implementation notes:
- `blocked-by` swaps issue order before calling create with type "blocks"
- `list` shows both `relations` and `inverseRelations` from the issue

- [ ] **Step 8: Register relations, add smoke tests, commit**

Commit message: `"feat: implement relations command group with 6 subcommands"`

#### 12e: Customers

- [ ] **Step 9: Create commands/customers.rs**

8 subcommands: `list`, `create`, `update`, `delete`, `link`, `needs`, `tiers`, `create-tier`.

Key implementation notes:
- `needs` auto-detects issue identifier (regex `^[A-Z]+-\d+$`) vs customer name
- `link` creates a `CustomerNeed` linking customer to issue
- Revenue formatted as currency with `$` prefix

- [ ] **Step 10: Register customers, add smoke tests, commit**

Commit message: `"feat: implement customers command group with 8 subcommands"`

#### 12f: Views

- [ ] **Step 11: Create commands/views.rs**

6 subcommands: `list`, `get`, `create`, `update`, `delete`, `issues`.

Key implementation notes:
- `--filter` accepts Linear filter JSON string
- `--shared`/`--personal` are mutually exclusive flags
- `issues` extracts `filterData` from the view and runs it as an issue query

- [ ] **Step 12: Register views, add smoke tests, commit**

Commit message: `"feat: implement views command group with 6 subcommands"`

#### 12g: Docs

- [ ] **Step 13: Create commands/docs.rs**

6 subcommands: `list`, `get`, `search`, `create`, `update`, `delete`.

Key implementation notes:
- `get` renders full document content (markdown via termimad)
- `search` uses `searchDocuments` query

- [ ] **Step 14: Register docs, add smoke tests, commit**

Commit message: `"feat: implement docs command group with 6 subcommands"`

#### 12h: Notifications

- [ ] **Step 15: Create commands/notifications.rs**

3 subcommands: `list`, `read`, `archive`.

Key implementation notes:
- `list --unread` filters by `readAt` being null
- `read --all` uses `notificationMarkReadAll` mutation
- `archive --all` uses `notificationArchiveAll` mutation
- Notification type mapping: issueAssignedToYou→Assigned, issueMention→Mentioned, issueComment→Comment, etc.

- [ ] **Step 16: Register notifications, add smoke tests, commit**

Commit message: `"feat: implement notifications command group with 3 subcommands"`

---

### Task 13: Interactive Features

**Files:**
- Modify: `src/commands/issues.rs` (add fuzzy select for `--team`)
- Modify: `src/commands/projects.rs`
- Modify: `src/commands/cycles.rs`
- Modify: `src/output/interactive.rs`

- [ ] **Step 1: Add fuzzy team selection when --team is omitted at a TTY**

In commands that take `--team` as optional, add this pattern at the start of the handler:

```rust
let team = match team {
    Some(t) => t.clone(),
    None if crate::output::interactive::is_interactive() => {
        let teams = client.get_teams().await?;
        let items: Vec<String> = teams.iter().map(|t| format!("{} ({})", t.name, t.key)).collect();
        let idx = crate::output::interactive::fuzzy_select("Select team", &items)?;
        teams[idx].key.clone()
    }
    None => anyhow::bail!("--team is required (or run interactively for selection)"),
};
```

- [ ] **Step 2: Add confirm prompts for destructive actions**

In `archive`, `delete` commands, add:
```rust
if crate::output::interactive::is_interactive() {
    if !crate::output::interactive::confirm(&format!("Archive {identifier}?"))? {
        println!("Cancelled.");
        return Ok(());
    }
}
```

- [ ] **Step 3: Run all tests**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test
```

- [ ] **Step 4: Commit**

```bash
git add src/commands/ src/output/interactive.rs
git commit -m "feat: add interactive team selection and destructive action confirmations"
```

---

### Task 14: Integration Tests

**Files:**
- Create: `tests/integration.rs`

- [ ] **Step 1: Write integration tests**

Create `tests/integration.rs`:
```rust
//! Integration tests — require LINEAR_API_KEY.
//! Run with: cargo test --test integration -- --ignored

use assert_cmd::Command;
use predicates::prelude::*;

fn has_api_key() -> bool {
    std::env::var("LINEAR_API_KEY").is_ok()
        || std::path::Path::new(".env.local").exists()
        || std::path::Path::new(".env").exists()
}

fn lin() -> Command {
    Command::cargo_bin("lin").unwrap()
}

#[test]
#[ignore]
fn test_teams_list() {
    if !has_api_key() { return; }
    lin()
        .args(["teams", "list"])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_issues_list() {
    if !has_api_key() { return; }
    lin()
        .args(["issues", "list", "--limit", "3"])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_issues_list_json() {
    if !has_api_key() { return; }
    lin()
        .args(["--json", "issues", "list", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("identifier"));
}

#[test]
#[ignore]
fn test_projects_list() {
    if !has_api_key() { return; }
    lin()
        .args(["projects", "list", "--limit", "3"])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_notifications_list() {
    if !has_api_key() { return; }
    lin()
        .args(["notifications", "list", "--limit", "3"])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_docs_list() {
    if !has_api_key() { return; }
    lin()
        .args(["docs", "list", "--limit", "3"])
        .assert()
        .success();
}
```

- [ ] **Step 2: Run integration tests**

```bash
cd /Users/wookiedrool/src/lin-cli && LINEAR_API_KEY="..." cargo test --test integration -- --ignored
```
Expected: all pass (assuming valid API key with workspace data).

- [ ] **Step 3: Commit**

```bash
git add tests/integration.rs
git commit -m "test: add integration tests for core read operations"
```

---

### Task 15: Final Polish

**Files:**
- Modify: `README.md` (verify accuracy)
- Modify: `CHANGELOG.md` (verify accuracy)
- Create: `LICENSE`

- [ ] **Step 1: Create LICENSE file**

Create `LICENSE`:
```
MIT License

Copyright (c) 2026

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

- [ ] **Step 2: Run full test suite**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo test
```
Expected: all smoke + unit tests pass.

- [ ] **Step 3: Build release binary and test size**

```bash
cd /Users/wookiedrool/src/lin-cli && cargo build --release && ls -lh target/release/lin
```

- [ ] **Step 4: Final commit**

```bash
git add LICENSE README.md CHANGELOG.md
git commit -m "chore: add LICENSE, finalize README and CHANGELOG"
```
