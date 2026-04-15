use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct RelationsArgs {
    #[command(subcommand)]
    pub command: RelationsCommand,
}

#[derive(Subcommand, Debug)]
pub enum RelationsCommand {
    /// List all relations for an issue
    List {
        /// Issue identifier (e.g., ENG-123)
        identifier: String,
    },
    /// Create "A blocks B" relation
    Blocks {
        /// Blocking issue identifier
        issue_a: String,
        /// Blocked issue identifier
        issue_b: String,
    },
    /// Create "A is blocked by B" relation
    BlockedBy {
        /// Blocked issue identifier
        issue_a: String,
        /// Blocking issue identifier
        issue_b: String,
    },
    /// Create "related" relation
    Relates {
        /// First issue identifier
        issue_a: String,
        /// Second issue identifier
        issue_b: String,
    },
    /// Create "duplicate" relation
    Duplicate {
        /// Duplicate issue identifier
        issue_a: String,
        /// Original issue identifier
        issue_b: String,
    },
    /// Remove a relation
    Remove {
        /// Relation ID
        relation_id: String,
    },
}

pub async fn execute(args: &RelationsArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug, None)?;

    match &args.command {
        RelationsCommand::List { identifier } => {
            let query = r#"
                query($id: String!) {
                    issue(id: $id) {
                        identifier title
                        relations {
                            nodes {
                                id type
                                relatedIssue {
                                    identifier title
                                    state { name }
                                }
                            }
                        }
                        inverseRelations {
                            nodes {
                                id type
                                issue {
                                    identifier title
                                    state { name }
                                }
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

                let ident = issue
                    .get("identifier")
                    .and_then(|v| v.as_str())
                    .unwrap_or(identifier);

                println!("\n  Relations for {}", crate::output::color::bold(ident));
                println!();

                let mut found = false;

                if let Some(rels) = issue.pointer("/relations/nodes").and_then(|v| v.as_array()) {
                    for rel in rels {
                        found = true;
                        let rel_type = rel.get("type").and_then(|v| v.as_str()).unwrap_or("?");
                        let rel_id = rel.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let related = rel
                            .pointer("/relatedIssue/identifier")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        let title = rel
                            .pointer("/relatedIssue/title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        println!(
                            "  {} {} {} {}",
                            crate::output::color::bold(related),
                            crate::output::color::cyan(rel_type),
                            title,
                            crate::output::color::dim(rel_id),
                        );
                    }
                }

                if let Some(inv) = issue
                    .pointer("/inverseRelations/nodes")
                    .and_then(|v| v.as_array())
                {
                    for rel in inv {
                        found = true;
                        let rel_type = rel.get("type").and_then(|v| v.as_str()).unwrap_or("?");
                        let rel_id = rel.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let related = rel
                            .pointer("/issue/identifier")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        let title = rel
                            .pointer("/issue/title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let inverse_label = match rel_type {
                            "blocks" => "blocked by",
                            "duplicate" => "duplicate of",
                            _ => rel_type,
                        };
                        println!(
                            "  {} {} {} {}",
                            crate::output::color::bold(related),
                            crate::output::color::cyan(inverse_label),
                            title,
                            crate::output::color::dim(rel_id),
                        );
                    }
                }

                if !found {
                    println!("  No relations found.");
                }
            }
        }

        RelationsCommand::Blocks { issue_a, issue_b } => {
            create_relation(&client, issue_a, issue_b, "blocks", json).await?;
        }

        RelationsCommand::BlockedBy { issue_a, issue_b } => {
            // "A blocked-by B" means B blocks A
            create_relation(&client, issue_b, issue_a, "blocks", json).await?;
        }

        RelationsCommand::Relates { issue_a, issue_b } => {
            create_relation(&client, issue_a, issue_b, "related", json).await?;
        }

        RelationsCommand::Duplicate { issue_a, issue_b } => {
            create_relation(&client, issue_a, issue_b, "duplicate", json).await?;
        }

        RelationsCommand::Remove { relation_id } => {
            let query = r#"
                mutation($id: String!) {
                    issueRelationDelete(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": relation_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/issueRelationDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Removed relation {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(relation_id),
                    );
                } else {
                    println!(
                        "  {} Failed to remove relation",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }
    }

    Ok(())
}

async fn create_relation(
    client: &LinearClient,
    issue_id: &str,
    related_issue_id: &str,
    rel_type: &str,
    json_mode: bool,
) -> anyhow::Result<()> {
    let query = r#"
        mutation($input: IssueRelationCreateInput!) {
            issueRelationCreate(input: $input) {
                success
                issueRelation { id type }
            }
        }
    "#;
    let variables = json!({
        "input": {
            "issueId": issue_id,
            "relatedIssueId": related_issue_id,
            "type": rel_type,
        }
    });
    let result = client.query_raw(query, Some(variables)).await?;

    if json_mode {
        crate::output::print_json(&result);
    } else {
        let success = result
            .pointer("/data/issueRelationCreate/success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if success {
            println!(
                "  {} Created {} relation: {} -> {}",
                crate::output::color::green("OK"),
                rel_type,
                crate::output::color::bold(issue_id),
                crate::output::color::bold(related_issue_id),
            );
        } else {
            println!(
                "  {} Failed to create relation",
                crate::output::color::red("ERROR")
            );
        }
    }
    Ok(())
}
