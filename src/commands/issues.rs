use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct IssuesArgs {
    #[command(subcommand)]
    pub command: IssuesCommand,
}

#[derive(Subcommand, Debug)]
pub enum IssuesCommand {
    /// Get issue details
    Get {
        /// Issue identifier (e.g., ENG-123)
        identifier: String,
    },
    /// List issues
    List {
        /// Filter by team key or name
        #[arg(long)]
        team: Option<String>,
        /// Query across all teams (conflicts with --team)
        #[arg(long, conflicts_with = "team")]
        all_teams: bool,
        /// Filter by workflow state name
        #[arg(long, visible_alias = "status")]
        state: Option<String>,
        /// Filter by assignee name
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by priority (1=Urgent, 2=High, 3=Medium, 4=Low)
        #[arg(long)]
        priority: Option<i32>,
        /// Filter by label name (can be specified multiple times)
        #[arg(long)]
        label: Vec<String>,
        /// Only issues created on or after this date (YYYY-MM-DD)
        #[arg(long)]
        created_after: Option<String>,
        /// Only issues updated on or after this date (YYYY-MM-DD)
        #[arg(long)]
        updated_after: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// Search issues by text
    Search {
        /// Search query
        query: String,
        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,
    },
    /// Create a new issue
    Create {
        /// Team key or name (required)
        #[arg(long)]
        team: String,
        /// Issue title (required)
        #[arg(long)]
        title: String,
        /// Issue description
        #[arg(long)]
        description: Option<String>,
        /// Read description from a file (conflicts with --description)
        #[arg(long, conflicts_with = "description")]
        description_file: Option<String>,
        /// Assignee name
        #[arg(long)]
        assignee: Option<String>,
        /// Priority (1=Urgent, 2=High, 3=Medium, 4=Low)
        #[arg(long)]
        priority: Option<i32>,
        /// Story points estimate
        #[arg(long)]
        estimate: Option<f64>,
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due_date: Option<String>,
        /// Label name
        #[arg(long)]
        label: Option<String>,
        /// Parent issue identifier
        #[arg(long)]
        parent: Option<String>,
        /// Project name
        #[arg(long)]
        project: Option<String>,
        /// Initial status
        #[arg(long)]
        status: Option<String>,
    },
    /// Update an issue
    Update {
        /// Issue identifier
        identifier: String,
        /// New status
        #[arg(long)]
        status: Option<String>,
        /// New assignee
        #[arg(long)]
        assignee: Option<String>,
        /// New priority
        #[arg(long)]
        priority: Option<i32>,
        /// New estimate
        #[arg(long)]
        estimate: Option<f64>,
        /// New due date
        #[arg(long)]
        due_date: Option<String>,
        /// Parent issue identifier
        #[arg(long)]
        parent: Option<String>,
        /// Project name
        #[arg(long)]
        project: Option<String>,
        /// Label name
        #[arg(long)]
        label: Option<String>,
        /// Milestone name
        #[arg(long)]
        milestone: Option<String>,
        /// Move issue to a different team (key or name)
        #[arg(long)]
        team: Option<String>,
        /// Read description from a file
        #[arg(long)]
        description_file: Option<String>,
    },
    /// Add a comment to an issue
    Comment {
        /// Issue identifier
        identifier: String,
        /// Comment body (omit if using --body-file)
        body: Option<String>,
        /// Read comment body from a file
        #[arg(long)]
        body_file: Option<String>,
    },
    /// Archive an issue
    Archive {
        /// Issue identifier
        identifier: String,
    },
    /// Find issue associated with a git branch name
    Branch {
        /// Git branch name
        branch_name: String,
    },
    /// Delete an issue
    Delete {
        /// Issue identifier
        identifier: String,
    },
    /// Unarchive an issue
    Unarchive {
        /// Issue identifier
        identifier: String,
    },
    /// Subscribe to an issue
    Subscribe {
        /// Issue identifier
        identifier: String,
    },
    /// Unsubscribe from an issue
    Unsubscribe {
        /// Issue identifier
        identifier: String,
    },
    /// Start working on an issue: create/switch to its git branch
    Start {
        /// Issue identifier (e.g., ENG-123)
        identifier: String,
        /// Update issue status after branching
        #[arg(long)]
        status: Option<String>,
        /// Just print the branch name, don't run git commands
        #[arg(long)]
        print_only: bool,
    },
    /// Create a GitHub PR linked to an issue (requires gh CLI)
    Pr {
        /// Issue identifier (e.g., ENG-123)
        identifier: String,
        /// Create as draft PR
        #[arg(long)]
        draft: bool,
        /// Base branch for the PR
        #[arg(long)]
        base: Option<String>,
    },
}

pub async fn execute(args: &IssuesArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug, None)?;

