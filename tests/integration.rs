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
    lin().args(["teams", "list"]).assert().success();
}

#[test]
#[ignore]
fn test_issues_list() {
    if !has_api_key() { return; }
    lin().args(["issues", "list", "--limit", "3"]).assert().success();
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
    lin().args(["projects", "list", "--limit", "3"]).assert().success();
}

#[test]
#[ignore]
fn test_notifications_list() {
    if !has_api_key() { return; }
    lin().args(["notifications", "list", "--limit", "3"]).assert().success();
}

#[test]
#[ignore]
fn test_docs_list() {
    if !has_api_key() { return; }
    lin().args(["docs", "list", "--limit", "3"]).assert().success();
}

#[test]
#[ignore]
fn test_labels_list() {
    if !has_api_key() { return; }
    lin().args(["labels", "list"]).assert().success();
}
