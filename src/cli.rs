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

    /// Print GraphQL queries and full API responses to stderr (may contain workspace data)
    #[arg(long, global = true)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage issues
    Issues(crate::commands::issues::IssuesArgs),
    /// Manage projects
    Projects(crate::commands::projects::ProjectsArgs),
    /// Manage cycles
    Cycles(crate::commands::cycles::CyclesArgs),
    /// Roadmap: updates, milestones, initiatives
    Roadmap(crate::commands::roadmap::RoadmapArgs),
    /// Manage labels
    Labels(crate::commands::labels::LabelsArgs),
    /// Manage teams
    Teams(crate::commands::teams::TeamsArgs),
    /// Manage issue relations
    Relations(crate::commands::relations::RelationsArgs),
    /// Manage customers
    Customers(crate::commands::customers::CustomersArgs),
    /// Manage custom views
    Views(crate::commands::views::ViewsArgs),
    /// Manage documents
    Docs(crate::commands::docs::DocsArgs),
    /// Manage notifications
    Notifications(crate::commands::notifications::NotificationsArgs),
}
