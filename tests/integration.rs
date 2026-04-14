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
    if !has_api_key() {
        return;
    }
    lin().args(["teams", "list"]).assert().success();
}

#[test]
#[ignore]
fn test_issues_list() {
    if !has_api_key() {
        return;
    }
    lin()
        .args(["issues", "list", "--limit", "3"])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_issues_list_json() {
    if !has_api_key() {
        return;
    }
    lin()
        .args(["--json", "issues", "list", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("identifier"));
}

#[test]
#[ignore]
fn test_projects_list() {
    if !has_api_key() {
        return;
    }
    lin()
        .args(["projects", "list", "--limit", "3"])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_notifications_list() {
    if !has_api_key() {
        return;
    }
    lin()
        .args(["notifications", "list", "--limit", "3"])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_docs_list() {
    if !has_api_key() {
        return;
    }
    lin()
        .args(["docs", "list", "--limit", "3"])
        .assert()
        .success();
}

#[test]
#[ignore]
fn test_labels_list() {
    if !has_api_key() {
        return;
    }
    lin().args(["labels", "list"]).assert().success();
}

// --- babyclaw integration tests ---

#[test]
#[ignore]
fn test_api_viewer() {
    // lin api passthrough — simplest possible read-only query
    if !has_api_key() {
        return;
    }
    lin()
        .args(["api", "{ viewer { id displayName } }"])
        .assert()
        .success()
        .stdout(predicate::str::contains("id"));
}

#[test]
#[ignore]
fn test_api_viewer_json_shape() {
    // Output should contain viewer data, no GraphQL envelope
    if !has_api_key() {
        return;
    }
    let out = lin()
        .args(["api", "{ viewer { id displayName } }"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should have raw data object, not wrapped in {"data": ...}
    assert!(
        stdout.contains("\"viewer\""),
        "expected viewer key in output"
    );
    assert!(
        !stdout.contains("\"data\""),
        "output should not have GraphQL envelope"
    );
}

#[test]
#[ignore]
fn test_projects_search_json_shape() {
    // projects search --json should emit {"projects": [...]}
    if !has_api_key() {
        return;
    }
    let out = lin()
        .args(["--json", "projects", "search", "a", "--limit", "1"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("output is not valid JSON");
    assert!(
        v.get("projects").is_some(),
        "expected top-level 'projects' key, got: {stdout}"
    );
    assert!(
        v.get("searchProjects").is_none(),
        "should not have raw 'searchProjects' key"
    );
}

#[test]
#[ignore]
fn test_projects_issues_json_shape() {
    // projects issues --json should emit {"issues": [...]}
    // First find any project name, then check its issues shape.
    if !has_api_key() {
        return;
    }
    // Get first project name
    let proj_out = lin()
        .args(["--json", "projects", "list", "--limit", "1"])
        .output()
        .expect("failed to run");
    let proj_stdout = String::from_utf8_lossy(&proj_out.stdout);
    let proj_v: serde_json::Value =
        serde_json::from_str(&proj_stdout).expect("projects list output not valid JSON");
    let name = proj_v
        .pointer("/projects/nodes/0/name")
        .and_then(|v| v.as_str())
        .expect("no projects found");

    let out = lin()
        .args(["--json", "projects", "issues", name, "--limit", "1"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("output is not valid JSON");
    assert!(
        v.get("issues").is_some(),
        "expected top-level 'issues' key, got: {stdout}"
    );
}
