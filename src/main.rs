mod cli;
mod client;
mod commands;
mod config;
mod error;
mod graphql;
mod output;

use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Issues(args) => commands::issues::execute(args, cli.json, cli.debug).await,
        Commands::Projects(args) => commands::projects::execute(args, cli.json, cli.debug).await,
        Commands::Cycles(args) => commands::cycles::execute(args, cli.json, cli.debug).await,
        Commands::Initiatives(args) => {
            commands::initiatives::execute(args, cli.json, cli.debug).await
        }
        Commands::Roadmap(args) => commands::roadmap::execute(args, cli.json, cli.debug).await,
        Commands::Labels(args) => commands::labels::execute(args, cli.json, cli.debug).await,
        Commands::Teams(args) => commands::teams::execute(args, cli.json, cli.debug).await,
        Commands::Relations(args) => commands::relations::execute(args, cli.json, cli.debug).await,
        Commands::Customers(args) => commands::customers::execute(args, cli.json, cli.debug).await,
        Commands::Views(args) => commands::views::execute(args, cli.json, cli.debug).await,
        Commands::Docs(args) => commands::docs::execute(args, cli.json, cli.debug).await,
        Commands::Notifications(args) => {
            commands::notifications::execute(args, cli.json, cli.debug).await
        }
        Commands::Me(args) => commands::me::execute(args, cli.json, cli.debug).await,
        Commands::Attachments(args) => {
            commands::attachments::execute(args, cli.json, cli.debug).await
        }
        Commands::Search(args) => commands::search::execute(args, cli.json, cli.debug).await,
        Commands::Config(args) => commands::config::execute(args, cli.json, cli.debug).await,
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