    match &args.command {
        IssuesCommand::Get { identifier } => {
            let query = r#"
                query($id: String!) {
                    issue(id: $id) {
                        id identifier title description
                        state { id name }
                        priority
                        assignee { id displayName }
                        team { id key name }
                        project { id name }
                        estimate
                        dueDate
                        createdAt
                        updatedAt
                        labels { nodes { id name } }
                        parent { id identifier title }
                        comments(first: 10) {
                            nodes {
                                id body createdAt
                                user { displayName }
                            }
                        }
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let issue = result
                    .pointer("/data/issue")
                    .ok_or_else(|| anyhow::anyhow!("Issue not found: {identifier}"))?;
                crate::output::detail::print_issue_detail(issue);
            }
        }

        IssuesCommand::List {
            team,
            all_teams,
            state,
            assignee,
            priority,
            label,
            created_after,
            updated_after,
            limit,
        } => {
            let query = r#"
                query($filter: IssueFilter, $first: Int!, $after: String) {
                    issues(filter: $filter, first: $first, after: $after, orderBy: updatedAt) {
                        nodes {
                            id identifier title
                            state { name }
                            assignee { displayName }
                            priority
                            team { key }
                            labels { nodes { name } }
                        }
                        pageInfo { hasNextPage endCursor }
                    }
                }
            "#;

            let mut filter = json!({});

            let team_key = if *all_teams {
                None
            } else {
                match team {
                    Some(t) => Some(t.clone()),
                    None if crate::output::interactive::is_interactive() => {
                        let teams = client.get_teams().await?;
                        let items: Vec<String> = teams
                            .iter()
                            .map(|t| format!("{} ({})", t.name, t.key))
                            .collect();
                        let idx =
                            crate::output::interactive::fuzzy_select("Select team", &items)?;
                        Some(teams[idx].key.clone())
                    }
                    None => None,
                }
            };

            if let Some(team_key) = &team_key {
                let team_id = client.get_team_id(team_key).await?;
                filter["team"] = json!({ "id": { "eq": team_id } });
            }
            if let Some(state_name) = state {
                filter["state"] = json!({ "name": { "containsIgnoreCase": state_name } });
            }
            if let Some(assignee_name) = assignee {
                let user_id = client.get_user_id(assignee_name).await?;
                filter["assignee"] = json!({ "id": { "eq": user_id } });
            }
            if let Some(p) = priority {
                filter["priority"] = json!({ "eq": p });
            }
            if !label.is_empty() {
                let label_refs: Vec<&str> = label.iter().map(|s| s.as_str()).collect();
                let label_ids = client
                    .get_label_ids(&label_refs, team_key.as_deref())
                    .await?;
                // Use "some" filter: match issues that have at least one of the specified labels
                let or_filters: Vec<serde_json::Value> = label_ids
                    .iter()
                    .map(|id| json!({ "id": { "eq": id } }))
                    .collect();
                filter["labels"] = json!({ "some": { "or": or_filters } });
            }
            if let Some(date) = created_after {
                filter["createdAt"] = json!({ "gte": date });
            }
            if let Some(date) = updated_after {
                filter["updatedAt"] = json!({ "gte": date });
            }

            let variables = json!({
                "filter": filter,
                "first": limit,
            });
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
                    _ => {
                        println!("  No issues found.");
                    }
                }
            }
        }

