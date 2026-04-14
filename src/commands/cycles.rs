use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct CyclesArgs {
    #[command(subcommand)]
    pub command: CyclesCommand,
}

#[derive(Subcommand, Debug)]
pub enum CyclesCommand {
    /// List cycles for a team
    List {
        /// Team key or name
        #[arg(long)]
        team: String,
        /// Filter type: current, previous, next, all
        #[arg(long, default_value = "all")]
        r#type: String,
    },
    /// Get cycle details
    Get {
        /// Cycle ID
        cycle_id: String,
    },
    /// List issues in the active cycle
    Issues {
        /// Team key or name
        #[arg(long)]
        team: String,
    },
    /// Create a cycle
    Create {
        /// Team key or name
        #[arg(long)]
        team: String,
        /// Cycle name
        #[arg(long)]
        name: Option<String>,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start: String,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        end: String,
    },
    /// Update a cycle
    Update {
        /// Cycle ID
        cycle_id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New start date
        #[arg(long)]
        start: Option<String>,
        /// New end date
        #[arg(long)]
        end: Option<String>,
    },
    /// Add an issue to the active cycle
    Add {
        /// Issue identifier (e.g., ENG-123)
        issue_id: String,
        /// Team key or name (uses active cycle)
        #[arg(long)]
        team: Option<String>,
        /// Cycle ID (overrides --team active cycle lookup)
        #[arg(long)]
        cycle: Option<String>,
    },
    /// Remove an issue from its cycle
    Remove {
        /// Issue identifier (e.g., ENG-123)
        issue_id: String,
    },
    /// Archive a cycle
    Archive {
        /// Cycle ID
        cycle_id: String,
    },
}

