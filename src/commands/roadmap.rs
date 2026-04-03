use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct RoadmapArgs {
    #[command(subcommand)]
    pub command: RoadmapCommand,
}

#[derive(Subcommand, Debug)]
pub enum RoadmapCommand {
    /// List project status updates
    Updates {
        /// Project name
        project: String,
        /// Max results
        #[arg(long, default_value = "10")]
        limit: i32,
    },
    /// Post a project status update
    Post {
        /// Project name
        project: String,
        /// Update body text
        body: String,
        /// Health: onTrack, atRisk, offTrack
        #[arg(long, default_value = "onTrack")]
        health: String,
    },
    /// List project milestones
    Milestones {
        /// Project name
        project: String,
    },
    /// Create a project milestone
    CreateMilestone {
        /// Project name
        project: String,
        /// Milestone name
        name: String,
        /// Target date (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
    },
    /// Update a project milestone
    UpdateMilestone {
        /// Milestone ID
        milestone_id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New target date
        #[arg(long)]
        date: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a project milestone
    DeleteMilestone {
        /// Milestone ID
        milestone_id: String,
    },
    /// List initiatives
    Initiatives {
        /// Max results
        #[arg(long, default_value = "20")]
        limit: i32,
    },
}

pub async fn execute(args: &RoadmapArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    match &args.command {
        RoadmapCommand::Updates { project, limit } => {
            let project_id = client.get_project_id(project).await?;
            let query = r#"
                query($projectId: String!, $first: Int!) {
                    project(id: $projectId) {
                        projectUpdates(first: $first) {
                            nodes {
                                id body health createdAt
                                user { displayName }
                            }
                        }
                    }
                }
            "#;
            let variables = json!({ "projectId": project_id, "first": limit });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/project/projectUpdates/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(updates) if !updates.is_empty() => {
                        for update in updates {
                            let health =
                                update.get("health").and_then(|v| v.as_str()).unwrap_or("?");
                            let user = update
                                .pointer("/user/displayName")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?");
                            let date = update
                                .get("createdAt")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            let body = update.get("body").and_then(|v| v.as_str()).unwrap_or("");

                            println!(
                                "  {} {} {}",
                                crate::output::detail::format_health(health),
                                crate::output::color::bold(user),
                                crate::output::color::dim(date),
                            );
                            for line in body.lines().take(5) {
                                println!("    {line}");
                            }
                            println!();
                        }
                    }
                    _ => println!("  No updates found."),
                }
            }
        }

        RoadmapCommand::Post {
            project,
            body,
            health,
        } => {
            let project_id = client.get_project_id(project).await?;
            let query = r#"
                mutation($input: ProjectUpdateCreateInput!) {
                    projectUpdateCreate(input: $input) {
                        success
                        projectUpdate { id health }
                    }
                }
            "#;
            let variables = json!({
                "input": {
                    "projectId": project_id,
                    "body": body,
                    "health": health,
                }
            });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/projectUpdateCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Posted status update for {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(project),
                    );
                } else {
                    println!(
                        "  {} Failed to post update",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        RoadmapCommand::Milestones { project } => {
            let project_id = client.get_project_id(project).await?;
            let query = r#"
                query($projectId: String!) {
                    project(id: $projectId) {
                        projectMilestones {
                            nodes {
                                id name description targetDate
                            }
                        }
                    }
                }
            "#;
            let variables = json!({ "projectId": project_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/project/projectMilestones/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(milestones) if !milestones.is_empty() => {
                        let rows: Vec<Vec<String>> = milestones
                            .iter()
                            .map(|m| {
                                vec![
                                    m.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    m.get("targetDate")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    m.get("description")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    m.get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(
                            &["Name", "Target Date", "Description", "ID"],
                            &rows,
                        );
                    }
                    _ => println!("  No milestones found."),
                }
            }
        }

        RoadmapCommand::CreateMilestone {
            project,
            name,
            date,
            description,
        } => {
            let project_id = client.get_project_id(project).await?;
            let query = r#"
                mutation($input: ProjectMilestoneCreateInput!) {
                    projectMilestoneCreate(input: $input) {
                        success
                        projectMilestone { id name }
                    }
                }
            "#;

            let mut input = json!({
                "projectId": project_id,
                "name": name,
            });
            if let Some(d) = date {
                input["targetDate"] = json!(d);
            }
            if let Some(desc) = description {
                input["description"] = json!(desc);
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/projectMilestoneCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Created milestone {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to create milestone",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        RoadmapCommand::UpdateMilestone {
            milestone_id,
            name,
            date,
            description,
        } => {
            let query = r#"
                mutation($id: String!, $input: ProjectMilestoneUpdateInput!) {
                    projectMilestoneUpdate(id: $id, input: $input) {
                        success
                        projectMilestone { id name }
                    }
                }
            "#;

            let mut input = json!({});
            if let Some(n) = name {
                input["name"] = json!(n);
            }
            if let Some(d) = date {
                input["targetDate"] = json!(d);
            }
            if let Some(desc) = description {
                input["description"] = json!(desc);
            }

            let variables = json!({ "id": milestone_id, "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/projectMilestoneUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Updated milestone {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(milestone_id),
                    );
                } else {
                    println!(
                        "  {} Failed to update milestone",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        RoadmapCommand::DeleteMilestone { milestone_id } => {
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!(
                    "Delete milestone {}?",
                    milestone_id
                ))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let query = r#"
                mutation($id: String!) {
                    projectMilestoneDelete(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": milestone_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/projectMilestoneDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Deleted milestone {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(milestone_id),
                    );
                } else {
                    println!(
                        "  {} Failed to delete milestone",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        RoadmapCommand::Initiatives { limit } => {
            let query = r#"
                query($first: Int!) {
                    initiatives(first: $first) {
                        nodes {
                            id name description status
                            createdAt
                        }
                    }
                }
            "#;
            let variables = json!({ "first": limit });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/initiatives/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(initiatives) if !initiatives.is_empty() => {
                        let rows: Vec<Vec<String>> = initiatives
                            .iter()
                            .map(|i| {
                                vec![
                                    i.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    i.get("status")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    i.get("description")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(
                            &["Name", "Status", "Description"],
                            &rows,
                        );
                    }
                    _ => println!("  No initiatives found."),
                }
            }
        }
    }

    Ok(())
}
