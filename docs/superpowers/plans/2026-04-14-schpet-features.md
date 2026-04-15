# schpet-inspired Features Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add query power flags, file-based content input, git workflow commands (`issues start`, `issues pr`), and multi-workspace auth to lin CLI — bringing the best ideas from schpet/linear-cli.

**Architecture:** Four independent buckets shipped incrementally. Buckets C/D add flags to existing commands. Bucket A adds `issues start` and `issues pr` using `std::process::Command` for git/gh. Bucket B restructures config.rs for named workspaces and adds `auth` command group.

**Tech Stack:** Rust, Clap 4 (derive), serde_json, tokio, anyhow. No new dependencies.

---

## Bucket C: Query Power

### Task 1: Add `--created-after` and `--updated-after` to `issues list`

**Files:**
- Modify: `src/commands/issues.rs:20-38` (List struct)
- Modify: `src/commands/issues.rs:224-270` (List handler filter building)
- Test: `tests/cli_smoke.rs`

- [ ] **Step 1: Add fields to `IssuesCommand::List`**

In `src/commands/issues.rs`, add these fields after the `label` field (line 35) inside the `List` variant:

```rust
        /// Only issues created on or after this date (YYYY-MM-DD)
        #[arg(long)]
        created_after: Option<String>,
        /// Only issues updated on or after this date (YYYY-MM-DD)
        #[arg(long)]
        updated_after: Option<String>,
```

- [ ] **Step 2: Update the destructure in the List handler**

In the `IssuesCommand::List` match arm (around line 200), update the destructure to include the new fields:

```rust
        IssuesCommand::List {
            team,
            state,
            assignee,
            priority,
            label,
            limit,
            created_after,
            updated_after,
        } => {
```

- [ ] **Step 3: Add filter logic after the label block**

After the `if !label.is_empty() { ... }` block (around line 265), add:

```rust
            if let Some(date) = created_after {
                filter["createdAt"] = json!({ "gte": date });
            }
            if let Some(date) = updated_after {
                filter["updatedAt"] = json!({ "gte": date });
            }
```

- [ ] **Step 4: Add smoke tests**

In `tests/cli_smoke.rs`, add:

```rust
#[test]
fn test_issues_list_date_filters() {
    lin()
        .args(["issues", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--created-after"))
        .stdout(predicate::str::contains("--updated-after"));
}

#[test]
fn test_issues_list_date_filters_parse() {
    let output = lin()
        .args([
            "issues", "list",
            "--team", "ENG",
            "--created-after", "2026-01-01",
            "--updated-after", "2026-03-01",
            "--limit", "1",
        ])
        .output()
        .expect("failed to run command");
    let code = output.status.code().unwrap_or(0);
    assert_ne!(code, 2, "clap argument parsing failed");
}
```

- [ ] **Step 5: Build and test**

```bash
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

Expected: clean build, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/commands/issues.rs tests/cli_smoke.rs
git commit -m "feat(issues): add --created-after and --updated-after date filters to issues list"
```

---

### Task 2: Add `--all-teams` flag to `issues list`

**Files:**
- Modify: `src/commands/issues.rs:20-40` (List struct)
- Modify: `src/commands/issues.rs:200-243` (List handler team resolution)
- Test: `tests/cli_smoke.rs`

- [ ] **Step 1: Add `--all-teams` field to `IssuesCommand::List`**

In `src/commands/issues.rs`, add this field after the `team` field (line 23):

```rust
        /// Query across all teams (conflicts with --team)
        #[arg(long, conflicts_with = "team")]
        all_teams: bool,
```

- [ ] **Step 2: Update the destructure**

Update the destructure in the `IssuesCommand::List` match arm to include `all_teams`:

```rust
        IssuesCommand::List {
            team,
            all_teams,
            state,
            assignee,
            priority,
            label,
            limit,
            created_after,
            updated_after,
        } => {
```

- [ ] **Step 3: Skip team prompt when `--all-teams` is set**

Replace the `team_key` match block (lines 226-238) with:

```rust
            let team_key = if *all_teams {
                None
            } else {
                match team {
                    Some(t) => Some(t.clone()),
                    None if crate::output::interactive::is_interactive() => {
                        let teams = client.get_teams().await?;
                        let items: Vec<String> = teams
                            .iter()
                            .map(|t| format!("{} ({})", t.name, t.key))
                            .collect();
                        let idx =
                            crate::output::interactive::fuzzy_select("Select team", &items)?;
                        Some(teams[idx].key.clone())
                    }
                    None => None,
                }
            };
```

- [ ] **Step 4: Add smoke tests**

In `tests/cli_smoke.rs`, add:

