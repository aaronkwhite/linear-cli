use super::color::{bold, dim, format_priority, green, red, yellow};
use serde_json::Value;

pub fn print_detail(label: &str, value: &str, indent: usize) {
    let pad = " ".repeat(indent * 2);
    println!("{pad}  {}: {value}", bold(label));
}

pub fn print_section(title: &str) {
    println!("\n  {}", bold(title));
    println!("  {}", "─".repeat(title.len()));
}

pub fn format_user(user: Option<&Value>) -> String {
    user.and_then(|u| u.get("displayName").or_else(|| u.get("name")))
        .and_then(|v| v.as_str())
        .unwrap_or("-")
        .to_string()
}

pub fn format_health(health: &str) -> String {
    match health {
        "onTrack" => green("On Track"),
        "atRisk" => yellow("At Risk"),
        "offTrack" => red("Off Track"),
        _ => health.to_string(),
    }
}

pub fn print_issue_summary(issue: &Value) {
    let identifier = issue
        .get("identifier")
        .and_then(|v| v.as_str())
        .unwrap_or("???");

    let status = issue
        .pointer("/state/name")
        .and_then(|v| v.as_str())
        .unwrap_or("-");

    let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("");

    let assignee = issue
        .pointer("/assignee/displayName")
        .and_then(|v| v.as_str())
        .map(|name| dim(&format!("@{name}")))
        .unwrap_or_default();

    let priority = issue.get("priority").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    println!(
        "  {:<10} {:<14} {}  {} {}",
        bold(identifier),
        status,
        title,
        assignee,
        format_priority(priority),
    );
}

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

    let priority = issue.get("priority").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    if priority > 0 {
        let label = match priority {
            1 => "Urgent",
            2 => "High",
            3 => "Medium",
            4 => "Low",
            _ => "None",
        };
        print_detail(
            "Priority",
            &format!("{} {}", label, format_priority(priority)),
            0,
        );
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
            let pid = parent
                .get("identifier")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
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

    if let Some(desc) = issue.get("description").and_then(|v| v.as_str()) {
        if !desc.is_empty() {
            print_section("Description");
            let skin = termimad::MadSkin::default();
            let rendered = skin.term_text(desc);
            for line in rendered.to_string().lines() {
                println!("  {line}");
            }
        }
    }

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
                let body = comment.get("body").and_then(|v| v.as_str()).unwrap_or("");

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
