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
