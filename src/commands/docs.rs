use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: DocsCommand,
}

#[derive(Subcommand, Debug)]
pub enum DocsCommand {
    /// List documents
    List {
        /// Filter by project name
        #[arg(long)]
        project: Option<String>,
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// Get a document with full content
    Get {
        /// Document ID
        doc_id: String,
    },
    /// Search documents
    Search {
        /// Search query
        query: String,
        /// Max results
        #[arg(long, default_value = "25")]
        limit: i32,
    },
    /// Create a document
    Create {
        /// Document title
        #[arg(long)]
        title: String,
        /// Document content (markdown)
        #[arg(long)]
        content: Option<String>,
        /// Project name
        #[arg(long)]
        project: Option<String>,
        /// Icon
        #[arg(long)]
        icon: Option<String>,
    },
    /// Update a document
    Update {
        /// Document ID
        doc_id: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New content
        #[arg(long)]
        content: Option<String>,
        /// Project name
        #[arg(long)]
        project: Option<String>,
        /// Icon
        #[arg(long)]
        icon: Option<String>,
    },
    /// Delete a document
    Delete {
        /// Document ID
        doc_id: String,
    },
}

pub async fn execute(args: &DocsArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    match &args.command {
        DocsCommand::List { project, limit } => {
            let query = r#"
                query($first: Int!) {
                    documents(first: $first) {
                        nodes {
                            id title icon
                            project { name }
                            creator { displayName }
                            updatedAt
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
                    .pointer("/data/documents/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(docs) if !docs.is_empty() => {
                        let filtered: Vec<&serde_json::Value> = if let Some(proj) = project {
                            docs.iter()
                                .filter(|d| {
                                    d.pointer("/project/name")
                                        .and_then(|v| v.as_str())
                                        .map(|n| n.eq_ignore_ascii_case(proj))
                                        .unwrap_or(false)
                                })
                                .collect()
                        } else {
                            docs.iter().collect()
                        };

                        if filtered.is_empty() {
                            println!("  No documents found.");
                        } else {
                            let rows: Vec<Vec<String>> = filtered
                                .iter()
                                .map(|d| {
                                    vec![
                                        d.get("id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        d.get("title")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        d.pointer("/project/name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        d.pointer("/creator/displayName")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        d.get("updatedAt")
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.chars().take(10).collect())
                                            .unwrap_or_else(|| "-".to_string()),
                                    ]
                                })
                                .collect();
                            crate::output::table::print_table(
                                &["ID", "Title", "Project", "Creator", "Updated"],
                                &rows,
                            );
                        }
                    }
                    _ => println!("  No documents found."),
                }
            }
        }

        DocsCommand::Get { doc_id } => {
            let query = r#"
                query($id: String!) {
                    document(id: $id) {
                        id title content icon
                        project { name }
                        creator { displayName }
                        createdAt updatedAt
                    }
                }
            "#;
            let variables = json!({ "id": doc_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let doc = result
                    .pointer("/data/document")
                    .ok_or_else(|| anyhow::anyhow!("Document not found: {doc_id}"))?;

                let title = doc.get("title").and_then(|v| v.as_str()).unwrap_or("?");
                println!("\n  {}", crate::output::color::bold(title));
                println!();

                if let Some(proj) = doc.pointer("/project/name").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail("Project", proj, 0);
                }
                if let Some(creator) = doc.pointer("/creator/displayName").and_then(|v| v.as_str())
                {
                    crate::output::detail::print_detail("Creator", creator, 0);
                }
                if let Some(created) = doc.get("createdAt").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail(
                        "Created",
                        &crate::output::color::dim(created),
                        0,
                    );
                }
                if let Some(updated) = doc.get("updatedAt").and_then(|v| v.as_str()) {
                    crate::output::detail::print_detail(
                        "Updated",
                        &crate::output::color::dim(updated),
                        0,
                    );
                }

                if let Some(content) = doc.get("content").and_then(|v| v.as_str()) {
                    if !content.is_empty() {
                        crate::output::detail::print_section("Content");
                        let skin = termimad::MadSkin::default();
                        let rendered = skin.term_text(content);
                        for line in rendered.to_string().lines() {
                            println!("  {line}");
                        }
                    }
                }
            }
        }

        DocsCommand::Search { query: term, limit } => {
            let gql = r#"
                query($term: String!, $first: Int!) {
                    searchDocuments(term: $term, first: $first) {
                        nodes {
                            id title
                            project { name }
                            creator { displayName }
                            updatedAt
                        }
                    }
                }
            "#;
            let variables = json!({ "term": term, "first": limit });
            let result = client.query_raw(gql, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/searchDocuments/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(docs) if !docs.is_empty() => {
                        let rows: Vec<Vec<String>> = docs
                            .iter()
                            .map(|d| {
                                vec![
                                    d.get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    d.get("title")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    d.pointer("/project/name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    d.get("updatedAt")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.chars().take(10).collect())
                                        .unwrap_or_else(|| "-".to_string()),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(
                            &["ID", "Title", "Project", "Updated"],
                            &rows,
                        );
                    }
                    _ => println!("  No documents found."),
                }
            }
        }

        DocsCommand::Create {
            title,
            content,
            project,
            icon,
        } => {
            let query = r#"
                mutation($input: DocumentCreateInput!) {
                    documentCreate(input: $input) {
                        success
                        document { id title }
                    }
                }
            "#;

            let mut input = json!({ "title": title });
            if let Some(c) = content {
                input["content"] = json!(c);
            }
            if let Some(proj) = project {
                let project_id = client.get_project_id(proj).await?;
                input["projectId"] = json!(project_id);
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
                    .pointer("/data/documentCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    let doc_id = result
                        .pointer("/data/documentCreate/document/id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    println!(
                        "  {} Created document {} ({})",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(title),
                        crate::output::color::dim(doc_id),
                    );
                } else {
                    println!(
                        "  {} Failed to create document",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        DocsCommand::Update {
            doc_id,
            title,
            content,
            project,
            icon,
        } => {
            let query = r#"
                mutation($id: String!, $input: DocumentUpdateInput!) {
                    documentUpdate(id: $id, input: $input) {
                        success
                        document { id title }
                    }
                }
            "#;

            let mut input = json!({});
            if let Some(t) = title {
                input["title"] = json!(t);
            }
            if let Some(c) = content {
                input["content"] = json!(c);
            }
            if let Some(proj) = project {
                let project_id = client.get_project_id(proj).await?;
                input["projectId"] = json!(project_id);
            }
            if let Some(i) = icon {
                input["icon"] = json!(i);
            }

            let variables = json!({ "id": doc_id, "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/documentUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Updated document {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(doc_id),
                    );
                } else {
                    println!(
                        "  {} Failed to update document",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        DocsCommand::Delete { doc_id } => {
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!("Delete document {}?", doc_id))? {
                    println!("Cancelled.");
                    return Ok(());
                }

            let query = r#"
                mutation($id: String!) {
                    documentDelete(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": doc_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/documentDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Deleted document {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(doc_id),
                    );
                } else {
                    println!(
                        "  {} Failed to delete document",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }
    }

    Ok(())
}
