use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "lin",
    about = "lin — Linear CLI. Manage issues, projects, cycles, and more.",
    version
)]
pub struct Cli {
    /// Output raw JSON for scripting
    #[arg(long, global = true)]
    pub json: bool,

    /// Print GraphQL queries/responses to stderr
    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage issues
    Issues(crate::commands::issues::IssuesArgs),
}
