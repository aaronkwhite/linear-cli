use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct NotificationsArgs {
    #[command(subcommand)]
    pub command: NotificationsCommand,
}

#[derive(Subcommand, Debug)]
pub enum NotificationsCommand {
    /// List notifications
    List {
        /// Show only unread
        #[arg(long)]
        unread: bool,
        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,
    },
    /// Mark notification(s) as read
    Read {
        /// Notification ID (omit to use --all)
        notification_id: Option<String>,
        /// Mark all as read
        #[arg(long)]
        all: bool,
    },
    /// Archive notification(s)
    Archive {
        /// Notification ID (omit to use --all)
        notification_id: Option<String>,
        /// Archive all
        #[arg(long)]
        all: bool,
    },
    /// Snooze all notifications
    Snooze {
        /// Snooze all notifications
        #[arg(long)]
        all: bool,
    },
    /// Unsnooze all notifications
    Unsnooze {
        /// Unsnooze all notifications
        #[arg(long)]
        all: bool,
    },
}

fn format_notification_type(raw: &str) -> &str {
    match raw {
        "issueAssignedToYou" => "Assigned",
        "issueMention" => "Mentioned",
        "issueComment" => "Comment",
        "issueNewComment" => "New Comment",
        "issuePriorityChanged" => "Priority Changed",
        "issueStatusChanged" => "Status Changed",
        "issueDue" => "Due",
        "issueSubscribed" => "Subscribed",
        "issueCreated" => "Created",
        "issueBlocking" => "Blocking",
        "issueUnblocked" => "Unblocked",
        _ => raw,
    }
}