        IssuesCommand::Search { query: term, limit } => {
            let gql = r#"
                query($term: String!, $first: Int) {
                    searchIssues(term: $term, first: $first) {
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

            let variables = json!({
                "term": term,
                "first": limit,
            });
            let result = client.query_raw(gql, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/searchIssues/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(issues) if !issues.is_empty() => {
                        for issue in issues {
                            crate::output::detail::print_issue_summary(issue);
                        }
                    }
                    _ => {
                        println!("  No issues found.");
                    }
                }
            }
        }

        IssuesCommand::Create {
            team,
            title,
            description,
            description_file,
            assignee,
            priority,
            estimate,
            due_date,
            label,
            parent,
            project,
            status,
        } => {
            let query = r#"
                mutation($input: IssueCreateInput!) {
                    issueCreate(input: $input) {
                        success
                        issue {
                            id identifier title
                            state { name }
                            assignee { displayName }
                            team { key }
                        }
                    }
                }
            "#;

            let team_id = client.get_team_id(team).await?;
            let mut input = json!({
                "teamId": team_id,
                "title": title,
            });

            let desc = match (description, description_file) {
                (Some(d), _) => Some(d.clone()),
                (_, Some(path)) => Some(std::fs::read_to_string(path).map_err(|e| {
                    anyhow::anyhow!("Failed to read description file '{}': {}", path, e)
                })?),
                _ => None,
            };
            if let Some(desc) = desc {
                input["description"] = json!(desc);
            }
            if let Some(assignee_name) = assignee {
                let user_id = client.get_user_id(assignee_name).await?;
                input["assigneeId"] = json!(user_id);
            }
            if let Some(p) = priority {
                input["priority"] = json!(p);
            }
            if let Some(est) = estimate {
                input["estimate"] = json!(est);
            }
            if let Some(due) = due_date {
                input["dueDate"] = json!(due);
            }
            if let Some(label_name) = label {
                let label_ids = client
                    .get_label_ids(&[label_name.as_str()], Some(team))
                    .await?;
                input["labelIds"] = json!(label_ids);
            }
            if let Some(parent_id) = parent {
                input["parentId"] = json!(parent_id);
            }
            if let Some(project_name) = project {
                let project_id = client.get_project_id(project_name).await?;
                input["projectId"] = json!(project_id);
            }
            if let Some(status_name) = status {
                let state_id = client.get_state_id(team, status_name).await?;
                input["stateId"] = json!(state_id);
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    let issue = result.pointer("/data/issueCreate/issue");
                    let identifier = issue
                        .and_then(|i| i.get("identifier"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("???");
                    println!(
                        "  {} Created issue {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(identifier),
                    );
                } else {
                    println!(
                        "  {} Failed to create issue",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        IssuesCommand::Update {
            identifier,
            status,
            assignee,
            priority,
            estimate,
            due_date,
            parent,
            project,
            label,
            milestone: _,
            team,
            description_file,
        } => {
            let query = r#"
                mutation($id: String!, $input: IssueUpdateInput!) {
                    issueUpdate(id: $id, input: $input) {
                        success
                        issue {
                            id identifier title
                            state { name }
                            assignee { displayName }
                            team { key }
                        }
                    }
                }
            "#;

            let mut input = json!({});

            // For status resolution we need the issue's team
            if status.is_some() || label.is_some() {
                // Fetch the issue to get its team key
                let get_query = r#"
                    query($id: String!) {
                        issue(id: $id) {
                            team { key }
                        }
                    }
                "#;
                let get_result = client
                    .query_raw(get_query, Some(json!({ "id": identifier })))
                    .await?;
                let team_key = get_result
                    .pointer("/data/issue/team/key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Could not determine team for issue {identifier}")
                    })?;

                if let Some(status_name) = status {
                    let state_id = client.get_state_id(team_key, status_name).await?;
                    input["stateId"] = json!(state_id);
                }
                if let Some(label_name) = label {
                    let label_ids = client
                        .get_label_ids(&[label_name.as_str()], Some(team_key))
                        .await?;
                    input["labelIds"] = json!(label_ids);
                }
            }

            if let Some(assignee_name) = assignee {
                let user_id = client.get_user_id(assignee_name).await?;
                input["assigneeId"] = json!(user_id);
            }
            if let Some(p) = priority {
                input["priority"] = json!(p);
            }
            if let Some(est) = estimate {
                input["estimate"] = json!(est);
            }
            if let Some(due) = due_date {
                input["dueDate"] = json!(due);
            }
            if let Some(parent_id) = parent {
                input["parentId"] = json!(parent_id);
            }
            if let Some(project_name) = project {
                let project_id = client.get_project_id(project_name).await?;
                input["projectId"] = json!(project_id);
            }
            if let Some(team_name) = team {
                let team_id = client.get_team_id(team_name).await?;
                input["teamId"] = json!(team_id);
            }
            if let Some(path) = description_file {
                let content = std::fs::read_to_string(path).map_err(|e| {
                    anyhow::anyhow!("Failed to read description file '{}': {}", path, e)
                })?;
                input["description"] = json!(content);
            }

            let variables = json!({
                "id": identifier,
                "input": input,
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
                        "  {} Updated issue {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(identifier),
                    );
                } else {
                    println!(
                        "  {} Failed to update issue",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        IssuesCommand::Comment {
            identifier,
            body,
            body_file,
        } => {
            let comment_body = match (body, body_file) {
                (Some(b), _) => b.clone(),
                (_, Some(path)) => std::fs::read_to_string(path)
                    .map_err(|e| anyhow::anyhow!("Failed to read body file '{}': {}", path, e))?,
                (None, None) => anyhow::bail!("Provide a comment body or --body-file"),
            };

            let query = r#"
                mutation($input: CommentCreateInput!) {
                    commentCreate(input: $input) {
                        success
                        comment { id body }
                    }
                }
            "#;
            let variables = json!({
                "input": {
                    "issueId": identifier,
                    "body": comment_body,
                }
            });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/commentCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Added comment to {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(identifier),
                    );
                } else {
                    println!(
                        "  {} Failed to add comment",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        IssuesCommand::Archive { identifier } => {
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!("Archive issue {}?", identifier))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let query = r#"
                mutation($id: String!) {
                    issueArchive(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueArchive/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Archived issue {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(identifier),
                    );
                } else {
                    println!(
                        "  {} Failed to archive issue",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        IssuesCommand::Branch { branch_name } => {
            let query = r#"
                query($branchName: String!) {
                    issueVcsBranchSearch(branchName: $branchName) {
                        id identifier title
                        state { name }
                        assignee { displayName }
                        priority
                        team { key }
                    }
                }
            "#;
            let variables = json!({ "branchName": branch_name });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let issue = result.pointer("/data/issueVcsBranchSearch");
                match issue {
                    Some(i) if !i.is_null() => {
                        crate::output::detail::print_issue_summary(i);
                    }
                    _ => {
                        println!("  No issue found for branch '{branch_name}'.");
                    }
                }
            }
        }

        IssuesCommand::Delete { identifier } => {
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!("Delete issue {}?", identifier))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let query = r#"
                mutation($id: String!) {
                    issueDelete(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Deleted issue {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(identifier),
                    );
                } else {
                    println!(
                        "  {} Failed to delete issue",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        IssuesCommand::Unarchive { identifier } => {
            let query = r#"
                mutation($id: String!) {
                    issueUnarchive(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueUnarchive/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Unarchived issue {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(identifier),
                    );
                } else {
                    println!(
                        "  {} Failed to unarchive issue",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        IssuesCommand::Subscribe { identifier } => {
            let query = r#"
                mutation($id: String!) {
                    issueSubscribe(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueSubscribe/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Subscribed to issue {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(identifier),
                    );
                } else {
                    println!(
                        "  {} Failed to subscribe to issue",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        IssuesCommand::Unsubscribe { identifier } => {
            let query = r#"
                mutation($id: String!) {
                    issueUnsubscribe(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueUnsubscribe/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Unsubscribed from issue {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(identifier),
                    );
                } else {
                    println!(
                        "  {} Failed to unsubscribe from issue",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        IssuesCommand::Start {
            identifier,
            status,
            print_only,
        } => {
            let query = r#"
                query($id: String!) {
                    issue(id: $id) {
                        id identifier branchName
                        team { key }
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;
            let issue = result
                .pointer("/data/issue")
                .ok_or_else(|| anyhow::anyhow!("Issue not found: {identifier}"))?;
            let branch_name = issue
                .get("branchName")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("No branch name for issue {identifier}"))?;

            if *print_only {
                if json {
                    crate::output::print_json(&serde_json::json!({
                        "branch": branch_name,
                        "identifier": identifier,
                    }));
                } else {
                    println!("{branch_name}");
                }
                return Ok(());
            }

            // Check if we're in a git repo
            let git_check = std::process::Command::new("git")
                .args(["rev-parse", "--is-inside-work-tree"])
                .output();
            match git_check {
                Ok(output) if output.status.success() => {}
                _ => anyhow::bail!("Not a git repository"),
            }

            // Try to create and switch to the branch; if it exists, just switch
            let checkout = std::process::Command::new("git")
                .args(["checkout", "-b", branch_name])
                .output()?;
            if !checkout.status.success() {
                let switch = std::process::Command::new("git")
                    .args(["checkout", branch_name])
                    .output()?;
                if !switch.status.success() {
                    let stderr = String::from_utf8_lossy(&switch.stderr);
                    anyhow::bail!(
                        "Failed to checkout branch '{}': {}",
                        branch_name,
                        stderr.trim()
                    );
                }
            }

            // Optionally update status
            if let Some(status_name) = status {
                let team_key = issue
                    .get("team")
                    .and_then(|t| t.get("key"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Could not determine team for {identifier}")
                    })?;
                let state_id = client.get_state_id(team_key, status_name).await?;
                let update_query = r#"
                    mutation($id: String!, $input: IssueUpdateInput!) {
                        issueUpdate(id: $id, input: $input) { success }
                    }
                "#;
                client
                    .query_raw(
                        update_query,
                        Some(json!({ "id": identifier, "input": { "stateId": state_id } })),
                    )
                    .await?;
            }

            if json {
                crate::output::print_json(&serde_json::json!({
                    "branch": branch_name,
                    "identifier": identifier,
                }));
            } else {
                println!(
                    "  {} Switched to branch {}",
                    crate::output::color::green("OK"),
                    crate::output::color::bold(branch_name),
                );
            }
        }

        IssuesCommand::Pr {
            identifier,
            draft,
            base,
        } => {
            // Check gh is available
            let gh_check = std::process::Command::new("gh")
                .args(["--version"])
                .output();
            match gh_check {
                Ok(output) if output.status.success() => {}
                _ => anyhow::bail!("gh CLI required for pr command (https://cli.github.com)"),
            }

            let query = r#"
                query($id: String!) {
                    issue(id: $id) {
                        identifier title url
                    }
                }
            "#;
            let variables = json!({ "id": identifier });
            let result = client.query_raw(query, Some(variables)).await?;
            let issue = result
                .pointer("/data/issue")
                .ok_or_else(|| anyhow::anyhow!("Issue not found: {identifier}"))?;
            let ident = issue
                .get("identifier")
                .and_then(|v| v.as_str())
                .unwrap_or(identifier);
            let title = issue
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");
            let url = issue
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let pr_title = format!("{}: {}", ident, title);
            let pr_body = format!("Resolves {}", url);

            let mut gh_args = vec![
                "pr".to_string(),
                "create".to_string(),
                "--title".to_string(),
                pr_title.clone(),
                "--body".to_string(),
                pr_body,
            ];
            if *draft {
                gh_args.push("--draft".to_string());
            }
            if let Some(base_branch) = base {
                gh_args.push("--base".to_string());
                gh_args.push(base_branch.clone());
            }

            let arg_refs: Vec<&str> = gh_args.iter().map(|s| s.as_str()).collect();
            let output = std::process::Command::new("gh")
                .args(&arg_refs)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output()?;

            if !output.status.success() {
                anyhow::bail!("gh pr create failed");
            }

            if json {
                crate::output::print_json(&serde_json::json!({
                    "pr_title": pr_title,
                    "identifier": ident,
                }));
            }
        }
    }

    Ok(())
}
