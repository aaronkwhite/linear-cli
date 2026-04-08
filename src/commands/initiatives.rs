use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct InitiativesArgs {
    #[command(subcommand)]
    pub command: InitiativesCommand,
}

#[derive(Subcommand, Debug)]
pub enum InitiativesCommand {
    /// List initiatives
    List {
        /// Filter by status (Planned/Active/Completed)
        #[arg(long)]
        status: Option<String>,
        /// Max results
        #[arg(long, default_value = "20")]
        limit: i32,
    },
    /// Get initiative details
    Get {
        /// Initiative name or ID
        name_or_id: String,
    },
    /// Create a new initiative
    Create {
        /// Initiative name (required)
        #[arg(long)]
        name: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Owner name
        #[arg(long)]
        owner: Option<String>,
        /// Status (Planned/Active/Completed)
        #[arg(long)]
        status: Option<String>,
        /// Target date (YYYY-MM-DD)
        #[arg(long)]
        target_date: Option<String>,
        /// Content (markdown body)
        #[arg(long)]
        content: Option<String>,
    },
    /// Update an initiative
    Update {
        /// Initiative name or ID
        name_or_id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New owner name
        #[arg(long)]
        owner: Option<String>,
        /// New status (Planned/Active/Completed)
        #[arg(long)]
        status: Option<String>,
        /// New target date (YYYY-MM-DD)
        #[arg(long)]
        target_date: Option<String>,
        /// New content (markdown body)
        #[arg(long)]
        content: Option<String>,
    },
    /// Archive an initiative
    Archive {
        /// Initiative name or ID
        name_or_id: String,
    },
    /// Delete an initiative
    Delete {
        /// Initiative name or ID
        name_or_id: String,
    },
    /// List projects belonging to an initiative
    Projects {
        /// Initiative name or ID
        name_or_id: String,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
}

async fn resolve_initiative_id(client: &LinearClient, name_or_id: &str) -> anyhow::Result<String> {
    // If it looks like a UUID, use directly
    if name_or_id.len() > 30 && name_or_id.contains('-') {
        return Ok(name_or_id.to_string());
    }
    // Otherwise search by name
    let result = client
        .query_raw(
            "query { initiatives(first: 250) { nodes { id name } } }",
            None,
        )
        .await?;
    let needle = name_or_id.to_lowercase();
    if let Some(nodes) = result
        .pointer("/data/initiatives/nodes")
        .and_then(|v| v.as_array())
    {
        for node in nodes {
            if let Some(name) = node.get("name").and_then(|v| v.as_str()) {
                if name.to_lowercase().contains(&needle) {
                    return Ok(node.get("id").and_then(|v| v.as_str()).unwrap().to_string());
                }
            }
        }
    }
    anyhow::bail!("Initiative not found: {name_or_id}")
}

pub async fn execute(args: &InitiativesArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    match &args.command {
        InitiativesCommand::List { status, limit } => {
            let query = r#"
                query($first: Int!) {
                    initiatives(first: $first) {
                        nodes {
                            id name status targetDate
                            owner { displayName }
                            projects { nodes { id name } }
                            health
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
                        let status_filter = status.as_ref().map(|s| s.to_lowercase());
                        let rows: Vec<Vec<String>> = initiatives
                            .iter()
                            .filter(|i| {
                                if let Some(ref filter) = status_filter {
                                    i.get("status")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_lowercase() == *filter)
                                        .unwrap_or(false)
                                } else {
                                    true
                                }
                            })
                            .map(|i| {
                                let project_count = i
                                    .pointer("/projects/nodes")
                                    .and_then(|v| v.as_array())
                                    .map(|a| a.len())
                                    .unwrap_or(0);
                                vec![
                                    i.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    i.get("status")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    i.get("health")
                                        .and_then(|v| v.as_str())
                                        .map(crate::output::detail::format_health)
                                        .unwrap_or_else(|| "-".to_string()),
                                    crate::output::detail::format_user(i.get("owner")),
                                    i.get("targetDate")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    project_count.to_string(),
                                ]
                            })
                            .collect();
                        if rows.is_empty() {
                            println!("  No initiatives found matching filter.");
                        } else {
                            crate::output::table::print_table(
                                &[
                                    "Name",
                                    "Status",
                                    "Health",
                                    "Owner",
                                    "Target Date",
                                    "Projects",
                                ],
                                &rows,
                            );
                        }
                    }
                    _ => println!("  No initiatives found."),
                }
            }
        }

        InitiativesCommand::Get { name_or_id } => {
            let id = resolve_initiative_id(&client, name_or_id).await?;
            let query = r#"
                query($id: String!) {
                    initiative(id: $id) {
                        id name description status targetDate
                        owner { displayName }
                        creator { displayName }
                        color icon health
                        startedAt completedAt
                        createdAt updatedAt
                        projects(first: 50) {
                            nodes { id name state lead { displayName } }
                        }
                    }
                }
            "#;
            let variables = json!({ "id": id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let init = result
                    .pointer("/data/initiative")
                    .ok_or_else(|| anyhow::anyhow!("Initiative not found: {name_or_id}"))?;

                crate::output::detail::print_section("Initiative");
                crate::output::detail::print_detail(
                    "Name",
                    init.get("name").and_then(|v| v.as_str()).unwrap_or("-"),
                    1,
                );
                crate::output::detail::print_detail(
                    "Status",
                    init.get("status").and_then(|v| v.as_str()).unwrap_or("-"),
                    1,
                );
                if let Some(health) = init.get("health").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail(
                        "Health",
                        &crate::output::detail::format_health(health),
                        1,
                    );
                }
                crate::output::detail::print_detail(
                    "Owner",
                    &crate::output::detail::format_user(init.get("owner")),
                    1,
                );
                crate::output::detail::print_detail(
                    "Creator",
                    &crate::output::detail::format_user(init.get("creator")),
                    1,
                );
                if let Some(desc) = init.get("description").and_then(|v| v.as_str()) {
                    if !desc.is_empty() {
                        crate::output::detail::print_detail("Description", desc, 1);
                    }
                }
                if let Some(color) = init.get("color").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Color", color, 1);
                }
                if let Some(icon) = init.get("icon").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Icon", icon, 1);
                }
                crate::output::detail::print_detail(
                    "Target Date",
                    init.get("targetDate")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-"),
                    1,
                );
                if let Some(started) = init.get("startedAt").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Started", started, 1);
                }
                if let Some(completed) = init.get("completedAt").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Completed", completed, 1);
                }
                crate::output::detail::print_detail(
                    "Created",
                    init.get("createdAt")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-"),
                    1,
                );
                crate::output::detail::print_detail(
                    "Updated",
                    init.get("updatedAt")
                        .and_then(|v| v.as_str())
                        .unwrap_or("-"),
                    1,
                );
                crate::output::detail::print_detail(
                    "ID",
                    init.get("id").and_then(|v| v.as_str()).unwrap_or("-"),
                    1,
                );

                // Projects table
                let projects = init.pointer("/projects/nodes").and_then(|v| v.as_array());
                if let Some(projects) = projects {
                    if !projects.is_empty() {
                        crate::output::detail::print_section("Projects");
                        let rows: Vec<Vec<String>> = projects
                            .iter()
                            .map(|p| {
                                vec![
                                    p.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    p.get("state")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    crate::output::detail::format_user(p.get("lead")),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(&["Name", "Status", "Lead"], &rows);
                    }
                }
            }
        }

        InitiativesCommand::Create {
            name,
            description,
            owner,
            status,
            target_date,
            content,
        } => {
            let query = r#"
                mutation($input: InitiativeCreateInput!) {
                    initiativeCreate(input: $input) {
                        success
                        initiative { id name status }
                    }
                }
            "#;

            let mut input = json!({ "name": name });

            if let Some(desc) = description {
                input["description"] = json!(desc);
            }
            if let Some(owner_name) = owner {
                let user_id = client.get_user_id(owner_name).await?;
                input["ownerId"] = json!(user_id);
            }
            if let Some(s) = status {
                input["status"] = json!(s);
            }
            if let Some(date) = target_date {
                input["targetDate"] = json!(date);
            }
            if let Some(c) = content {
                input["content"] = json!(c);
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/initiativeCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    let init_name = result
                        .pointer("/data/initiativeCreate/initiative/name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(name);
                    println!(
                        "  {} Created initiative {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(init_name),
                    );
                } else {
                    println!(
                        "  {} Failed to create initiative",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        InitiativesCommand::Update {
            name_or_id,
            name,
            description,
            owner,
            status,
            target_date,
            content,
        } => {
            let id = resolve_initiative_id(&client, name_or_id).await?;
            let query = r#"
                mutation($id: String!, $input: InitiativeUpdateInput!) {
                    initiativeUpdate(id: $id, input: $input) {
                        success
                        initiative { id name status }
                    }
                }
            "#;

            let mut input = json!({});

            if let Some(n) = name {
                input["name"] = json!(n);
            }
            if let Some(desc) = description {
                input["description"] = json!(desc);
            }
            if let Some(owner_name) = owner {
                let user_id = client.get_user_id(owner_name).await?;
                input["ownerId"] = json!(user_id);
            }
            if let Some(s) = status {
                input["status"] = json!(s);
            }
            if let Some(date) = target_date {
                input["targetDate"] = json!(date);
            }
            if let Some(c) = content {
                input["content"] = json!(c);
            }

            let variables = json!({ "id": id, "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/initiativeUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Updated initiative {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name_or_id),
                    );
                } else {
                    println!(
                        "  {} Failed to update initiative",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        InitiativesCommand::Archive { name_or_id } => {
            let id = resolve_initiative_id(&client, name_or_id).await?;

            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!(
                    "Archive initiative {}?",
                    name_or_id
                ))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let query = r#"
                mutation($id: String!) {
                    initiativeArchive(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/initiativeArchive/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Archived initiative {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name_or_id),
                    );
                } else {
                    println!(
                        "  {} Failed to archive initiative",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        InitiativesCommand::Delete { name_or_id } => {
            let id = resolve_initiative_id(&client, name_or_id).await?;

            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!(
                    "Delete initiative {}? This cannot be undone.",
                    name_or_id
                ))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let query = r#"
                mutation($id: String!) {
                    initiativeDelete(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/initiativeDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Deleted initiative {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name_or_id),
                    );
                } else {
                    println!(
                        "  {} Failed to delete initiative",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        InitiativesCommand::Projects { name_or_id, limit } => {
            let id = resolve_initiative_id(&client, name_or_id).await?;
            let query = r#"
                query($id: String!, $first: Int!) {
                    initiative(id: $id) {
                        projects(first: $first) {
                            nodes {
                                id name state
                                lead { displayName }
                                startDate targetDate
                            }
                        }
                    }
                }
            "#;
            let variables = json!({ "id": id, "first": limit });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/initiative/projects/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(projects) if !projects.is_empty() => {
                        let rows: Vec<Vec<String>> = projects
                            .iter()
                            .map(|p| {
                                vec![
                                    p.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    p.get("state")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    crate::output::detail::format_user(p.get("lead")),
                                    p.get("startDate")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    p.get("targetDate")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(
                            &["Name", "Status", "Lead", "Start", "Target"],
                            &rows,
                        );
                    }
                    _ => println!("  No projects found for this initiative."),
                }
            }
        }
    }

    Ok(())
}
