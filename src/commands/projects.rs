use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct ProjectsArgs {
    #[command(subcommand)]
    pub command: ProjectsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ProjectsCommand {
    /// Get project details
    Get {
        /// Project name
        name: String,
    },
    /// List projects
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by team key
        #[arg(long)]
        team: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// List issues in a project
    Issues {
        /// Project name
        name: String,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// Create a project
    Create {
        /// Project name
        #[arg(long)]
        name: String,
        /// Team key or name
        #[arg(long)]
        team: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Lead (user name)
        #[arg(long)]
        lead: Option<String>,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start_date: Option<String>,
        /// Target date (YYYY-MM-DD)
        #[arg(long)]
        target_date: Option<String>,
    },
    /// Update a project
    Update {
        /// Project name
        name: String,
        /// New status
        #[arg(long)]
        status: Option<String>,
        /// New lead
        #[arg(long)]
        lead: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New start date
        #[arg(long)]
        start_date: Option<String>,
        /// New target date
        #[arg(long)]
        target_date: Option<String>,
    },
    /// Search projects by name or keyword
    Search {
        /// Search query
        query: String,
        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,
    },
    /// Archive a project
    Archive {
        /// Project name
        name: String,
    },
    /// Unarchive a project
    Unarchive {
        /// Project name
        name: String,
    },
    /// Delete a project
    Delete {
        /// Project name
        name: String,
    },
}

pub async fn execute(args: &ProjectsArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    match &args.command {
        ProjectsCommand::Get { name } => {
            let project_id = client.get_project_id(name).await?;
            let query = r#"
                query($id: String!) {
                    project(id: $id) {
                        id name description state
                        lead { id displayName }
                        startDate targetDate
                        teams { nodes { id key name } }
                        members { nodes { id displayName } }
                        issues { nodes { id } }
                        projectUpdates(first: 5) {
                            nodes {
                                id body health createdAt
                                user { displayName }
                            }
                        }
                        createdAt updatedAt
                    }
                }
            "#;
            let variables = json!({ "id": project_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let project = result
                    .pointer("/data/project")
                    .ok_or_else(|| anyhow::anyhow!("Project not found: {name}"))?;

                let pname = project.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                println!("\n  {}", crate::output::color::bold(pname));
                println!();

                if let Some(desc) = project.get("description").and_then(|v| v.as_str())
                    && !desc.is_empty()
                {
                    crate::output::detail::print_detail("Description", desc, 0);
                }
                if let Some(state) = project.get("state").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Status", state, 0);
                }
                let lead = crate::output::detail::format_user(project.get("lead"));
                crate::output::detail::print_detail("Lead", &lead, 0);

                if let Some(start) = project.get("startDate").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Start Date", start, 0);
                }
                if let Some(target) = project.get("targetDate").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Target Date", target, 0);
                }

                if let Some(teams) = project.pointer("/teams/nodes").and_then(|v| v.as_array()) {
                    let names: Vec<&str> = teams
                        .iter()
                        .filter_map(|t| t.get("key").and_then(|v| v.as_str()))
                        .collect();
                    if !names.is_empty() {
                        crate::output::detail::print_detail("Teams", &names.join(", "), 0);
                    }
                }

                if let Some(members) = project.pointer("/members/nodes").and_then(|v| v.as_array())
                {
                    crate::output::detail::print_detail(
                        "Members",
                        &format!("{}", members.len()),
                        0,
                    );
                }

                if let Some(issues) = project.pointer("/issues/nodes").and_then(|v| v.as_array()) {
                    crate::output::detail::print_detail("Issues", &format!("{}", issues.len()), 0);
                }

                if let Some(updates) = project
                    .pointer("/projectUpdates/nodes")
                    .and_then(|v| v.as_array())
                    && !updates.is_empty()
                {
                    crate::output::detail::print_section("Recent Updates");
                    for update in updates.iter().take(3) {
                        let health = update.get("health").and_then(|v| v.as_str()).unwrap_or("?");
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
                            "    {} {} {}",
                            crate::output::detail::format_health(health),
                            crate::output::color::bold(user),
                            crate::output::color::dim(date),
                        );
                        for line in body.lines().take(3) {
                            println!("      {line}");
                        }
                        println!();
                    }
                }
            }
        }

        ProjectsCommand::List {
            status,
            team,
            limit,
        } => {
            let query = r#"
                query($filter: ProjectFilter, $first: Int!) {
                    projects(filter: $filter, first: $first) {
                        nodes {
                            id name state
                            lead { displayName }
                            startDate targetDate
                        }
                    }
                }
            "#;

            let mut filter = json!({});
            if let Some(s) = status {
                filter["state"] = json!({ "eq": s });
            }
            if let Some(team_key) = team {
                let team_id = client.get_team_id(team_key).await?;
                filter["accessibleTeams"] = json!({ "id": { "eq": team_id } });
            }

            let variables = json!({ "filter": filter, "first": limit });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/projects/nodes")
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
                    _ => println!("  No projects found."),
                }
            }
        }

        ProjectsCommand::Issues {
            name,
            status,
            limit,
        } => {
            let project_id = client.get_project_id(name).await?;
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

            let mut filter = json!({ "project": { "id": { "eq": project_id } } });
            if let Some(status_name) = status {
                filter["state"] = json!({ "name": { "eqCaseInsensitive": status_name } });
            }

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
                    _ => println!("  No issues found."),
                }
            }
        }

        ProjectsCommand::Create {
            name,
            team,
            description,
            lead,
            start_date,
            target_date,
        } => {
            let query = r#"
                mutation($input: ProjectCreateInput!) {
                    projectCreate(input: $input) {
                        success
                        project { id name state }
                    }
                }
            "#;

            let team_id = client.get_team_id(team).await?;
            let mut input = json!({
                "name": name,
                "teamIds": [team_id],
            });

            if let Some(desc) = description {
                input["description"] = json!(desc);
            }
            if let Some(lead_name) = lead {
                let user_id = client.get_user_id(lead_name).await?;
                input["leadId"] = json!(user_id);
            }
            if let Some(start) = start_date {
                input["startDate"] = json!(start);
            }
            if let Some(target) = target_date {
                input["targetDate"] = json!(target);
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/projectCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Created project {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to create project",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        ProjectsCommand::Update {
            name,
            status,
            lead,
            description,
            start_date,
            target_date,
        } => {
            let project_id = client.get_project_id(name).await?;
            let query = r#"
                mutation($id: String!, $input: ProjectUpdateInput!) {
                    projectUpdate(id: $id, input: $input) {
                        success
                        project { id name state }
                    }
                }
            "#;

            let mut input = json!({});
            if let Some(s) = status {
                input["state"] = json!(s);
            }
            if let Some(lead_name) = lead {
                let user_id = client.get_user_id(lead_name).await?;
                input["leadId"] = json!(user_id);
            }
            if let Some(desc) = description {
                input["description"] = json!(desc);
            }
            if let Some(start) = start_date {
                input["startDate"] = json!(start);
            }
            if let Some(target) = target_date {
                input["targetDate"] = json!(target);
            }

            let variables = json!({ "id": project_id, "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/projectUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Updated project {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to update project",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        ProjectsCommand::Search { query: term, limit } => {
            let query = r#"
                query($term: String!, $first: Int) {
                    searchProjects(term: $term, first: $first) {
                        nodes { id name state lead { displayName } startDate targetDate }
                    }
                }
            "#;
            let variables = json!({ "term": term, "first": limit });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/searchProjects/nodes")
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
                    _ => println!("  No projects found."),
                }
            }
        }

        ProjectsCommand::Archive { name } => {
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!("Archive project {name}?"))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let project_id = client.get_project_id(name).await?;
            let query = r#"
                mutation($id: String!) {
                    projectArchive(id: $id) { success }
                }
            "#;
            let variables = json!({ "id": project_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/projectArchive/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Archived project {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to archive project",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        ProjectsCommand::Unarchive { name } => {
            let project_id = client.get_project_id(name).await?;
            let query = r#"
                mutation($id: String!) {
                    projectUnarchive(id: $id) { success }
                }
            "#;
            let variables = json!({ "id": project_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/projectUnarchive/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Unarchived project {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to unarchive project",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        ProjectsCommand::Delete { name } => {
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!("Delete project {name}?"))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let project_id = client.get_project_id(name).await?;
            let query = r#"
                mutation($id: String!) {
                    projectDelete(id: $id) { success }
                }
            "#;
            let variables = json!({ "id": project_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/projectDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Deleted project {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to delete project",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }
    }

    Ok(())
}
