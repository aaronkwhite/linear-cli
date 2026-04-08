use clap::{Args, Subcommand};
use serde_json::json;
use std::collections::HashMap;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct TeamsArgs {
    #[command(subcommand)]
    pub command: TeamsCommand,
}

#[derive(Subcommand, Debug)]
pub enum TeamsCommand {
    /// List all teams
    List,
    /// Get team details
    Get {
        /// Team key (e.g., ENG)
        key: String,
    },
    /// List team members
    Members {
        /// Team key
        key: String,
    },
    /// List workflow states
    States {
        /// Team key
        key: String,
    },
    /// Show workload distribution
    Workload {
        /// Team key
        key: String,
        /// Max issues to analyze
        #[arg(long, default_value = "200")]
        limit: i32,
    },
}

pub async fn execute(args: &TeamsArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    match &args.command {
        TeamsCommand::List => {
            let query = r#"
                query {
                    teams {
                        nodes { id key name description }
                    }
                }
            "#;
            let result = client.query_raw(query, None).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/teams/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(teams) if !teams.is_empty() => {
                        let rows: Vec<Vec<String>> = teams
                            .iter()
                            .map(|t| {
                                vec![
                                    t.get("key")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    t.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    t.get("description")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(&["Key", "Name", "Description"], &rows);
                    }
                    _ => println!("  No teams found."),
                }
            }
        }

        TeamsCommand::Get { key } => {
            let team_id = client.get_team_id(key).await?;
            let query = r#"
                query($id: ID!) {
                    team(id: $id) {
                        id key name description timezone
                        members { nodes { id } }
                        projects { nodes { id name state } }
                        activeCycle { id name number startsAt endsAt }
                    }
                }
            "#;
            let variables = json!({ "id": team_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let team = result
                    .pointer("/data/team")
                    .ok_or_else(|| anyhow::anyhow!("Team not found: {key}"))?;

                let name = team.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                let tkey = team.get("key").and_then(|v| v.as_str()).unwrap_or("?");
                println!(
                    "\n  {} ({})",
                    crate::output::color::bold(name),
                    crate::output::color::dim(tkey)
                );
                println!();

                if let Some(desc) = team.get("description").and_then(|v| v.as_str())
                    && !desc.is_empty()
                {
                    crate::output::detail::print_detail("Description", desc, 0);
                }
                if let Some(tz) = team.get("timezone").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Timezone", tz, 0);
                }
                if let Some(members) = team.pointer("/members/nodes").and_then(|v| v.as_array()) {
                    crate::output::detail::print_detail(
                        "Members",
                        &format!("{}", members.len()),
                        0,
                    );
                }
                if let Some(projects) = team.pointer("/projects/nodes").and_then(|v| v.as_array()) {
                    let names: Vec<&str> = projects
                        .iter()
                        .filter_map(|p| p.get("name").and_then(|v| v.as_str()))
                        .collect();
                    if !names.is_empty() {
                        crate::output::detail::print_detail("Projects", &names.join(", "), 0);
                    }
                }

                if let Some(cycle) = team.get("activeCycle")
                    && !cycle.is_null()
                {
                    let cycle_name = cycle.get("name").and_then(|v| v.as_str()).unwrap_or({
                        // handled below
                        "Active Cycle"
                    });
                    let start = cycle
                        .get("startsAt")
                        .and_then(|v| v.as_str())
                        .map(|s| s.chars().take(10).collect::<String>())
                        .unwrap_or_default();
                    let end = cycle
                        .get("endsAt")
                        .and_then(|v| v.as_str())
                        .map(|s| s.chars().take(10).collect::<String>())
                        .unwrap_or_default();
                    crate::output::detail::print_detail(
                        "Active Cycle",
                        &format!("{cycle_name} ({start} to {end})"),
                        0,
                    );
                }
            }
        }

        TeamsCommand::Members { key } => {
            let team_id = client.get_team_id(key).await?;
            let query = r#"
                query($id: ID!) {
                    team(id: $id) {
                        members {
                            nodes {
                                id displayName admin
                                assignedIssues { nodes { id } }
                            }
                        }
                    }
                }
            "#;
            let variables = json!({ "id": team_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/team/members/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(members) if !members.is_empty() => {
                        let rows: Vec<Vec<String>> = members
                            .iter()
                            .map(|m| {
                                let name = m
                                    .get("displayName")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("-")
                                    .to_string();
                                let role =
                                    if m.get("admin").and_then(|v| v.as_bool()).unwrap_or(false) {
                                        "Admin".to_string()
                                    } else {
                                        "Member".to_string()
                                    };
                                let issues = m
                                    .pointer("/assignedIssues/nodes")
                                    .and_then(|v| v.as_array())
                                    .map(|a| a.len())
                                    .unwrap_or(0);
                                vec![name, role, format!("{issues}")]
                            })
                            .collect();
                        crate::output::table::print_table(
                            &["Name", "Role", "Assigned Issues"],
                            &rows,
                        );
                    }
                    _ => println!("  No members found."),
                }
            }
        }

        TeamsCommand::States { key } => {
            let team_id = client.get_team_id(key).await?;
            let query = r#"
                query($teamId: ID!) {
                    workflowStates(filter: { team: { id: { eq: $teamId } } }) {
                        nodes { id name type position }
                    }
                }
            "#;
            let variables = json!({ "teamId": team_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/workflowStates/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(states) if !states.is_empty() => {
                        let mut sorted: Vec<&serde_json::Value> = states.iter().collect();
                        // Sort by type order then position
                        let type_order = |t: &str| -> i32 {
                            match t {
                                "triage" => 0,
                                "backlog" => 1,
                                "unstarted" => 2,
                                "started" => 3,
                                "completed" => 4,
                                "cancelled" | "canceled" => 5,
                                _ => 6,
                            }
                        };
                        sorted.sort_by(|a, b| {
                            let at = a.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            let bt = b.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            let ap = a.get("position").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let bp = b.get("position").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            type_order(at)
                                .cmp(&type_order(bt))
                                .then(ap.partial_cmp(&bp).unwrap_or(std::cmp::Ordering::Equal))
                        });

                        let rows: Vec<Vec<String>> = sorted
                            .iter()
                            .map(|s| {
                                vec![
                                    s.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    s.get("type")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(&["Name", "Type"], &rows);
                    }
                    _ => println!("  No workflow states found."),
                }
            }
        }

        TeamsCommand::Workload { key, limit } => {
            let team_id = client.get_team_id(key).await?;
            let query = r#"
                query($filter: IssueFilter, $first: Int!) {
                    issues(filter: $filter, first: $first) {
                        nodes {
                            assignee { displayName }
                            state { name type }
                        }
                    }
                }
            "#;
            let filter = json!({
                "team": { "id": { "eq": team_id } },
                "state": { "type": { "in": ["started", "unstarted", "backlog"] } }
            });
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
                        // Group by assignee
                        let mut by_assignee: HashMap<String, HashMap<String, i32>> = HashMap::new();
                        for issue in issues {
                            let assignee = issue
                                .pointer("/assignee/displayName")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unassigned")
                                .to_string();
                            let status = issue
                                .pointer("/state/name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            *by_assignee
                                .entry(assignee)
                                .or_default()
                                .entry(status)
                                .or_insert(0) += 1;
                        }

                        let mut rows: Vec<Vec<String>> = by_assignee
                            .iter()
                            .map(|(name, statuses)| {
                                let total: i32 = statuses.values().sum();
                                let breakdown: Vec<String> =
                                    statuses.iter().map(|(s, c)| format!("{s}: {c}")).collect();
                                vec![name.clone(), format!("{total}"), breakdown.join(", ")]
                            })
                            .collect();
                        rows.sort_by(|a, b| {
                            b[1].parse::<i32>()
                                .unwrap_or(0)
                                .cmp(&a[1].parse::<i32>().unwrap_or(0))
                        });

                        crate::output::table::print_table(
                            &["Assignee", "Total", "Breakdown"],
                            &rows,
                        );
                    }
                    _ => println!("  No active issues found."),
                }
            }
        }
    }

    Ok(())
}