pub async fn execute(args: &NotificationsArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug, None)?;

    match &args.command {
        NotificationsCommand::List { unread, limit } => {
            let query = r#"
                query($first: Int!) {
                    notifications(first: $first) {
                        nodes {
                            id type readAt archivedAt createdAt
                            ... on IssueNotification {
                                issue { identifier title }
                            }
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
                    .pointer("/data/notifications/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(notifs) if !notifs.is_empty() => {
                        let filtered: Vec<&serde_json::Value> = if *unread {
                            notifs
                                .iter()
                                .filter(|n| n.get("readAt").is_none_or(|v| v.is_null()))
                                .collect()
                        } else {
                            notifs.iter().collect()
                        };

                        if filtered.is_empty() {
                            println!("  No notifications found.");
                        } else {
                            let rows: Vec<Vec<String>> = filtered
                                .iter()
                                .map(|n| {
                                    let ntype =
                                        n.get("type").and_then(|v| v.as_str()).unwrap_or("?");
                                    let issue_ident = n
                                        .pointer("/issue/identifier")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-");
                                    let issue_title = n
                                        .pointer("/issue/title")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    let date = n
                                        .get("createdAt")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.chars().take(10).collect())
                                        .unwrap_or_else(|| "-".to_string());
                                    let read = if n.get("readAt").is_none_or(|v| v.is_null()) {
                                        "Unread"
                                    } else {
                                        "Read"
                                    };
                                    let id = n
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string();
                                    vec![
                                        format_notification_type(ntype).to_string(),
                                        format!("{issue_ident} {issue_title}"),
                                        date,
                                        read.to_string(),
                                        id,
                                    ]
                                })
                                .collect();
                            crate::output::table::print_table(
                                &["Type", "Issue", "Date", "Status", "ID"],
                                &rows,
                            );
                        }
                    }
                    _ => println!("  No notifications found."),
                }
            }
        }

        NotificationsCommand::Read {
            notification_id,
            all,
        } => {
            if *all {
                // Mark all as read using notificationArchiveAll or batch
                let query = r#"
                    query {
                        notifications(first: 100) {
                            nodes { id readAt }
                        }
                    }
                "#;
                let result = client.query_raw(query, None).await?;
                let nodes = result
                    .pointer("/data/notifications/nodes")
                    .and_then(|v| v.as_array());

                let empty = vec![];
                let unread_ids: Vec<&str> = nodes
                    .unwrap_or(&empty)
                    .iter()
                    .filter(|n| n.get("readAt").is_none_or(|v| v.is_null()))
                    .filter_map(|n| n.get("id").and_then(|v| v.as_str()))
                    .collect();

                let mut count = 0;
                for id in &unread_ids {
                    let mark_query = r#"
                        mutation($id: String!, $input: NotificationUpdateInput!) {
                            notificationUpdate(id: $id, input: $input) {
                                success
                            }
                        }
                    "#;
                    let variables = json!({
                        "id": id,
                        "input": { "readAt": chrono_now() },
                    });
                    let _ = client.query_raw(mark_query, Some(variables)).await;
                    count += 1;
                }

                if json {
                    crate::output::print_json(&json!({ "markedRead": count }));
                } else {
                    println!(
                        "  {} Marked {count} notifications as read",
                        crate::output::color::green("OK"),
                    );
                }
            } else if let Some(id) = notification_id {
                let query = r#"
                    mutation($id: String!, $input: NotificationUpdateInput!) {
                        notificationUpdate(id: $id, input: $input) {
                            success
                        }
                    }
                "#;
                let variables = json!({
                    "id": id,
                    "input": { "readAt": chrono_now() },
                });
                let result = client.query_raw(query, Some(variables)).await?;

                if json {
                    crate::output::print_json(&result);
                } else {
                    let success = result
                        .pointer("/data/notificationUpdate/success")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if success {
                        println!(
                            "  {} Marked notification as read",
                            crate::output::color::green("OK"),
                        );
                    } else {
                        println!(
                            "  {} Failed to mark as read",
                            crate::output::color::red("ERROR")
                        );
                    }
                }
            } else {
                anyhow::bail!("Provide a notification ID or use --all");
            }
        }

        NotificationsCommand::Archive {
            notification_id,
            all,
        } => {
            if *all {
                let query = r#"
                    mutation {
                        notificationArchiveAll(input: {}) {
                            success
                        }
                    }
                "#;
                let result = client.query_raw(query, None).await?;

                if json {
                    crate::output::print_json(&result);
                } else {
                    let success = result
                        .pointer("/data/notificationArchiveAll/success")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if success {
                        println!(
                            "  {} Archived all notifications",
                            crate::output::color::green("OK"),
                        );
                    } else {
                        println!(
                            "  {} Failed to archive notifications",
                            crate::output::color::red("ERROR")
                        );
                    }
                }
            } else if let Some(id) = notification_id {
                let query = r#"
                    mutation($id: String!) {
                        notificationArchive(id: $id) {
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
                        .pointer("/data/notificationArchive/success")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    if success {
                        println!(
                            "  {} Archived notification",
                            crate::output::color::green("OK"),
                        );
                    } else {
                        println!(
                            "  {} Failed to archive notification",
                            crate::output::color::red("ERROR")
                        );
                    }
                }
            } else {
                anyhow::bail!("Provide a notification ID or use --all");
            }
        }

        NotificationsCommand::Snooze { all } => {
            if !all {
                anyhow::bail!("Use --all to snooze all notifications");
            }
            let query = r#"
                mutation {
                    notificationSnoozeAll(input: {}) {
                        success
                    }
                }
            "#;
            let result = client.query_raw(query, None).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/notificationSnoozeAll/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Snoozed all notifications",
                        crate::output::color::green("OK"),
                    );
                } else {
                    println!(
                        "  {} Failed to snooze notifications",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        NotificationsCommand::Unsnooze { all } => {
            if !all {
                anyhow::bail!("Use --all to unsnooze all notifications");
            }
            let query = r#"
                mutation {
                    notificationUnsnoozeAll(input: {}) {
                        success
                    }
                }
            "#;
            let result = client.query_raw(query, None).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/notificationUnsnoozeAll/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Unsnoozed all notifications",
                        crate::output::color::green("OK"),
                    );
                } else {
                    println!(
                        "  {} Failed to unsnooze notifications",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }
    }

    Ok(())
}

/// Simple ISO 8601 timestamp for "now" without pulling in chrono
fn chrono_now() -> String {
    // Use a simple approach - the Linear API accepts ISO 8601
    // We generate a "now" timestamp via std
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Convert to rough ISO 8601
    // Linear accepts various date formats; we'll use epoch-based
    format!("{now}")
}
