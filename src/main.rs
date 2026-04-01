mod cli;
mod client;
mod commands;
mod error;
mod graphql;
mod output;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Issues(args) => commands::issues::execute(args, cli.json, cli.debug).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
