use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct ViewsArgs {
    #[command(subcommand)]
    pub command: ViewsCommand,
}

#[derive(Subcommand, Debug)]
pub enum ViewsCommand {
    /// List custom views
    List {
        /// Filter by team key
        #[arg(long)]
        team: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// Get view details
    Get {
        /// View ID
        view_id: String,
    },
    /// Create a custom view
    Create {
        /// View name
        #[arg(long)]
        name: String,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Team key
        #[arg(long)]
        team: Option<String>,
        /// Make shared (team/org visible)
        #[arg(long)]
        shared: bool,
        /// Make personal (only you)
        #[arg(long)]
        personal: bool,
        /// Filter JSON string
        #[arg(long)]
        filter: Option<String>,
        /// Icon name
        #[arg(long)]
        icon: Option<String>,
    },
    /// Update a custom view
    Update {
        /// View ID
        view_id: String,
        /// New name
        #[arg(long)]
        name: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// Make shared
        #[arg(long)]
        shared: bool,
        /// Make personal
        #[arg(long)]
        personal: bool,
        /// New filter JSON string
        #[arg(long)]
        filter: Option<String>,
        /// New icon
        #[arg(long)]
        icon: Option<String>,
    },
    /// Delete a custom view
    Delete {
        /// View ID
        view_id: String,
    },
    /// List issues matching a view's filters
    Issues {
        /// View ID
        view_id: String,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
}

/// Resolve a view identifier to a UUID. If it already looks like a UUID, return it as-is.
/// Otherwise, list all custom views and fuzzy-match by name (case-insensitive substring).
async fn resolve_view_id(client: &LinearClient, id_or_name: &str) -> anyhow::Result<String> {
    if crate::util::looks_like_uuid(id_or_name) {
        return Ok(id_or_name.to_string());
    }

    // Not a UUID — search by name
    let query = r#"
        query {
            customViews(first: 250) {
                nodes { id name }
            }
        }
    "#;
    let result = client.query_raw(query, None).await?;
    let nodes = result
        .pointer("/data/customViews/nodes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Could not fetch custom views"))?;

    let needle = id_or_name.to_lowercase();

    // Try exact match first, then substring
    let found = nodes
        .iter()
        .find(|v| {
            v.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.to_lowercase() == needle)
                .unwrap_or(false)
        })
        .or_else(|| {
            nodes.iter().find(|v| {
                v.get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n.to_lowercase().contains(&needle))
                    .unwrap_or(false)
            })
        });

    match found {
        Some(view) => view
            .get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| anyhow::anyhow!("View matched but has no ID")),
        None => Err(anyhow::anyhow!("View not found: {id_or_name}")),
    }
}

