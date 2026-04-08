use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct LabelsArgs {
    #[command(subcommand)]
    pub command: LabelsCommand,
}

#[derive(Subcommand, Debug)]
pub enum LabelsCommand {
    /// List labels
    List {
        /// Filter by team key
        #[arg(long)]
        team: Option<String>,
    },
    /// Create a label
    Create {
        /// Label name
        name: String,
        /// Team key or name
        #[arg(long)]
        team: Option<String>,
        /// Color hex (e.g., #ff0000)
        #[arg(long)]
        color: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Parent label name
        #[arg(long)]
        parent: Option<String>,
    },
    /// Update a label
    Update {
        /// Label name
        name: String,
        /// New name
        #[arg(long)]
        new_name: Option<String>,
        /// New color
        #[arg(long)]
        color: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a label
    Delete {
        /// Label name
        name: String,
        /// Team key (to scope lookup)
        #[arg(long)]
        team: Option<String>,
    },
    /// Apply labels to an issue (merges with existing)
    Apply {
        /// Issue identifier (e.g., ENG-123)
        issue_id: String,
        /// Comma-separated label names
        label_names: String,
    },
    /// Remove labels from an issue
    Remove {
        /// Issue identifier (e.g., ENG-123)
        issue_id: String,
        /// Comma-separated label names to remove
        label_names: String,
    },
    /// List issues with a label
    Usage {
        /// Label name
        name: String,
        /// Filter by team
        #[arg(long)]
        team: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
}

pub async fn execute(args: &LabelsArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    match &args.command {
        LabelsCommand::List { team } => {
            let resolved_team = match team {
                Some(t) => Some(t.clone()),
                None if crate::output::interactive::is_interactive() => {
                    let teams = client.get_teams().await?;
                    let items: Vec<String> = teams
                        .iter()
                        .map(|t| format!("{} ({})", t.name, t.key))
                        .collect();
                    let idx = crate::output::interactive::fuzzy_select("Select team", &items)?;
                    Some(teams[idx].key.clone())
                }
                None => None,
            };

            let (query, variables) = if let Some(team_key) = &resolved_team {
                let team_id = client.get_team_id(team_key).await?;
                (
                    r#"
                        query($filter: IssueLabelFilter) {
                            issueLabels(filter: $filter, first: 250) {
                                nodes { id name color description }
                            }
                        }
                    "#,
                    json!({ "filter": { "team": { "id": { "eq": team_id } } } }),
                )
            } else {
                (
                    r#"
                        query($filter: IssueLabelFilter) {
                            issueLabels(filter: $filter, first: 250) {
                                nodes { id name color description }
                            }
                        }
                    "#,
                    json!({ "filter": {} }),
                )
            };
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/issueLabels/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(labels) if !labels.is_empty() => {
                        let rows: Vec<Vec<String>> = labels
                            .iter()
                            .map(|l| {
                                vec![
                                    l.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    l.get("color")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    l.get("description")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(&["Name", "Color", "Description"], &rows);
                    }
                    _ => println!("  No labels found."),
                }
            }
        }

        LabelsCommand::Create {
            name,
            team,
            color,
            description,
            parent,
        } => {
            let query = r#"
                mutation($input: IssueLabelCreateInput!) {
                    issueLabelCreate(input: $input) {
                        success
                        issueLabel { id name }
                    }
                }
            "#;

            let mut input = json!({ "name": name });
            if let Some(team_key) = team {
                let team_id = client.get_team_id(team_key).await?;
                input["teamId"] = json!(team_id);
            }
            if let Some(c) = color {
                input["color"] = json!(c);
            }
            if let Some(desc) = description {
                input["description"] = json!(desc);
            }
            if let Some(parent_name) = parent {
                let parent_ids = client
                    .get_label_ids(&[parent_name.as_str()], team.as_deref())
                    .await?;
                if let Some(pid) = parent_ids.first() {
                    input["parentId"] = json!(pid);
                }
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueLabelCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Created label {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to create label",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        LabelsCommand::Update {
            name,
            new_name,
            color,
            description,
        } => {
            // Resolve label ID by name
            let label_ids = client.get_label_ids(&[name.as_str()], None).await?;
            let label_id = label_ids
                .first()
                .ok_or_else(|| anyhow::anyhow!("Label not found: {name}"))?;

            let query = r#"
                mutation($id: String!, $input: IssueLabelUpdateInput!) {
                    issueLabelUpdate(id: $id, input: $input) {
                        success
                        issueLabel { id name }
                    }
                }
            "#;

            let mut input = json!({});
            if let Some(n) = new_name {
                input["name"] = json!(n);
            }
            if let Some(c) = color {
                input["color"] = json!(c);
            }
            if let Some(desc) = description {
                input["description"] = json!(desc);
            }

            let variables = json!({ "id": label_id, "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueLabelUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Updated label {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to update label",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        LabelsCommand::Delete { name, team } => {
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!("Delete label {}?", name))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let label_ids = client
                .get_label_ids(&[name.as_str()], team.as_deref())
                .await?;
            let label_id = label_ids
                .first()
                .ok_or_else(|| anyhow::anyhow!("Label not found: {name}"))?;

            let query = r#"
                mutation($id: String!) {
                    issueLabelDelete(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": label_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueLabelDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Deleted label {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to delete label",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        LabelsCommand::Apply {
            issue_id,
            label_names,
        } => {
            // Fetch current labels on the issue
            let get_query = r#"
                query($id: String!) {
                    issue(id: $id) {
                        team { key }
                        labels { nodes { id name } }
                    }
                }
            "#;
            let get_result = client
                .query_raw(get_query, Some(json!({ "id": issue_id })))
                .await?;
            let issue = get_result
                .pointer("/data/issue")
                .ok_or_else(|| anyhow::anyhow!("Issue not found: {issue_id}"))?;
            let team_key = issue
                .pointer("/team/key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Could not determine team for {issue_id}"))?;

            let current_ids: Vec<String> = issue
                .pointer("/labels/nodes")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|l| l.get("id").and_then(|v| v.as_str()).map(String::from))
                .collect();

            let new_names: Vec<&str> = label_names.split(',').map(|s| s.trim()).collect();
            let new_ids = client.get_label_ids(&new_names, Some(team_key)).await?;

            // Merge: existing + new (deduplicated)
            let mut all_ids = current_ids;
            for id in new_ids {
                if !all_ids.contains(&id) {
                    all_ids.push(id);
                }
            }

            let query = r#"
                mutation($id: String!, $input: IssueUpdateInput!) {
                    issueUpdate(id: $id, input: $input) {
                        success
                        issue { id identifier }
                    }
                }
            "#;
            let variables = json!({
                "id": issue_id,
                "input": { "labelIds": all_ids },
            });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Applied labels to {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(issue_id),
                    );
                } else {
                    println!(
                        "  {} Failed to apply labels",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        LabelsCommand::Remove {
            issue_id,
            label_names,
        } => {
            // Fetch current labels
            let get_query = r#"
                query($id: String!) {
                    issue(id: $id) {
                        team { key }
                        labels { nodes { id name } }
                    }
                }
            "#;
            let get_result = client
                .query_raw(get_query, Some(json!({ "id": issue_id })))
                .await?;
            let issue = get_result
                .pointer("/data/issue")
                .ok_or_else(|| anyhow::anyhow!("Issue not found: {issue_id}"))?;

            let remove_names: Vec<&str> = label_names.split(',').map(|s| s.trim()).collect();
            let remove_lower: Vec<String> = remove_names.iter().map(|n| n.to_lowercase()).collect();

            let remaining_ids: Vec<String> = issue
                .pointer("/labels/nodes")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter(|l| {
                    let name = l
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_lowercase();
                    !remove_lower.contains(&name)
                })
                .filter_map(|l| l.get("id").and_then(|v| v.as_str()).map(String::from))
                .collect();

            let query = r#"
                mutation($id: String!, $input: IssueUpdateInput!) {
                    issueUpdate(id: $id, input: $input) {
                        success
                        issue { id identifier }
                    }
                }
            "#;
            let variables = json!({
                "id": issue_id,
                "input": { "labelIds": remaining_ids },
            });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Removed labels from {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(issue_id),
                    );
                } else {
                    println!(
                        "  {} Failed to remove labels",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        LabelsCommand::Usage { name, team, limit } => {
            let mut filter = json!({
                "labels": { "name": { "containsIgnoreCase": name } }
            });
            if let Some(team_key) = team {
                let team_id = client.get_team_id(team_key).await?;
                filter["team"] = json!({ "id": { "eq": team_id } });
            }

            let query = r#"
                query($filter: IssueFilter, $first: Int!) {
                    issues(filter: $filter, first: $first, orderBy: updatedAt) {
                        nodes {
                            id identifier title
                            state { name }
                            assignee { displayName }
                            priority
                            team { key }
                        }
                    }
                }
            "#;
            let variables = json!({ "filter": filter, "first": limit });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/issues/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(issues) if !issues.is_empty() => {
                        for issue in issues {
                            crate::output::detail::print_issue_summary(issue);
                        }
                    }
                    _ => println!("  No issues found with label '{name}'."),
                }
            }
        }
    }

    Ok(())
}
