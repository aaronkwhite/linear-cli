use clap::{Args, Subcommand};
use console::style;
use dialoguer::{Password, theme::ColorfulTheme};

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Set the Linear API key (saved to config file)
    SetToken,
    /// Show the stored API key (masked)
    GetToken,
    /// Print the config file path
    Path,
    /// Manage anonymous usage analytics
    Analytics(AnalyticsArgs),
}

#[derive(Args, Debug)]
pub struct AnalyticsArgs {
    #[command(subcommand)]
    pub command: AnalyticsCommand,
}

#[derive(Subcommand, Debug)]
pub enum AnalyticsCommand {
    /// Enable anonymous usage analytics
    On,
    /// Disable anonymous usage analytics
    Off,
    /// Show analytics status
    Status,
}

pub async fn execute(
    args: &ConfigArgs,
    json: bool,
    _debug: bool,
    _workspace: Option<&str>,
) -> anyhow::Result<()> {
    match &args.command {
        ConfigCommand::SetToken => set_token(json),
        ConfigCommand::GetToken => get_token(json),
        ConfigCommand::Path => print_path(json),
        ConfigCommand::Analytics(args) => analytics_cmd(&args.command, json),
    }
}

fn set_token(json: bool) -> anyhow::Result<()> {
    let key = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter your Linear API key")
        .interact()?;

    if key.is_empty() {
        anyhow::bail!("API key cannot be empty");
    }

    crate::config::set_api_key(&key)?;

    let path = crate::config::config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if json {
        let output = serde_json::json!({
            "saved": true,
            "path": path,
        });
        crate::output::print_json(&output);
    } else {
        println!(
            "{} API key saved to {}",
            style("✓").green().bold(),
            style(&path).cyan()
        );
    }

    Ok(())
}

fn get_token(json: bool) -> anyhow::Result<()> {
    match crate::config::get_api_key() {
        Some(key) => {
            let masked = mask_token(&key);
            if json {
                let output = serde_json::json!({
                    "token": masked,
                    "configured": true,
                });
                crate::output::print_json(&output);
            } else {
                println!("{masked}");
            }
        }
        None => {
            if json {
                let output = serde_json::json!({
                    "token": null,
                    "configured": false,
                });
                crate::output::print_json(&output);
            } else {
                println!("No API key configured. Run `lin config set-token` to set one.");
            }
        }
    }
    Ok(())
}

fn print_path(json: bool) -> anyhow::Result<()> {
    let path = crate::config::config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if json {
        let output = serde_json::json!({ "path": path });
        crate::output::print_json(&output);
    } else {
        println!("{path}");
    }
    Ok(())
}

fn analytics_cmd(cmd: &AnalyticsCommand, json: bool) -> anyhow::Result<()> {
    match cmd {
        AnalyticsCommand::Off => {
            crate::config::set_analytics_enabled(false)?;
            if json {
                crate::output::print_json(&serde_json::json!({ "analytics": false }));
            } else {
                println!("{} Analytics disabled", style("✓").green().bold());
            }
        }
        AnalyticsCommand::On => {
            crate::config::set_analytics_enabled(true)?;
            if json {
                crate::output::print_json(&serde_json::json!({ "analytics": true }));
            } else {
                println!("{} Analytics enabled", style("✓").green().bold());
            }
        }
        AnalyticsCommand::Status => {
            let enabled = crate::analytics::is_enabled();
            let do_not_track = std::env::var("DO_NOT_TRACK").ok().as_deref() == Some("1");
            let config_value = crate::config::load().ok().and_then(|c| c.analytics_enabled);
            let install_id_path = crate::config::config_dir().map(|d| d.join("analytics_id"));
            let install_id = install_id_path
                .and_then(|p| std::fs::read_to_string(p).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            if json {
                crate::output::print_json(&serde_json::json!({
                    "enabled": enabled,
                    "config_value": config_value,
                    "do_not_track_override": do_not_track,
                    "install_id": install_id,
                }));
            } else {
                let status = if enabled { "enabled" } else { "disabled" };
                println!("Analytics: {}", style(status).bold());
                if do_not_track {
                    println!(
                        "  {} DO_NOT_TRACK=1 is set (overrides config)",
                        style("!").yellow().bold()
                    );
                }
                if let Some(id) = install_id {
                    println!("  Install ID: {}", style(id).dim());
                }
            }
        }
    }
    Ok(())
}

/// Mask a token, showing only the last 4 characters.
fn mask_token(token: &str) -> String {
    if token.len() <= 4 {
        return "*".repeat(token.len());
    }
    let visible = &token[token.len() - 4..];
    format!("{}{}", "*".repeat(token.len() - 4), visible)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_token_normal() {
        assert_eq!(mask_token("lin_api_abcdef1234"), "**************1234");
    }

    #[test]
    fn test_mask_token_short() {
        assert_eq!(mask_token("abc"), "***");
    }

    #[test]
    fn test_mask_token_exactly_four() {
        assert_eq!(mask_token("abcd"), "****");
    }

    #[test]
    fn test_mask_token_five() {
        assert_eq!(mask_token("abcde"), "*bcde");
    }
}
