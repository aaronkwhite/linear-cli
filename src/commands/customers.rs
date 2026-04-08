use clap::{Args, Subcommand};
use serde_json::json;

use crate::client::LinearClient;

#[derive(Args, Debug)]
pub struct CustomersArgs {
    #[command(subcommand)]
    pub command: CustomersCommand,
}

#[derive(Subcommand, Debug)]
pub enum CustomersCommand {
    /// List customers
    List {
        /// Max results
        #[arg(long, default_value = "50")]
        limit: i32,
    },
    /// Create a customer
    Create {
        /// Customer name
        name: String,
        /// Domain (e.g., example.com)
        domain: String,
        /// Revenue amount
        #[arg(long)]
        revenue: Option<f64>,
        /// Tier name
        #[arg(long)]
        tier: Option<String>,
    },
    /// Update a customer
    Update {
        /// Customer name
        name: String,
        /// New tier
        #[arg(long)]
        tier: Option<String>,
        /// New revenue
        #[arg(long)]
        revenue: Option<f64>,
    },
    /// Delete a customer
    Delete {
        /// Customer name
        name: String,
    },
    /// Link a customer to an issue (create CustomerNeed)
    Link {
        /// Customer name
        customer_name: String,
        /// Issue identifier
        issue_id: String,
        /// Need body/description
        #[arg(long)]
        body: Option<String>,
    },
    /// List customer needs (auto-detects issue vs customer by pattern)
    Needs {
        /// Customer name or issue identifier (e.g., ENG-123)
        name_or_identifier: String,
    },
    /// List customer tiers
    Tiers,
    /// Create a customer tier
    CreateTier {
        /// Tier name
        name: String,
        /// Tier color
        #[arg(long)]
        color: Option<String>,
    },
}

/// Resolve a customer ID by name
async fn resolve_customer_id(client: &LinearClient, name: &str) -> anyhow::Result<String> {
    let query = r#"
        query($filter: CustomerFilter) {
            customers(filter: $filter, first: 1) {
                nodes { id name }
            }
        }
    "#;
    let variables = json!({ "filter": { "name": { "containsIgnoreCase": name } } });
    let result = client.query_raw(query, Some(variables)).await?;
    let id = result
        .pointer("/data/customers/nodes/0/id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Customer not found: {name}"))?;
    Ok(id.to_string())
}

