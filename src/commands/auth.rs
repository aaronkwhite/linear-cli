use clap::{Args, Subcommand};
use dialoguer::{Input, Password, theme::ColorfulTheme};

#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Add a workspace (interactive, or use --name/--key for scripting)
    Login {
        /// Workspace name (required in non-interactive mode)
        #[arg(long)]
        name: Option<String>,
        /// API key (required in non-interactive mode)
        #[arg(long)]
        key: Option<String>,
    },
    /// List configured workspaces
    List,
    /// Set the default workspace
    Default {
        /// Workspace name
        name: String,
    },
    /// Show current workspace and authenticated user
    Whoami,
}

pub async fn execute(
    args: &AuthArgs,
    json: bool,
    debug: bool,
    workspace: Option<&str>,
) -> anyhow::Result<()> {
    match &args.command {
        AuthCommand::Login {
            name: arg_name,
            key: arg_key,
        } => {
            let (name, key) = match (arg_name, arg_key) {
                (Some(n), Some(k)) => (n.clone(), k.clone()),
                _ if crate::output::interactive::is_interactive() => {
                    let n: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Workspace name")
                        .interact_text()?;
                    let k = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Linear API key")
                        .interact()?;
                    (n, k)
                }
                _ => {
                    anyhow::bail!(
                        "Non-interactive mode: provide --name and --key (e.g., lin auth login --name myco --key lin_api_...)"
                    );
                }
            };

            if name.is_empty() || key.is_empty() {
                anyhow::bail!("Workspace name and API key cannot be empty");
            }

            crate::config::set_workspace_key(&name, &key)?;

            if json {
                crate::output::print_json(&serde_json::json!({
                    "workspace": name,
                    "saved": true,
                }));
            } else {
                println!(
                    "  {} Workspace '{}' saved",
                    crate::output::color::green("OK"),
                    crate::output::color::bold(&name),
                );
            }
        }

        AuthCommand::List => {
            let (workspaces, default) = crate::config::list_workspaces()?;

            if json {
                crate::output::print_json(&serde_json::json!({
                    "workspaces": workspaces,
                    "default": default,
                }));
            } else if workspaces.is_empty() {
                println!("  No workspaces configured. Run `lin auth login` to add one.");
            } else {
                for ws in &workspaces {
                    let marker = if default.as_deref() == Some(ws.as_str()) {
                        " *"
                    } else {
                        ""
                    };
                    println!("  {}{}", ws, marker);
                }
            }
        }

        AuthCommand::Default { name } => {
            crate::config::set_default_workspace(name)?;
            if json {
                crate::output::print_json(&serde_json::json!({
                    "default": name,
                }));
            } else {
                println!(
                    "  {} Default workspace set to '{}'",
                    crate::output::color::green("OK"),
                    crate::output::color::bold(name),
                );
            }
        }

        AuthCommand::Whoami => {
            let client = crate::client::LinearClient::new(None, debug, workspace)?;
            let result = client
                .query_raw("query { viewer { id displayName email } }", None)
                .await?;

            if json {
                crate::output::print_json(&result);
            } else {
                let viewer = result.pointer("/data/viewer");
                let name = viewer
                    .and_then(|v| v.get("displayName"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let email = viewer
                    .and_then(|v| v.get("email"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let ws = workspace
                    .map(|s| s.to_string())
                    .or_else(|| crate::config::load().ok().and_then(|c| c.default_workspace))
                    .unwrap_or_else(|| "env".to_string());
                println!("  {} ({}) — workspace: {}", name, email, ws);
            }
        }
    }
    Ok(())
}
