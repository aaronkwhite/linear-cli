use crate::client::LinearClient;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct ApiArgs {
    /// GraphQL query or mutation string
    pub query: String,
    /// JSON variables object (e.g. '{"teamId": "abc123"}')
    #[arg(long)]
    pub variables: Option<String>,
}

pub async fn execute(args: &ApiArgs, _json: bool, debug: bool) -> anyhow::Result<()> {
    let variables = if let Some(vars_str) = &args.variables {
        let v: serde_json::Value = serde_json::from_str(vars_str)
            .map_err(|e| anyhow::anyhow!("Invalid JSON for --variables: {e}"))?;
        Some(v)
    } else {
        None
    };

    let client = LinearClient::new(None, debug)?;
    let result = client.query_raw(&args.query, variables).await?;
    crate::output::print_json(&result);
    Ok(())
}