pub async fn execute(args: &ViewsArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    match &args.command {
        ViewsCommand::List { team, limit } => {
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

            let query = r#"
                query($first: Int!) {
                    customViews(first: $first) {
                        nodes {
                            id name description shared
                            team { key }
                            creator { displayName }
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
                    .pointer("/data/customViews/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(views) if !views.is_empty() => {
                        let filtered: Vec<&serde_json::Value> =
                            if let Some(team_key) = &resolved_team {
                                views
                                    .iter()
                                    .filter(|v| {
                                        v.pointer("/team/key")
                                            .and_then(|k| k.as_str())
                                            .map(|k| k.eq_ignore_ascii_case(team_key))
                                            .unwrap_or(false)
                                    })
                                    .collect()
                            } else {
                                views.iter().collect()
                            };

                        if filtered.is_empty() {
                            println!("  No views found.");
                        } else {
                            let rows: Vec<Vec<String>> = filtered
                                .iter()
                                .map(|v| {
                                    vec![
                                        v.get("id")
                                            .and_then(|x| x.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        v.get("name")
                                            .and_then(|x| x.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        if v.get("shared")
                                            .and_then(|x| x.as_bool())
                                            .unwrap_or(false)
                                        {
                                            "Shared".to_string()
                                        } else {
                                            "Personal".to_string()
                                        },
                                        v.pointer("/team/key")
                                            .and_then(|x| x.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        v.pointer("/creator/displayName")
                                            .and_then(|x| x.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                    ]
                                })
                                .collect();
                            crate::output::table::print_table(
                                &["ID", "Name", "Visibility", "Team", "Creator"],
                                &rows,
                            );
                        }
                    }
                    _ => println!("  No views found."),
                }
            }
        }

        ViewsCommand::Get { view_id } => {
            let view_id = resolve_view_id(&client, view_id).await?;
            let query = r#"
                query($id: String!) {
                    customView(id: $id) {
                        id name description shared
                        team { key name }
                        creator { displayName }
                        filterData
                        icon
                        createdAt updatedAt
                    }
                }
            "#;
            let variables = json!({ "id": view_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let view = result
                    .pointer("/data/customView")
                    .ok_or_else(|| anyhow::anyhow!("View not found: {view_id}"))?;

                let name = view.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                println!("\n  {}", crate::output::color::bold(name));
                println!();

                if let Some(desc) = view.get("description").and_then(|v| v.as_str())
                    && !desc.is_empty()
                {
                    crate::output::detail::print_detail("Description", desc, 0);
                }
                let vis = if view
                    .get("shared")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    "Shared"
                } else {
                    "Personal"
                };
                crate::output::detail::print_detail("Visibility", vis, 0);
                if let Some(team_name) = view.pointer("/team/name").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Team", team_name, 0);
                }
                if let Some(creator) = view
                    .pointer("/creator/displayName")
                    .and_then(|v| v.as_str())
                {
                    crate::output::detail::print_detail("Creator", creator, 0);
                }
                if let Some(icon) = view.get("icon").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Icon", icon, 0);
                }

                if let Some(filter) = view.get("filterData")
                    && !filter.is_null()
                {
                    crate::output::detail::print_section("Filter Data");
                    let pretty =
                        serde_json::to_string_pretty(filter).unwrap_or_else(|_| "?".into());
                    for line in pretty.lines() {
                        println!("    {line}");
                    }
                }
            }
        }

        ViewsCommand::Create {
            name,
            description,
            team,
            shared,
            personal: _,
            filter,
            icon,
        } => {
            let query = r#"
                mutation($input: CustomViewCreateInput!) {
                    customViewCreate(input: $input) {
                        success
                        customView { id name }
                    }
                }
            "#;

            let mut input = json!({ "name": name });
            if let Some(desc) = description {
                input["description"] = json!(desc);
            }
            if let Some(team_key) = team {
                let team_id = client.get_team_id(team_key).await?;
                input["teamId"] = json!(team_id);
            }
            input["shared"] = json!(*shared);
            if let Some(f) = filter {
                let filter_val: serde_json::Value =
                    serde_json::from_str(f).unwrap_or_else(|_| json!(f));
                input["filterData"] = filter_val;
            }
            if let Some(i) = icon {
                input["icon"] = json!(i);
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/customViewCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    let view_id = result
                        .pointer("/data/customViewCreate/customView/id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    println!(
                        "  {} Created view {} ({})",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                        crate::output::color::dim(view_id),
                    );
                } else {
                    println!(
                        "  {} Failed to create view",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        ViewsCommand::Update {
            view_id,
            name,
            description,
            shared,
            personal: _,
            filter,
            icon,
        } => {
            let view_id = resolve_view_id(&client, view_id).await?;
            let query = r#"
                mutation($id: String!, $input: CustomViewUpdateInput!) {
                    customViewUpdate(id: $id, input: $input) {
                        success
                        customView { id name }
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
            if *shared {
                input["shared"] = json!(true);
            }
            if let Some(f) = filter {
                let filter_val: serde_json::Value =
                    serde_json::from_str(f).unwrap_or_else(|_| json!(f));
                input["filterData"] = filter_val;
            }
            if let Some(i) = icon {
                input["icon"] = json!(i);
            }

            let variables = json!({ "id": view_id, "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/customViewUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Updated view {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(&view_id),
                    );
                } else {
                    println!(
                        "  {} Failed to update view",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        ViewsCommand::Delete { view_id } => {
            let view_id = resolve_view_id(&client, view_id).await?;
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!("Delete view {}?", view_id))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let query = r#"
                mutation($id: String!) {
                    customViewDelete(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": view_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/customViewDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Deleted view {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(&view_id),
                    );
                } else {
                    println!(
                        "  {} Failed to delete view",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        ViewsCommand::Issues { view_id, limit } => {
            let view_id = resolve_view_id(&client, view_id).await?;
            // Get the view's filter data, then query issues with those filters
            let view_query = r#"
                query($id: String!) {
                    customView(id: $id) {
                        filterData
                    }
                }
            "#;
            let view_result = client
                .query_raw(view_query, Some(json!({ "id": view_id })))
                .await?;
            let filter_data = view_result
                .pointer("/data/customView/filterData")
                .cloned()
                .unwrap_or(json!({}));

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
            let variables = json!({ "filter": filter_data, "first": limit });
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
    }

    Ok(())
}