pub async fn execute(args: &CyclesArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    match &args.command {
        CyclesCommand::List { team, r#type } => {
            let team_id = client.get_team_id(team).await?;
            let query = r#"
                query($teamId: String!) {
                    team(id: $teamId) {
                        cycles(first: 50, orderBy: updatedAt) {
                            nodes {
                                id name number
                                startsAt endsAt
                                completedAt
                            }
                        }
                        activeCycle { id }
                    }
                }
            "#;
            let variables = json!({ "teamId": team_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let active_id = result
                    .pointer("/data/team/activeCycle/id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let nodes = result
                    .pointer("/data/team/cycles/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(cycles) if !cycles.is_empty() => {
                        let filtered: Vec<&serde_json::Value> = cycles
                            .iter()
                            .filter(|c| {
                                let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                let is_active = id == active_id;
                                let completed = c.get("completedAt").is_some_and(|v| !v.is_null());
                                match r#type.as_str() {
                                    "current" => is_active,
                                    "previous" => completed,
                                    "next" => !is_active && !completed,
                                    _ => true,
                                }
                            })
                            .collect();
                        if filtered.is_empty() {
                            println!("  No cycles found.");
                        } else {
                            let rows: Vec<Vec<String>> = filtered
                                .iter()
                                .map(|c| {
                                    let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("-");
                                    let name_or_number = c
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| {
                                            c.get("number")
                                                .and_then(|v| v.as_i64())
                                                .map(|n| format!("Cycle {n}"))
                                                .unwrap_or_else(|| "-".to_string())
                                        });
                                    let is_active = id == active_id;
                                    let status = if is_active {
                                        "Active".to_string()
                                    } else if c.get("completedAt").is_some_and(|v| !v.is_null()) {
                                        "Completed".to_string()
                                    } else {
                                        "Upcoming".to_string()
                                    };
                                    let start = c
                                        .get("startsAt")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.chars().take(10).collect())
                                        .unwrap_or_else(|| "-".to_string());
                                    let end = c
                                        .get("endsAt")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.chars().take(10).collect())
                                        .unwrap_or_else(|| "-".to_string());
                                    vec![name_or_number, status, start, end]
                                })
                                .collect();
                            crate::output::table::print_table(
                                &["Name", "Status", "Start", "End"],
                                &rows,
                            );
                        }
                    }
                    _ => println!("  No cycles found."),
                }
            }
        }

        CyclesCommand::Get { cycle_id } => {
            let query = r#"
                query($id: String!) {
                    cycle(id: $id) {
                        id name number
                        startsAt endsAt completedAt
                        team { key name }
                        issues {
                            nodes {
                                id identifier title
                                state { name }
                                assignee { displayName }
                                priority
                                team { key }
                            }
                        }
                    }
                }
            "#;
            let variables = json!({ "id": cycle_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let cycle = result
                    .pointer("/data/cycle")
                    .ok_or_else(|| anyhow::anyhow!("Cycle not found: {cycle_id}"))?;

                let name = cycle.get("name").and_then(|v| v.as_str()).unwrap_or({
                    // fallback handled below
                    "Cycle"
                });
                let number = cycle.get("number").and_then(|v| v.as_i64()).unwrap_or(0);
                let display = if name == "Cycle" {
                    format!("Cycle {number}")
                } else {
                    name.to_string()
                };
                println!("\n  {}", crate::output::color::bold(&display));
                println!();

                if let Some(team_key) = cycle.pointer("/team/key").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Team", team_key, 0);
                }
                if let Some(start) = cycle.get("startsAt").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Start", &start[..10.min(start.len())], 0);
                }
                if let Some(end) = cycle.get("endsAt").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("End", &end[..10.min(end.len())], 0);
                }

                if let Some(issues) = cycle.pointer("/issues/nodes").and_then(|v| v.as_array())
                    && !issues.is_empty()
                {
                    crate::output::detail::print_section("Issues");
                    for issue in issues {
                        crate::output::detail::print_issue_summary(issue);
                    }
                }
            }
        }

        CyclesCommand::Issues { team } => {
            let team_id = client.get_team_id(team).await?;
            let query = r#"
                query($teamId: String!) {
                    team(id: $teamId) {
                        activeCycle {
                            id name number
                            issues {
                                nodes {
                                    id identifier title
                                    state { name }
                                    assignee { displayName }
                                    priority
                                    team { key }
                                }
                            }
                        }
                    }
                }
            "#;
            let variables = json!({ "teamId": team_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/team/activeCycle/issues/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(issues) if !issues.is_empty() => {
                        for issue in issues {
                            crate::output::detail::print_issue_summary(issue);
                        }
                    }
                    _ => println!("  No issues in active cycle."),
                }
            }
        }

        CyclesCommand::Create {
            team,
            name,
            start,
            end,
        } => {
            let team_id = client.get_team_id(team).await?;
            let query = r#"
                mutation($input: CycleCreateInput!) {
                    cycleCreate(input: $input) {
                        success
                        cycle { id name number }
                    }
                }
            "#;

            let mut input = json!({
                "teamId": team_id,
                "startsAt": start,
                "endsAt": end,
            });

            if let Some(n) = name {
                input["name"] = json!(n);
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/cycleCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!("  {} Created cycle", crate::output::color::green("OK"),);
                } else {
                    println!(
                        "  {} Failed to create cycle",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        CyclesCommand::Update {
            cycle_id,
            name,
            start,
            end,
        } => {
            let query = r#"
                mutation($id: String!, $input: CycleUpdateInput!) {
                    cycleUpdate(id: $id, input: $input) {
                        success
                        cycle { id name number }
                    }
                }
            "#;

            let mut input = json!({});
            if let Some(n) = name {
                input["name"] = json!(n);
            }
            if let Some(s) = start {
                input["startsAt"] = json!(s);
            }
            if let Some(e) = end {
                input["endsAt"] = json!(e);
            }

            let variables = json!({ "id": cycle_id, "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/cycleUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Updated cycle {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(cycle_id),
                    );
                } else {
                    println!(
                        "  {} Failed to update cycle",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        CyclesCommand::Add {
            issue_id,
            team,
            cycle,
        } => {
            let cycle_id = match (cycle, team) {
                (Some(cid), _) => cid.clone(),
                (None, Some(team_name)) => {
                    let team_id = client.get_team_id(team_name).await?;
                    let cycle_query = r#"
                        query($teamId: String!) {
                            team(id: $teamId) {
                                activeCycle { id }
                            }
                        }
                    "#;
                    let cycle_result = client
                        .query_raw(cycle_query, Some(json!({ "teamId": team_id })))
                        .await?;
                    cycle_result
                        .pointer("/data/team/activeCycle/id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("No active cycle for team {team_name}"))?
                        .to_string()
                }
                (None, None) => {
                    anyhow::bail!("Provide --team (uses active cycle) or --cycle <id>");
                }
            };

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
                "input": { "cycleId": cycle_id },
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
                        "  {} Added {} to cycle",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(issue_id),
                    );
                } else {
                    println!(
                        "  {} Failed to add issue to cycle",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        CyclesCommand::Remove { issue_id } => {
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
                "input": { "cycleId": null },
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
                        "  {} Removed {} from cycle",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(issue_id),
                    );
                } else {
                    println!(
                        "  {} Failed to remove issue from cycle",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        CyclesCommand::Archive { cycle_id } => {
            let query = r#"
                mutation($id: String!) {
                    cycleArchive(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": cycle_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/cycleArchive/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Archived cycle {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(cycle_id),
                    );
                } else {
                    println!(
                        "  {} Failed to archive cycle",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }
    }

    Ok(())
}
