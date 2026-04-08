use clap::Args;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query
    pub query: String,
    /// Max results per type
    #[arg(long, default_value = "10")]
    pub limit: i32,
}

pub async fn execute(args: &SearchArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    // Search issues
    let issue_result = client
        .query_raw(
            r#"query($term: String!, $first: Int) {
            searchIssues(term: $term, first: $first) {
                nodes { id identifier title state { name } assignee { displayName } priority team { key } }
            }
        }"#,
            Some(serde_json::json!({"term": &args.query, "first": args.limit})),
        )
        .await?;

    // Search projects
    let project_result = client
        .query_raw(
            r#"query($term: String!, $first: Int) {
            searchProjects(term: $term, first: $first) {
                nodes { id name state lead { displayName } }
            }
        }"#,
            Some(serde_json::json!({"term": &args.query, "first": args.limit})),
        )
        .await?;

    // Search docs
    let doc_result = client
        .query_raw(
            r#"query($term: String!, $first: Int) {
            searchDocuments(term: $term, first: $first) {
                nodes { id title creator { displayName } updatedAt }
            }
        }"#,
            Some(serde_json::json!({"term": &args.query, "first": args.limit})),
        )
        .await?;

    if json {
        let combined = serde_json::json!({
            "issues": issue_result.pointer("/data/searchIssues/nodes"),
            "projects": project_result.pointer("/data/searchProjects/nodes"),
            "documents": doc_result.pointer("/data/searchDocuments/nodes"),
        });
        crate::output::print_json(&combined);
    } else {
        // Issues
        if let Some(issues) = issue_result
            .pointer("/data/searchIssues/nodes")
            .and_then(|v| v.as_array())
            && !issues.is_empty()
        {
            println!("\n  {}", crate::output::color::bold("Issues"));
            for issue in issues {
                crate::output::detail::print_issue_summary(issue);
            }
        }

        // Projects
        if let Some(projects) = project_result
            .pointer("/data/searchProjects/nodes")
            .and_then(|v| v.as_array())
            && !projects.is_empty()
        {
            println!("\n  {}", crate::output::color::bold("Projects"));
            let rows: Vec<Vec<String>> = projects
                .iter()
                .map(|p| {
                    vec![
                        p["name"].as_str().unwrap_or("-").to_string(),
                        p["state"].as_str().unwrap_or("-").to_string(),
                        p.pointer("/lead/displayName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("-")
                            .to_string(),
                    ]
                })
                .collect();
            crate::output::table::print_table(&["Name", "Status", "Lead"], &rows);
        }

        // Documents
        if let Some(docs) = doc_result
            .pointer("/data/searchDocuments/nodes")
            .and_then(|v| v.as_array())
            && !docs.is_empty()
        {
            println!("\n  {}", crate::output::color::bold("Documents"));
            let rows: Vec<Vec<String>> = docs
                .iter()
                .map(|d| {
                    vec![
                        d["title"].as_str().unwrap_or("-").to_string(),
                        d.pointer("/creator/displayName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("-")
                            .to_string(),
                        d["updatedAt"]
                            .as_str()
                            .unwrap_or("-")
                            .chars()
                            .take(10)
                            .collect(),
                    ]
                })
                .collect();
            crate::output::table::print_table(&["Title", "Creator", "Updated"], &rows);
        }

        // Check if nothing found
        let all_empty = issue_result
            .pointer("/data/searchIssues/nodes")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(true)
            && project_result
                .pointer("/data/searchProjects/nodes")
                .and_then(|v| v.as_array())
                .map(|a| a.is_empty())
                .unwrap_or(true)
            && doc_result
                .pointer("/data/searchDocuments/nodes")
                .and_then(|v| v.as_array())
                .map(|a| a.is_empty())
                .unwrap_or(true);
        if all_empty {
            println!("  No results found for '{}'.", args.query);
        }
    }
    Ok(())
}
