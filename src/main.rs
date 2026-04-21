mod analytics;
mod cli;
mod client;
mod commands;
mod config;
mod error;
mod graphql;
mod output;
mod util;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let ws = cli.workspace.as_deref();

    // Completions are a local operation — skip analytics, return early
    if let Commands::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        clap_complete::generate(*shell, &mut cmd, "lin", &mut std::io::stdout());
        return;
    }

    let start = Instant::now();

    let result = match &cli.command {
        Commands::Api(args) => commands::api::execute(args, cli.json, cli.debug, ws).await,
        Commands::Auth(args) => commands::auth::execute(args, cli.json, cli.debug, ws).await,
        Commands::Issues(args) => commands::issues::execute(args, cli.json, cli.debug, ws).await,
        Commands::Projects(args) => {
            commands::projects::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Cycles(args) => commands::cycles::execute(args, cli.json, cli.debug, ws).await,
        Commands::Initiatives(args) => {
            commands::initiatives::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Roadmap(args) => commands::roadmap::execute(args, cli.json, cli.debug, ws).await,
        Commands::Labels(args) => commands::labels::execute(args, cli.json, cli.debug, ws).await,
        Commands::Teams(args) => commands::teams::execute(args, cli.json, cli.debug, ws).await,
        Commands::Relations(args) => {
            commands::relations::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Customers(args) => {
            commands::customers::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Views(args) => commands::views::execute(args, cli.json, cli.debug, ws).await,
        Commands::Docs(args) => commands::docs::execute(args, cli.json, cli.debug, ws).await,
        Commands::Notifications(args) => {
            commands::notifications::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Me(args) => commands::me::execute(args, cli.json, cli.debug, ws).await,
        Commands::Attachments(args) => {
            commands::attachments::execute(args, cli.json, cli.debug, ws).await
        }
        Commands::Search(args) => commands::search::execute(args, cli.json, cli.debug, ws).await,
        Commands::Config(args) => commands::config::execute(args, cli.json, cli.debug, ws).await,
        Commands::Completions { .. } => unreachable!(),
    };

    let duration = start.elapsed();
    let success = result.is_ok();

    // Print error immediately so user sees it without waiting for flush
    if let Err(ref e) = result {
        eprintln!("Error: {e}");
    }

    // Track analytics event (sync — writes to local queue file)
    let mut flags = Vec::new();
    if cli.json {
        flags.push("--json".to_string());
    }
    if cli.debug {
        flags.push("--debug".to_string());
    }
    if cli.workspace.is_some() {
        flags.push("--workspace".to_string());
    }

    analytics::track(&analytics::Event {
        command: analytics::command_name(&cli.command).to_string(),
        flags,
        success,
        duration_ms: duration.as_millis() as u64,
    });

    // Flush analytics queue in background, wait up to 3s
    let flush_handle = tokio::spawn(analytics::flush());
    let _ = tokio::time::timeout(std::time::Duration::from_secs(3), flush_handle).await;

    if result.is_err() {
        std::process::exit(1);
    }
}