```rust
#[test]
fn test_issues_list_all_teams_flag() {
    lin()
        .args(["issues", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--all-teams"));
}

#[test]
fn test_issues_list_all_teams_conflicts_with_team() {
    lin()
        .args(["issues", "list", "--team", "ENG", "--all-teams"])
        .assert()
        .failure();
}
```

- [ ] **Step 5: Build and test**

```bash
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

- [ ] **Step 6: Commit**

```bash
git add src/commands/issues.rs tests/cli_smoke.rs
git commit -m "feat(issues): add --all-teams flag for cross-team queries"
```

---

## Bucket D: Content Input from Files

### Task 3: Add `--description-file` to `issues create`

**Files:**
- Modify: `src/commands/issues.rs:48-83` (Create struct)
- Modify: `src/commands/issues.rs:332-427` (Create handler)
- Test: `tests/cli_smoke.rs`

- [ ] **Step 1: Add `--description-file` field**

In `src/commands/issues.rs`, add after the `description` field (line 58):

```rust
        /// Read description from a file (conflicts with --description)
        #[arg(long, conflicts_with = "description")]
        description_file: Option<String>,
```

- [ ] **Step 2: Update the Create destructure and handler**

In the `IssuesCommand::Create` match arm, add `description_file` to the destructure:

```rust
        IssuesCommand::Create {
            team,
            title,
            description,
            description_file,
            assignee,
            priority,
            estimate,
            due_date,
            label,
            parent,
            project,
            status,
        } => {
```

Then replace the description handling block (around line 365-367):

```rust
            let desc = match (description, description_file) {
                (Some(d), _) => Some(d.clone()),
                (_, Some(path)) => Some(std::fs::read_to_string(path).map_err(|e| {
                    anyhow::anyhow!("Failed to read description file '{}': {}", path, e)
                })?),
                _ => None,
            };
            if let Some(desc) = desc {
                input["description"] = json!(desc);
            }
```

- [ ] **Step 3: Add smoke test**

```rust
#[test]
fn test_issues_create_description_file_flag() {
    lin()
        .args(["issues", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--description-file"));
}

#[test]
fn test_issues_create_description_conflict() {
    lin()
        .args([
            "issues", "create",
            "--team", "ENG",
            "--title", "Test",
            "--description", "inline",
            "--description-file", "file.md",
        ])
        .assert()
        .failure();
}
```

- [ ] **Step 4: Build and test**

```bash
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

- [ ] **Step 5: Commit**

```bash
git add src/commands/issues.rs tests/cli_smoke.rs
git commit -m "feat(issues): add --description-file to issues create"
```

---

### Task 4: Add `--description-file` to `issues update` and `--body-file` to `issues comment`

**Files:**
- Modify: `src/commands/issues.rs:85-118` (Update struct)
- Modify: `src/commands/issues.rs:120-130` (Comment struct)
- Modify: `src/commands/issues.rs:429-541` (Update handler)
- Modify: `src/commands/issues.rs:543-580` (Comment handler)
- Test: `tests/cli_smoke.rs`

- [ ] **Step 1: Add `--description-file` to Update struct**

In the `Update` variant, add after the `team` field:

```rust
        /// Read description from a file
        #[arg(long)]
        description_file: Option<String>,
```

- [ ] **Step 2: Add `--body-file` to Comment struct**

Replace the `Comment` variant:

```rust
    /// Add a comment to an issue
    Comment {
        /// Issue identifier
        identifier: String,
        /// Comment body (omit if using --body-file)
        body: Option<String>,
        /// Read comment body from a file
        #[arg(long)]
        body_file: Option<String>,
    },
```

- [ ] **Step 3: Update the Update handler destructure**

Add `description_file` to the destructure:

```rust
        IssuesCommand::Update {
            identifier,
            status,
            assignee,
            priority,
            estimate,
            due_date,
            parent,
            project,
            label,
            milestone: _,
            team,
            description_file,
        } => {
```

After the `if let Some(team_name) = team { ... }` block, add:

```rust
            if let Some(path) = description_file {
                let content = std::fs::read_to_string(path)
                    .map_err(|e| anyhow::anyhow!("Failed to read description file '{}': {}", path, e))?;
                input["description"] = json!(content);
            }
```

- [ ] **Step 4: Update the Comment handler**

Replace the Comment handler body:

```rust
        IssuesCommand::Comment {
            identifier,
            body,
            body_file,
        } => {
            let comment_body = match (body, body_file) {
                (Some(b), _) => b.clone(),
                (_, Some(path)) => std::fs::read_to_string(path)
                    .map_err(|e| anyhow::anyhow!("Failed to read body file '{}': {}", path, e))?,
                (None, None) => anyhow::bail!("Provide a comment body or --body-file"),
            };

            let query = r#"
                mutation($input: CommentCreateInput!) {
                    commentCreate(input: $input) {
                        success
                        comment { id body }
                    }
                }
            "#;
            let variables = json!({
                "input": {
                    "issueId": identifier,
                    "body": comment_body,
                }
            });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/commentCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Added comment to {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(identifier),
                    );
                } else {
                    println!(
                        "  {} Failed to add comment",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }
```

- [ ] **Step 5: Add smoke tests**

```rust
#[test]
fn test_issues_update_description_file_flag() {
    lin()
        .args(["issues", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--description-file"));
}

#[test]
fn test_issues_comment_body_file_flag() {
    lin()
        .args(["issues", "comment", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--body-file"));
}
```

- [ ] **Step 6: Build and test**

```bash
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

- [ ] **Step 7: Commit**

```bash
git add src/commands/issues.rs tests/cli_smoke.rs
git commit -m "feat(issues): add --description-file to update, --body-file to comment"
```

---

## Bucket A: Developer Workflow

### Task 5: `issues start` — create git branch from issue

**Files:**
- Modify: `src/commands/issues.rs:12-155` (IssuesCommand enum)
- Modify: `src/commands/issues.rs` (end of execute fn, before closing brace)
- Test: `tests/cli_smoke.rs`

- [ ] **Step 1: Add `Start` variant to `IssuesCommand`**

Add after the `Unsubscribe` variant (before the closing `}`):

```rust
    /// Start working on an issue: create/switch to its git branch
    Start {
        /// Issue identifier (e.g., ENG-123)
        identifier: String,
        /// Update issue status after branching
        #[arg(long)]
        status: Option<String>,
        /// Just print the branch name, don't run git commands
        #[arg(long)]
        print_only: bool,
    },
```

- [ ] **Step 2: Implement the Start handler**

Add this match arm before the closing `}` of the execute function (before the `Ok(())`):

```rust
        IssuesCommand::Start {
            identifier,
            status,
            print_only,
        } => {
            // Fetch the issue's suggested branch name
            let query = r#"
                query($id: String!) {
                    issue(id: $id) {
                        id identifier branchName
                        team { key }
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;
            let issue = result
                .pointer("/data/issue")
                .ok_or_else(|| anyhow::anyhow!("Issue not found: {identifier}"))?;
            let branch_name = issue
                .get("branchName")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("No branch name for issue {identifier}"))?;

            if *print_only {
                if json {
                    crate::output::print_json(&serde_json::json!({
                        "branch": branch_name,
                        "identifier": identifier,
                    }));
                } else {
                    println!("{branch_name}");
                }
                return Ok(());
            }

            // Check if we're in a git repo
            let git_check = std::process::Command::new("git")
                .args(["rev-parse", "--is-inside-work-tree"])
                .output();
            match git_check {
                Ok(output) if output.status.success() => {}
                _ => anyhow::bail!("Not a git repository"),
            }

            // Try to create and switch to the branch; if it exists, just switch
            let checkout = std::process::Command::new("git")
                .args(["checkout", "-b", branch_name])
                .output()?;
            if !checkout.status.success() {
                // Branch might already exist — try plain checkout
                let switch = std::process::Command::new("git")
                    .args(["checkout", branch_name])
                    .output()?;
                if !switch.status.success() {
                    let stderr = String::from_utf8_lossy(&switch.stderr);
                    anyhow::bail!("Failed to checkout branch '{}': {}", branch_name, stderr.trim());
                }
            }

            // Optionally update status
            if let Some(status_name) = status {
                let team_key = issue
                    .get("team")
                    .and_then(|t| t.get("key"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Could not determine team for {identifier}"))?;
                let state_id = client.get_state_id(team_key, status_name).await?;
                let update_query = r#"
                    mutation($id: String!, $input: IssueUpdateInput!) {
                        issueUpdate(id: $id, input: $input) { success }
                    }
                "#;
                client
                    .query_raw(
                        update_query,
                        Some(json!({ "id": identifier, "input": { "stateId": state_id } })),
                    )
                    .await?;
            }

            if json {
                crate::output::print_json(&serde_json::json!({
                    "branch": branch_name,
                    "identifier": identifier,
                }));
            } else {
                println!(
                    "  {} Switched to branch {}",
                    crate::output::color::green("OK"),
                    crate::output::color::bold(branch_name),
                );
            }
        }
```

- [ ] **Step 3: Add smoke tests**

```rust
#[test]
fn test_issues_start_help() {
    lin()
        .args(["issues", "start", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("branch"))
        .stdout(predicate::str::contains("--status"))
        .stdout(predicate::str::contains("--print-only"));
}

#[test]
fn test_issues_start_missing_identifier() {
    lin().args(["issues", "start"]).assert().failure();
}
```

- [ ] **Step 4: Build and test**

```bash
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

- [ ] **Step 5: Commit**

```bash
git add src/commands/issues.rs tests/cli_smoke.rs
git commit -m "feat(issues): add issues start command for git branch creation"
```

---

### Task 6: `issues pr` — create GitHub PR from issue

**Files:**
- Modify: `src/commands/issues.rs` (IssuesCommand enum, execute fn)
- Test: `tests/cli_smoke.rs`

- [ ] **Step 1: Add `Pr` variant to `IssuesCommand`**

Add after the `Start` variant:

```rust
    /// Create a GitHub PR linked to an issue (requires gh CLI)
    Pr {
        /// Issue identifier (e.g., ENG-123)
        identifier: String,
        /// Create as draft PR
        #[arg(long)]
        draft: bool,
        /// Base branch for the PR
        #[arg(long)]
        base: Option<String>,
    },
```

- [ ] **Step 2: Implement the Pr handler**

Add this match arm after the `Start` handler:

```rust
        IssuesCommand::Pr {
            identifier,
            draft,
            base,
        } => {
            // Check gh is available
            let gh_check = std::process::Command::new("gh")
                .args(["--version"])
                .output();
            match gh_check {
                Ok(output) if output.status.success() => {}
                _ => anyhow::bail!(
                    "gh CLI required for pr command (https://cli.github.com)"
                ),
            }

            // Fetch issue details
            let query = r#"
                query($id: String!) {
                    issue(id: $id) {
                        identifier title url
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;
            let issue = result
                .pointer("/data/issue")
                .ok_or_else(|| anyhow::anyhow!("Issue not found: {identifier}"))?;
            let ident = issue
                .get("identifier")
                .and_then(|v| v.as_str())
                .unwrap_or(identifier);
            let title = issue
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");
            let url = issue
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let pr_title = format!("{}: {}", ident, title);
            let pr_body = format!("Resolves {}", url);

            let mut args = vec![
                "pr".to_string(),
                "create".to_string(),
                "--title".to_string(),
                pr_title.clone(),
                "--body".to_string(),
                pr_body,
            ];
            if *draft {
                args.push("--draft".to_string());
            }
            if let Some(base_branch) = base {
                args.push("--base".to_string());
                args.push(base_branch.clone());
            }

            let gh_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let output = std::process::Command::new("gh")
                .args(&gh_args)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output()?;

            if !output.status.success() {
                anyhow::bail!("gh pr create failed");
            }

            if json {
                crate::output::print_json(&serde_json::json!({
                    "pr_title": pr_title,
                    "identifier": ident,
                }));
            }
        }
```

- [ ] **Step 3: Add smoke tests**

```rust
#[test]
fn test_issues_pr_help() {
    lin()
        .args(["issues", "pr", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("GitHub"))
        .stdout(predicate::str::contains("--draft"))
        .stdout(predicate::str::contains("--base"));
}

#[test]
fn test_issues_pr_missing_identifier() {
    lin().args(["issues", "pr"]).assert().failure();
}
```

- [ ] **Step 4: Build and test**

```bash
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

- [ ] **Step 5: Commit**

```bash
git add src/commands/issues.rs tests/cli_smoke.rs
git commit -m "feat(issues): add issues pr command for GitHub PR creation"
```

---

## Bucket B: Multi-Workspace

### Task 7: Restructure config for named workspaces

**Files:**
- Modify: `src/config.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Add workspace config structs**

Replace the entire `src/config.rs` content:

```rust
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Default workspace name
    pub default_workspace: Option<String>,

    /// Named workspaces
    #[serde(default)]
    pub workspaces: BTreeMap<String, WorkspaceConfig>,

    /// Legacy auth section (migrated on load)
    #[serde(default, skip_serializing)]
    pub auth: Option<LegacyAuth>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceConfig {
    pub api_key: String,
}

/// Old format — only used for migration
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LegacyAuth {
    pub api_key: Option<String>,
}

/// Returns the config directory: `~/.config/lin`
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("lin"))
}

/// Returns the config file path: `~/.config/lin/config.toml`
pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.toml"))
}

/// Load config, auto-migrating legacy `[auth]` format to workspaces.
pub fn load() -> anyhow::Result<Config> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(Config::default()),
    };

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&path)?;
    let mut config: Config = toml::from_str(&contents)?;

    // Migrate legacy [auth] section
    if let Some(legacy) = config.auth.take() {
        if let Some(key) = legacy.api_key.filter(|k| !k.is_empty()) {
            if config.workspaces.is_empty() {
                config.workspaces.insert(
                    "default".to_string(),
                    WorkspaceConfig { api_key: key },
                );
                config.default_workspace = Some("default".to_string());
                // Save migrated config
                let _ = save(&config);
            }
        }
    }

    Ok(config)
}

/// Save the config to disk.
pub fn save(config: &Config) -> anyhow::Result<()> {
    let dir =
        config_dir().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let path = dir.join("config.toml");

    fs::create_dir_all(&dir)?;
    let contents = toml::to_string_pretty(config)?;
    fs::write(&path, contents)?;
    Ok(())
}

/// Get the API key for a specific workspace or the default.
pub fn get_workspace_key(workspace: Option<&str>) -> Option<String> {
    let config = load().ok()?;
    let ws_name = workspace
        .map(|s| s.to_string())
        .or(config.default_workspace)?;
    config
        .workspaces
        .get(&ws_name)
        .map(|ws| ws.api_key.clone())
}

/// Read the API key from the config file (default workspace). Backwards compat.
pub fn get_api_key() -> Option<String> {
    get_workspace_key(None)
}

/// Save an API key as a named workspace.
pub fn set_workspace_key(name: &str, key: &str) -> anyhow::Result<()> {
    let mut config = load().unwrap_or_default();
    config.workspaces.insert(
        name.to_string(),
        WorkspaceConfig {
            api_key: key.to_string(),
        },
    );
    if config.default_workspace.is_none() {
        config.default_workspace = Some(name.to_string());
    }
    save(&config)
}

/// Set the default workspace.
pub fn set_default_workspace(name: &str) -> anyhow::Result<()> {
    let mut config = load().unwrap_or_default();
    if !config.workspaces.contains_key(name) {
        anyhow::bail!("Workspace '{}' not found", name);
    }
    config.default_workspace = Some(name.to_string());
    save(&config)
}

/// List all configured workspaces.
pub fn list_workspaces() -> anyhow::Result<(Vec<String>, Option<String>)> {
    let config = load().unwrap_or_default();
    let names: Vec<String> = config.workspaces.keys().cloned().collect();
    Ok((names, config.default_workspace))
}

/// Save an API key to the config file as "default" workspace. Backwards compat.
pub fn set_api_key(key: &str) -> anyhow::Result<()> {
    set_workspace_key("default", key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_no_key() {
        let config = Config::default();
        assert!(config.workspaces.is_empty());
        assert!(config.default_workspace.is_none());
    }

    #[test]
    fn test_roundtrip_toml() {
        let mut config = Config::default();
        config.default_workspace = Some("myco".to_string());
        config.workspaces.insert(
            "myco".to_string(),
            WorkspaceConfig {
                api_key: "lin_api_test123".to_string(),
            },
        );
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized.default_workspace.as_deref(),
            Some("myco")
        );
        assert_eq!(
            deserialized.workspaces.get("myco").unwrap().api_key,
            "lin_api_test123"
        );
    }

    #[test]
    fn test_legacy_migration() {
        let legacy_toml = r#"
[auth]
api_key = "lin_api_legacy"
"#;
        let mut config: Config = toml::from_str(legacy_toml).unwrap();
        // Simulate migration
        if let Some(legacy) = config.auth.take() {
            if let Some(key) = legacy.api_key {
                config.workspaces.insert(
                    "default".to_string(),
                    WorkspaceConfig { api_key: key },
                );
                config.default_workspace = Some("default".to_string());
            }
        }
        assert_eq!(
            config.workspaces.get("default").unwrap().api_key,
            "lin_api_legacy"
        );
    }

    #[test]
    fn test_config_path_exists() {
        let path = config_path();
        assert!(path.is_some());
        let p = path.unwrap();
        assert!(p.to_string_lossy().contains("lin"));
        assert!(p.to_string_lossy().ends_with("config.toml"));
    }
}
```

- [ ] **Step 2: Build and test**

```bash
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

- [ ] **Step 3: Commit**

```bash
git add src/config.rs
git commit -m "refactor(config): restructure for named workspaces with legacy migration"
```

---

### Task 8: Update `resolve_api_key` to accept workspace parameter

**Files:**
- Modify: `src/client/mod.rs:22-26` (LinearClient::new)
- Modify: `src/client/mod.rs:42-72` (resolve_api_key)

- [ ] **Step 1: Update `resolve_api_key` to accept workspace name**

In `src/client/mod.rs`, change the `resolve_api_key` signature and body:

```rust
    fn resolve_api_key(workspace: Option<&str>) -> Result<String, LinearError> {
        // 1. Environment variable (always wins)
        if let Ok(key) = env::var("LINEAR_API_KEY")
            && !key.is_empty()
        {
            return Ok(key);
        }

        // 2. Config file (workspace-aware)
        if let Some(key) = crate::config::get_workspace_key(workspace) {
            return Ok(key);
        }

        // 3. .env and .env.local files
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

        Err(LinearError::Other(anyhow::anyhow!(
            "LINEAR_API_KEY not found. Set it via env var, lin config set-token, or lin auth login"
        )))
    }
```

- [ ] **Step 2: Update `LinearClient::new` to accept workspace**

Change the signature and body:

```rust
    pub fn new(
        api_key: Option<String>,
        debug: bool,
        workspace: Option<&str>,
    ) -> Result<Self, LinearError> {
        let api_key = match api_key {
            Some(key) => key,
            None => Self::resolve_api_key(workspace)?,
        };
```

- [ ] **Step 3: Update all `LinearClient::new` call sites**

Every call to `LinearClient::new(None, debug)?` needs a third argument. Find all of them:

In `src/commands/issues.rs:158`:
```rust
    let client = LinearClient::new(None, debug, None)?;
```

In `src/commands/api.rs:22`:
```rust
    let client = LinearClient::new(None, debug, None)?;
```

Apply the same change (`None, debug` -> `None, debug, None`) in every command file:
- `src/commands/projects.rs`
- `src/commands/cycles.rs`
- `src/commands/teams.rs`
- `src/commands/labels.rs`
- `src/commands/relations.rs`
- `src/commands/customers.rs`
- `src/commands/views.rs`
- `src/commands/docs.rs`
- `src/commands/notifications.rs`
- `src/commands/me.rs`
- `src/commands/attachments.rs`
- `src/commands/search.rs`
- `src/commands/initiatives.rs`
- `src/commands/roadmap.rs`

Use a find-and-replace: `LinearClient::new(None, debug)` -> `LinearClient::new(None, debug, None)`

- [ ] **Step 4: Update test call sites**

In `src/client/mod.rs` tests, update:

```rust
    #[test]
    fn test_new_with_explicit_key() {
        let client = LinearClient::new(Some("lin_api_explicit".into()), false, None);
        assert!(client.is_ok());
    }
```

- [ ] **Step 5: Build and test**

```bash
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(client): make resolve_api_key workspace-aware"
```

---

### Task 9: Add `--workspace` global flag and `auth` command group

**Files:**
- Modify: `src/cli.rs` — add `--workspace` and `Auth` variant
- Create: `src/commands/auth.rs` — login/list/default/whoami handlers
- Modify: `src/commands/mod.rs` — register auth
- Modify: `src/main.rs` — wire up auth, pass workspace to commands
- Test: `tests/cli_smoke.rs`

- [ ] **Step 1: Add `--workspace` to `Cli` and `Auth` to `Commands`**

In `src/cli.rs`:

```rust
use clap::{Parser, Subcommand};
use clap_complete::Shell;

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

    /// Print GraphQL queries and full API responses to stderr (may contain workspace data)
    #[arg(long, global = true)]
    pub debug: bool,

    /// Use a specific workspace (overrides default)
    #[arg(long, global = true)]
    pub workspace: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a raw GraphQL query or mutation
    Api(crate::commands::api::ApiArgs),
    /// Manage authentication and workspaces
    Auth(crate::commands::auth::AuthArgs),
    /// Manage issues
    Issues(crate::commands::issues::IssuesArgs),
    /// Manage projects
    Projects(crate::commands::projects::ProjectsArgs),
    /// Manage cycles
    Cycles(crate::commands::cycles::CyclesArgs),
    /// Manage initiatives
    Initiatives(crate::commands::initiatives::InitiativesArgs),
    /// Roadmap: project updates and milestones
    Roadmap(crate::commands::roadmap::RoadmapArgs),
    /// Manage labels
    Labels(crate::commands::labels::LabelsArgs),
    /// Manage teams
    Teams(crate::commands::teams::TeamsArgs),
    /// Manage issue relations
    Relations(crate::commands::relations::RelationsArgs),
    /// Manage customers
    Customers(crate::commands::customers::CustomersArgs),
    /// Manage custom views
    Views(crate::commands::views::ViewsArgs),
    /// Manage documents
    Docs(crate::commands::docs::DocsArgs),
    /// Manage notifications
    Notifications(crate::commands::notifications::NotificationsArgs),
    /// Show authenticated user info
    Me(crate::commands::me::MeArgs),
    /// Manage issue attachments and links
    Attachments(crate::commands::attachments::AttachmentsArgs),
    /// Search across issues, projects, and documents
    Search(crate::commands::search::SearchArgs),
    /// Manage CLI configuration
    Config(crate::commands::config::ConfigArgs),
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}
```

- [ ] **Step 2: Create `src/commands/auth.rs`**

```rust
use clap::{Args, Subcommand};
use dialoguer::{Input, Password, theme::ColorfulTheme};

#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Add a workspace (prompts for name and API key)
    Login,
    /// List configured workspaces
    List,
    /// Set the default workspace
    Default {
        /// Workspace name
        name: String,
    },
    /// Show current workspace and authenticated user
    Whoami,
}

pub async fn execute(
    args: &AuthArgs,
    json: bool,
    debug: bool,
    workspace: Option<&str>,
) -> anyhow::Result<()> {
    match &args.command {
        AuthCommand::Login => {
            let name: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Workspace name")
                .interact_text()?;
            let key = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Linear API key")
                .interact()?;

            if name.is_empty() || key.is_empty() {
                anyhow::bail!("Workspace name and API key cannot be empty");
            }

            crate::config::set_workspace_key(&name, &key)?;

            if json {
                crate::output::print_json(&serde_json::json!({
                    "workspace": name,
                    "saved": true,
                }));
            } else {
                println!(
                    "  {} Workspace '{}' saved",
                    crate::output::color::green("OK"),
                    crate::output::color::bold(&name),
                );
            }
        }

        AuthCommand::List => {
            let (workspaces, default) = crate::config::list_workspaces()?;

            if json {
                crate::output::print_json(&serde_json::json!({
                    "workspaces": workspaces,
                    "default": default,
                }));
            } else if workspaces.is_empty() {
                println!("  No workspaces configured. Run `lin auth login` to add one.");
            } else {
                for ws in &workspaces {
                    let marker = if default.as_deref() == Some(ws) {
                        " *"
                    } else {
                        ""
                    };
                    println!("  {}{}", ws, marker);
                }
            }
        }

        AuthCommand::Default { name } => {
            crate::config::set_default_workspace(name)?;
            if json {
                crate::output::print_json(&serde_json::json!({
                    "default": name,
                }));
            } else {
                println!(
                    "  {} Default workspace set to '{}'",
                    crate::output::color::green("OK"),
                    crate::output::color::bold(name),
                );
            }
        }

        AuthCommand::Whoami => {
            let client =
                crate::client::LinearClient::new(None, debug, workspace)?;
            let result = client
                .query_raw("query { viewer { id displayName email } }", None)
                .await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let viewer = result.pointer("/data/viewer");
                let name = viewer
                    .and_then(|v| v.get("displayName"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let email = viewer
                    .and_then(|v| v.get("email"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let ws = workspace
                    .map(|s| s.to_string())
                    .or_else(|| {
                        crate::config::load()
                            .ok()
                            .and_then(|c| c.default_workspace)
                    })
                    .unwrap_or_else(|| "env".to_string());
                println!("  {} ({}) — workspace: {}", name, email, ws);
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Register auth module**

In `src/commands/mod.rs`, add between `api` and `attachments`:

```rust
pub mod auth;
```

- [ ] **Step 4: Update `src/main.rs`**

Replace the entire main.rs:

```rust
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let ws = cli.workspace.as_deref();

    let result = match &cli.command {
        Commands::Api(args) => commands::api::execute(args, cli.json, cli.debug).await,
        Commands::Auth(args) => commands::auth::execute(args, cli.json, cli.debug, ws).await,
        Commands::Issues(args) => commands::issues::execute(args, cli.json, cli.debug).await,
        Commands::Projects(args) => commands::projects::execute(args, cli.json, cli.debug).await,
        Commands::Cycles(args) => commands::cycles::execute(args, cli.json, cli.debug).await,
        Commands::Initiatives(args) => {
            commands::initiatives::execute(args, cli.json, cli.debug).await
        }
        Commands::Roadmap(args) => commands::roadmap::execute(args, cli.json, cli.debug).await,
        Commands::Labels(args) => commands::labels::execute(args, cli.json, cli.debug).await,
        Commands::Teams(args) => commands::teams::execute(args, cli.json, cli.debug).await,
        Commands::Relations(args) => commands::relations::execute(args, cli.json, cli.debug).await,
        Commands::Customers(args) => commands::customers::execute(args, cli.json, cli.debug).await,
        Commands::Views(args) => commands::views::execute(args, cli.json, cli.debug).await,
        Commands::Docs(args) => commands::docs::execute(args, cli.json, cli.debug).await,
        Commands::Notifications(args) => {
            commands::notifications::execute(args, cli.json, cli.debug).await
        }
        Commands::Me(args) => commands::me::execute(args, cli.json, cli.debug).await,
        Commands::Attachments(args) => {
            commands::attachments::execute(args, cli.json, cli.debug).await
        }
        Commands::Search(args) => commands::search::execute(args, cli.json, cli.debug).await,
        Commands::Config(args) => commands::config::execute(args, cli.json, cli.debug).await,
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(*shell, &mut cmd, "lin", &mut std::io::stdout());
            return;
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

- [ ] **Step 5: Add smoke tests**

```rust
// --- Auth ---

#[test]
fn test_auth_help() {
    lin()
        .args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("login"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("default"))
        .stdout(predicate::str::contains("whoami"));
}

#[test]
fn test_auth_default_missing_name() {
    lin().args(["auth", "default"]).assert().failure();
}

#[test]
fn test_workspace_flag() {
    lin()
        .args(["--workspace", "myco", "me", "--help"])
        .assert()
        .success();
}

#[test]
fn test_help_lists_auth() {
    lin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth"));
}
```

- [ ] **Step 6: Build and test**

```bash
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add auth command group and --workspace global flag for multi-workspace support"
```

---

## Docs & Release

### Task 10: Update skill, README, CHANGELOG, version bump

**Files:**
- Modify: `.claude/skills/lin/SKILL.md`
- Modify: `.claude/skills/lin/references/commands.md`
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `Cargo.toml`

- [ ] **Step 1: Update SKILL.md quick reference table**

Add these rows:

```markdown
| List issues across all teams | `lin issues list --all-teams --json` |
| Issues updated recently | `lin issues list --updated-after 2026-04-01 --json` |
| Start working on an issue | `lin issues start ENG-123 --status "In Progress"` |
| Create PR from issue | `lin issues pr ENG-123 --draft` |
| Create from markdown file | `lin issues create --team ENG --title "..." --description-file spec.md --json` |
| Add workspace | `lin auth login` |
| Switch workspace | `lin auth default <name>` |
| Check auth | `lin auth whoami --json` |
```

- [ ] **Step 2: Update `references/commands.md`**

In the Issues section, add to `issues list`:
```
  --all-teams            Query across all teams (conflicts with --team)
  --created-after <DATE> Only issues created on or after this date (YYYY-MM-DD)
  --updated-after <DATE> Only issues updated on or after this date (YYYY-MM-DD)
```

Add to `issues create`:
```
  --description-file <PATH>  Read description from a file (conflicts with --description)
```

Add to `issues update`:
```
  --description-file <PATH>  Read description from a file
```

Update `issues comment`:
```
lin issues comment <IDENTIFIER> [BODY]
  --body-file <PATH>     Read body from a file
  --json
```

Add new commands at end of Issues section:
```
lin issues start <IDENTIFIER>
  --status <STATUS>      Update issue status after branching
  --print-only           Just print the branch name
  --json

lin issues pr <IDENTIFIER>
  --draft                Create as draft PR
  --base <BRANCH>        Base branch
```

Add new section before Other:
```markdown
## Auth (Workspaces)

lin auth login             # interactive: add workspace name + API key
lin auth list              # list configured workspaces (* = default)
lin auth default <NAME>    # set default workspace
lin auth whoami            # show current user and workspace
```

- [ ] **Step 3: Update README.md**

Add `auth` row to the Commands table:
```
| `auth` | Manage workspaces and authentication |
```

Add to Usage examples:
```bash
lin auth login                                          # Add a workspace
lin issues start ENG-123                                # Create branch from issue
lin issues pr ENG-123 --draft                           # Create draft PR from issue
```

- [ ] **Step 4: Bump version and update CHANGELOG**

In `Cargo.toml`:
```toml
version = "2026.4.16"
```

In `CHANGELOG.md`, add at top:
```markdown
## [2026.4.16] — 2026-04-14

### Added
- `lin issues list --all-teams` — query issues across all teams
- `lin issues list --created-after <DATE>` / `--updated-after <DATE>` — date filters
- `lin issues create --description-file <PATH>` — read description from a file
- `lin issues update --description-file <PATH>` — update description from a file
- `lin issues comment --body-file <PATH>` — read comment body from a file
- `lin issues start <ID>` — create/switch to the git branch Linear suggests for an issue
- `lin issues pr <ID>` — create a GitHub PR linked to an issue (requires `gh`)
- `lin auth login/list/default/whoami` — multi-workspace authentication
- `--workspace <NAME>` global flag — override the default workspace for a single command

### Changed
- Config format now supports named workspaces; legacy `[auth]` format auto-migrates
- `issues comment` body is now optional when using `--body-file`
```

- [ ] **Step 5: Final verification**

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo build 2>&1 | grep "^error"
cargo test 2>&1 | grep -E "^test result:|FAILED"
```

Expected: all clean, all tests pass.

- [ ] **Step 6: Commit**

```bash
git add .claude/skills/lin/SKILL.md .claude/skills/lin/references/commands.md README.md CHANGELOG.md Cargo.toml Cargo.lock
git commit -m "chore: bump to v2026.4.16, update skill, README, changelog for schpet features"
```
