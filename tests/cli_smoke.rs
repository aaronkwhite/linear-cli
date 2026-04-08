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
        .stdout(predicate::str::contains("completions"));
}
