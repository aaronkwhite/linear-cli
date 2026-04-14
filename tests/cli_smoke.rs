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
        .stdout(predicate::str::contains("lin"));
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
    lin().args(["issues", "get", "--help"]).assert().success();
}

#[test]
fn test_issues_list_help() {
    lin()
        .args(["issues", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--team"))
        .stdout(predicate::str::contains("--state"))
        .stdout(predicate::str::contains("--assignee"))
        .stdout(predicate::str::contains("--priority"))
        .stdout(predicate::str::contains("--label"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_issues_list_state_flag_accepted() {
    // --state flag should be accepted (parsing only, no API call)
    lin()
        .args(["issues", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--state"));
}

#[test]
fn test_issues_list_status_alias_accepted() {
    // --status should still work as an alias for --state
    lin()
        .args(["issues", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--status"));
}

#[test]
fn test_issues_list_label_repeatable() {
    // --label can be specified multiple times (help should show it)
    lin()
        .args(["issues", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--label"))
        .stdout(predicate::str::contains("multiple times"));
}

#[test]
fn test_issues_list_all_filter_flags_parse() {
    // Verify all filter flags parse correctly together (will fail at API call, not arg parse)
    let output = lin()
        .args([
            "issues",
            "list",
            "--team",
            "ENG",
            "--state",
            "In Progress",
            "--assignee",
            "Alice",
            "--priority",
            "2",
            "--label",
            "bug",
            "--label",
            "feature",
            "--limit",
            "10",
        ])
        .output()
        .expect("failed to run command");
    // The command will fail (no API key), but it should NOT fail due to argument parsing.
    // A clap parse error exits with code 2; a runtime error exits with code 1.
    let code = output.status.code().unwrap_or(0);
    assert_ne!(code, 2, "clap argument parsing failed");
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
    lin().args(["issues", "get"]).assert().failure();
}

#[test]
fn test_unknown_command() {
    lin().arg("nonexistent").assert().failure();
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

// --- Projects ---

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

#[test]
fn test_projects_list_help() {
    lin()
        .args(["projects", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--status"))
        .stdout(predicate::str::contains("--team"))
        .stdout(predicate::str::contains("--limit"));
}

// --- Cycles ---

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
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("archive"));
}

#[test]
fn test_cycles_list_help() {
    lin()
        .args(["cycles", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--team"))
        .stdout(predicate::str::contains("--type"));
}

// --- Initiatives ---

#[test]
fn test_initiatives_help() {
    lin()
        .args(["initiatives", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("archive"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("projects"));
}

// --- Roadmap ---

#[test]
fn test_roadmap_help() {
    lin()
        .args(["roadmap", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("updates"))
        .stdout(predicate::str::contains("post"))
        .stdout(predicate::str::contains("milestones"))
        .stdout(predicate::str::contains("create-milestone"))
        .stdout(predicate::str::contains("update-milestone"))
        .stdout(predicate::str::contains("delete-milestone"));
}

// --- Labels ---

#[test]
fn test_labels_help() {
    lin()
        .args(["labels", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("apply"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("usage"));
}

// --- Teams ---

#[test]
fn test_teams_help() {
    lin()
        .args(["teams", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("members"))
        .stdout(predicate::str::contains("states"))
        .stdout(predicate::str::contains("workload"));
}

// --- Relations ---

#[test]
fn test_relations_help() {
    lin()
        .args(["relations", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("blocks"))
        .stdout(predicate::str::contains("blocked-by"))
        .stdout(predicate::str::contains("relates"))
        .stdout(predicate::str::contains("duplicate"))
        .stdout(predicate::str::contains("remove"));
}

// --- Customers ---

#[test]
fn test_customers_help() {
    lin()
        .args(["customers", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("link"))
        .stdout(predicate::str::contains("needs"))
        .stdout(predicate::str::contains("tiers"))
        .stdout(predicate::str::contains("create-tier"));
}

// --- Views ---

#[test]
fn test_views_help() {
    lin()
        .args(["views", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("issues"));
}

// --- Docs ---

#[test]
fn test_docs_help() {
    lin()
        .args(["docs", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

// --- Me ---

#[test]
fn test_me_help() {
    lin().args(["me", "--help"]).assert().success();
}

// --- Notifications ---

#[test]
fn test_notifications_help() {
    lin()
        .args(["notifications", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("read"))
        .stdout(predicate::str::contains("archive"))
        .stdout(predicate::str::contains("snooze"))
        .stdout(predicate::str::contains("unsnooze"));
}

// --- Attachments ---

#[test]
fn test_attachments_help() {
    lin()
        .args(["attachments", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("link-url"))
        .stdout(predicate::str::contains("delete"));
}

// --- Search ---

#[test]
fn test_search_help() {
    lin()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_search_missing_arg() {
    lin().args(["search"]).assert().failure();
}

// --- Config ---

#[test]
fn test_config_help() {
    lin()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("set-token"))
        .stdout(predicate::str::contains("get-token"))
        .stdout(predicate::str::contains("path"));
}

#[test]
fn test_config_path() {
    lin()
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("config.toml"));
}

#[test]
fn test_config_path_json() {
    lin()
        .args(["--json", "config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"path\""))
        .stdout(predicate::str::contains("config.toml"));
}

#[test]
fn test_config_get_token_no_config() {
    lin().args(["config", "get-token"]).assert().success();
}

#[test]
fn test_config_missing_subcommand() {
    lin().args(["config"]).assert().failure();
}

// --- Completions ---

#[test]
fn test_completions_help() {
    lin()
        .args(["completions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("shell"));
}

#[test]
fn test_completions_bash() {
    lin()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lin"));
}

#[test]
fn test_completions_zsh() {
    lin()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lin"));
}

#[test]
fn test_completions_fish() {
    lin()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lin"));
}

#[test]
fn test_completions_powershell() {
    lin()
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("lin"));
}

#[test]
fn test_completions_missing_shell() {
    lin().args(["completions"]).assert().failure();
}

#[test]
fn test_completions_invalid_shell() {
    lin().args(["completions", "nushell"]).assert().failure();
}

// --- Top-level help lists all commands ---

#[test]
fn test_help_lists_all_commands() {
    lin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("issues"))
        .stdout(predicate::str::contains("projects"))
        .stdout(predicate::str::contains("cycles"))
        .stdout(predicate::str::contains("initiatives"))
        .stdout(predicate::str::contains("roadmap"))
        .stdout(predicate::str::contains("labels"))
        .stdout(predicate::str::contains("teams"))
        .stdout(predicate::str::contains("relations"))
        .stdout(predicate::str::contains("customers"))
        .stdout(predicate::str::contains("views"))
        .stdout(predicate::str::contains("docs"))
        .stdout(predicate::str::contains("notifications"))
        .stdout(predicate::str::contains("me"))
        .stdout(predicate::str::contains("attachments"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("completions"))
        .stdout(predicate::str::contains("config"));
}

// --- New babyclaw flags ---

#[test]
fn test_projects_update_add_team_flag() {
    // Verify --add-team parses; will fail at auth, not arg parse
    let output = lin()
        .args(["projects", "update", "My Project", "--add-team", "ENG"])
        .output()
        .expect("failed to run command");
    let code = output.status.code().unwrap_or(0);
    assert_ne!(code, 2, "clap argument parsing failed");
}

#[test]
fn test_cycles_add_cycle_flag() {
    let output = lin()
        .args(["cycles", "add", "ENG-123", "--cycle", "cycle-uuid-here"])
        .output()
        .expect("failed to run command");
    let code = output.status.code().unwrap_or(0);
    assert_ne!(code, 2, "clap argument parsing failed");
}

#[test]
fn test_cycles_add_requires_team_or_cycle() {
    lin()
        .args(["cycles", "add", "ENG-123"])
        .assert()
        .failure();
}

#[test]
fn test_issues_update_team_flag() {
    let output = lin()
        .args(["issues", "update", "ENG-123", "--team", "HP"])
        .output()
        .expect("failed to run command");
    let code = output.status.code().unwrap_or(0);
    assert_ne!(code, 2, "clap argument parsing failed");
}

#[test]
fn test_api_help() {
    lin()
        .args(["api", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("GraphQL"));
}

#[test]
fn test_api_missing_query() {
    lin().args(["api"]).assert().failure();
}

#[test]
fn test_api_variables_invalid_json() {
    lin()
        .args(["api", "{ viewer { id } }", "--variables", "not-json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON"));
}