pub async fn execute(args: &CustomersArgs, json: bool, debug: bool) -> anyhow::Result<()> {
    let client = LinearClient::new(None, debug)?;

    match &args.command {
        CustomersCommand::List { limit } => {
            let query = r#"
                query($first: Int!) {
                    customers(first: $first) {
                        nodes {
                            id name
                            domains
                            revenue
                            needs { id }
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
                    .pointer("/data/customers/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(customers) if !customers.is_empty() => {
                        let rows: Vec<Vec<String>> = customers
                            .iter()
                            .map(|c| {
                                let name = c
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("-")
                                    .to_string();
                                let domain = c
                                    .get("domains")
                                    .and_then(|v| v.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("-")
                                    .to_string();
                                let revenue = c
                                    .get("revenue")
                                    .and_then(|v| v.as_f64())
                                    .map(|r| format!("${:.0}", r))
                                    .unwrap_or_else(|| "-".to_string());
                                let needs = c
                                    .get("needs")
                                    .and_then(|v| v.as_array())
                                    .map(|a| format!("{}", a.len()))
                                    .unwrap_or_else(|| "0".to_string());
                                vec![name, domain, revenue, needs]
                            })
                            .collect();
                        crate::output::table::print_table(
                            &["Name", "Domain", "Revenue", "Needs"],
                            &rows,
                        );
                    }
                    _ => println!("  No customers found."),
                }
            }
        }

        CustomersCommand::Create {
            name,
            domain,
            revenue,
            tier,
        } => {
            let query = r#"
                mutation($input: CustomerCreateInput!) {
                    customerCreate(input: $input) {
                        success
                        customer { id name }
                    }
                }
            "#;

            let mut input = json!({
                "name": name,
                "domains": [domain],
            });
            if let Some(r) = revenue {
                input["revenue"] = json!(r);
            }
            if let Some(t) = tier {
                // Resolve tier ID
                let tier_query = r#"
                    query {
                        customerStatuses { nodes { id name } }
                    }
                "#;
                let tier_result = client.query_raw(tier_query, None).await?;
                let tier_id = tier_result
                    .pointer("/data/customerStatuses/nodes")
                    .and_then(|v| v.as_array())
                    .and_then(|tiers| {
                        tiers.iter().find(|s| {
                            s.get("name")
                                .and_then(|v| v.as_str())
                                .map(|n| n.eq_ignore_ascii_case(t))
                                .unwrap_or(false)
                        })
                    })
                    .and_then(|s| s.get("id").and_then(|v| v.as_str()));
                if let Some(tid) = tier_id {
                    input["customerStatusId"] = json!(tid);
                }
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/customerCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Created customer {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to create customer",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        CustomersCommand::Update {
            name,
            tier,
            revenue,
        } => {
            let customer_id = resolve_customer_id(&client, name).await?;
            let query = r#"
                mutation($id: String!, $input: CustomerUpdateInput!) {
                    customerUpdate(id: $id, input: $input) {
                        success
                        customer { id name }
                    }
                }
            "#;

            let mut input = json!({});
            if let Some(r) = revenue {
                input["revenue"] = json!(r);
            }
            if let Some(t) = tier {
                let tier_query = r#"
                    query {
                        customerStatuses { nodes { id name } }
                    }
                "#;
                let tier_result = client.query_raw(tier_query, None).await?;
                let tier_id = tier_result
                    .pointer("/data/customerStatuses/nodes")
                    .and_then(|v| v.as_array())
                    .and_then(|tiers| {
                        tiers.iter().find(|s| {
                            s.get("name")
                                .and_then(|v| v.as_str())
                                .map(|n| n.eq_ignore_ascii_case(t))
                                .unwrap_or(false)
                        })
                    })
                    .and_then(|s| s.get("id").and_then(|v| v.as_str()));
                if let Some(tid) = tier_id {
                    input["customerStatusId"] = json!(tid);
                }
            }

            let variables = json!({ "id": customer_id, "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/customerUpdate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Updated customer {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to update customer",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        CustomersCommand::Delete { name } => {
            if crate::output::interactive::is_interactive()
                && !crate::output::interactive::confirm(&format!("Delete customer {}?", name))?
            {
                println!("Cancelled.");
                return Ok(());
            }

            let customer_id = resolve_customer_id(&client, name).await?;
            let query = r#"
                mutation($id: String!) {
                    customerDelete(id: $id) {
                        success
                    }
                }
            "#;
            let variables = json!({ "id": customer_id });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/customerDelete/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Deleted customer {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to delete customer",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        CustomersCommand::Link {
            customer_name,
            issue_id,
            body,
        } => {
            let customer_id = resolve_customer_id(&client, customer_name).await?;
            let query = r#"
                mutation($input: CustomerNeedCreateInput!) {
                    customerNeedCreate(input: $input) {
                        success
                        customerNeed { id }
                    }
                }
            "#;
            let mut input = json!({
                "customerId": customer_id,
                "issueId": issue_id,
            });
            if let Some(b) = body {
                input["body"] = json!(b);
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/customerNeedCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Linked {} to {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(customer_name),
                        crate::output::color::bold(issue_id),
                    );
                } else {
                    println!(
                        "  {} Failed to link customer to issue",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }

        CustomersCommand::Needs { name_or_identifier } => {
            // Auto-detect: issue identifiers match pattern like ENG-123
            let is_issue = {
                let parts: Vec<&str> = name_or_identifier.splitn(2, '-').collect();
                parts.len() == 2
                    && parts[0].chars().all(|c| c.is_ascii_uppercase())
                    && !parts[0].is_empty()
                    && parts[1].chars().all(|c| c.is_ascii_digit())
                    && !parts[1].is_empty()
            };

            if is_issue {
                // Query needs for an issue
                let query = r#"
                    query($id: String!) {
                        issue(id: $id) {
                            identifier title
                            needs {
                                nodes {
                                    id body createdAt
                                    customer { name }
                                }
                            }
                        }
                    }
                "#;
                let variables = json!({ "id": name_or_identifier });
                let result = client.query_raw(query, Some(variables)).await?;

                if json {
                    crate::output::print_json(&result);
                } else {
                    let nodes = result
                        .pointer("/data/issue/needs/nodes")
                        .and_then(|v| v.as_array());
                    match nodes {
                        Some(needs) if !needs.is_empty() => {
                            let rows: Vec<Vec<String>> = needs
                                .iter()
                                .map(|n| {
                                    vec![
                                        n.pointer("/customer/name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        n.get("body")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        n.get("createdAt")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                    ]
                                })
                                .collect();
                            crate::output::table::print_table(
                                &["Customer", "Need", "Created"],
                                &rows,
                            );
                        }
                        _ => println!("  No customer needs found for {name_or_identifier}."),
                    }
                }
            } else {
                // Query needs for a customer
                let customer_id = resolve_customer_id(&client, name_or_identifier).await?;
                let query = r#"
                    query($id: String!) {
                        customer(id: $id) {
                            name
                            needs {
                                id body createdAt
                                issue { identifier title }
                            }
                        }
                    }
                "#;
                let variables = json!({ "id": customer_id });
                let result = client.query_raw(query, Some(variables)).await?;

                if json {
                    crate::output::print_json(&result);
                } else {
                    let nodes = result
                        .pointer("/data/customer/needs")
                        .and_then(|v| v.as_array());
                    match nodes {
                        Some(needs) if !needs.is_empty() => {
                            let rows: Vec<Vec<String>> = needs
                                .iter()
                                .map(|n| {
                                    let issue_ident = n
                                        .pointer("/issue/identifier")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-");
                                    let issue_title = n
                                        .pointer("/issue/title")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    vec![
                                        format!("{issue_ident} {issue_title}"),
                                        n.get("body")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                        n.get("createdAt")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("-")
                                            .to_string(),
                                    ]
                                })
                                .collect();
                            crate::output::table::print_table(&["Issue", "Need", "Created"], &rows);
                        }
                        _ => println!("  No needs found for {name_or_identifier}."),
                    }
                }
            }
        }

        CustomersCommand::Tiers => {
            let query = r#"
                query {
                    customerStatuses {
                        nodes { id name color description }
                    }
                }
            "#;
            let result = client.query_raw(query, None).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let nodes = result
                    .pointer("/data/customerStatuses/nodes")
                    .and_then(|v| v.as_array());
                match nodes {
                    Some(tiers) if !tiers.is_empty() => {
                        let rows: Vec<Vec<String>> = tiers
                            .iter()
                            .map(|t| {
                                vec![
                                    t.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    t.get("color")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                    t.get("description")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("-")
                                        .to_string(),
                                ]
                            })
                            .collect();
                        crate::output::table::print_table(&["Name", "Color", "Description"], &rows);
                    }
                    _ => println!("  No customer tiers found."),
                }
            }
        }

        CustomersCommand::CreateTier { name, color } => {
            let query = r#"
                mutation($input: CustomerStatusCreateInput!) {
                    customerStatusCreate(input: $input) {
                        success
                        customerStatus { id name }
                    }
                }
            "#;
            let mut input = json!({ "name": name });
            if let Some(c) = color {
                input["color"] = json!(c);
            }

            let variables = json!({ "input": input });
            let result = client.query_raw(query, Some(variables)).await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let success = result
                    .pointer("/data/customerStatusCreate/success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if success {
                    println!(
                        "  {} Created tier {}",
                        crate::output::color::green("OK"),
                        crate::output::color::bold(name),
                    );
                } else {
                    println!(
                        "  {} Failed to create tier",
                        crate::output::color::red("ERROR")
                    );
                }
            }
        }
    }

    Ok(())
}
