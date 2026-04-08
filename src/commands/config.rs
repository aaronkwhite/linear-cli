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
}

pub async fn execute(args: &ConfigArgs, json: bool, _debug: bool) -> anyhow::Result<()> {
    match &args.command {
        ConfigCommand::SetToken => set_token(json),
        ConfigCommand::GetToken => get_token(json),
        ConfigCommand::Path => print_path(json),
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
