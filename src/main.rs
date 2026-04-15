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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let ws = cli.workspace.as_deref();

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
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(*shell, &mut cmd, "lin", &mut std::io::stdout());
            return;
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
