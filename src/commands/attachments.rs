use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct AttachmentsArgs {
    #[command(subcommand)]
    pub command: AttachmentsCommand,
}

#[derive(Subcommand, Debug)]
pub enum AttachmentsCommand {
    /// List attachments for an issue
    List {
        /// Issue identifier (e.g., ENG-123)
        issue_identifier: String,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// Create an attachment on an issue
    Create {
        /// Issue identifier (e.g., ENG-123)
        #[arg(long)]
        issue: String,
        /// Attachment title
        #[arg(long)]
        title: String,
        /// Attachment URL
        #[arg(long)]
        url: String,
        /// Attachment subtitle
        #[arg(long)]
        subtitle: Option<String>,
    },
    /// Link a URL to an issue as an attachment
    LinkUrl {
        /// Issue identifier (e.g., ENG-123)
        #[arg(long)]
        issue: String,
        /// URL to link
        #[arg(long)]
        url: String,
        /// Optional title for the link
        #[arg(long)]
        title: Option<String>,
    },
    /// Delete an attachment
    Delete {
        /// Attachment ID
        attachment_id: String,
    },
}

pub async fn execute(args: &AttachmentsArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug, None)?;

    match &args.command {
        AttachmentsCommand::List {
            issue_identifier,
            limit,
        } => {
            // Resolve the issue identifier (e.g., ENG-123) to an ID first,
            // then query attachments through the issue connection.
            let issue_id = if crate::util::looks_like_uuid(issue_identifier) {
                issue_identifier.clone()
            } else {
                // Resolve identifier like ENG-123 to UUID
                let resolve_query = r#"
                    query($id: String!) {
                        issue(id: $id) { id }
                    }
                "#;
                let resolve_result = client
                    .query_raw(resolve_query, Some(json!({ "id": issue_identifier })))
                    .await?;
                resolve_result
                    .pointer("/data/issue/id")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .ok_or_else(|| anyhow::anyhow!("Issue not found: {issue_identifier}"))?
            };

            let query = r#"
                query($id: String!, $first: Int!) {
                    issue(id: $id) {
                        attachments(first: $first) {
                            nodes { id title subtitle url sourceType createdAt }
                        }
                    }
                }
            "#;
            let variables = json!({
                "id": issue_id,
                "first": limit,
            });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/issue/attachments/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(attachments) if !attachments.is_empty() => {
                        let headers = &["Title", "Type", "URL", "Created"];
                        let rows: Vec<Vec<String>> = attachments
                            .iter()
                            .map(|a| {
                                vec![
                                    a.get("title")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    a.get("sourceType")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    a.get("url")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    a.get("createdAt")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(headers, &rows);
                    }
                    _ => {
                        println!("  No attachments found for issue {issue_identifier}.");
                    }
                }
            }
        }

        AttachmentsCommand::Create {
            issue,
            title,
            url,
            subtitle,
        } => {
            let query = r#"
                mutation($input: AttachmentCreateInput!) {
                    attachmentCreate(input: $input) {
                        success
                        attachment { id title url }
                    }
                }
            "#;
            let mut input = json!({
                "issueId": issue,
                "title": title,
                "url": url,
            });
            if let Some(sub) = subtitle {
                input["subtitle"] = json!(sub);
            }
            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/attachmentCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    let id = result
                        .pointer("/data/attachmentCreate/attachment/id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("???");
                    println!(
                        "  {} Created attachment {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(id),
                    );
                } else {
                    println!(
                        "  {} Failed to create attachment",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        AttachmentsCommand::LinkUrl { issue, url, title } => {
            let query = r#"
                mutation($issueId: String!, $url: String!, $title: String) {
                    attachmentLinkURL(issueId: $issueId, url: $url, title: $title) {
                        success
                        attachment { id title url }
                    }
                }
            "#;
            let variables = json!({
                "issueId": issue,
                "url": url,
                "title": title,
            });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/attachmentLinkURL/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    let id = result
                        .pointer("/data/attachmentLinkURL/attachment/id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("???");
                    println!(
                        "  {} Linked URL as attachment {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(id),
                    );
                } else {
                    println!(
                        "  {} Failed to link URL",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        AttachmentsCommand::Delete { attachment_id } => {
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!(
                    "Delete attachment {}?",
                    attachment_id
                ))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let query = r#"
                mutation($id: String!) {
                    attachmentDelete(id: $id) { success }
                }
            "#;
            let variables = json!({ "id": attachment_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/attachmentDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Deleted attachment {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(attachment_id),
                    );
                } else {
                    println!(
                        "  {} Failed to delete attachment",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }
    }

    Ok(())
}
