use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Default workspace name
    pub default_workspace: Option<String>,

    /// Named workspaces
    #[serde(default)]
    pub workspaces: BTreeMap<String, WorkspaceConfig>,

    /// Legacy auth section (migrated on load)
    #[serde(default, skip_serializing)]
    pub auth: Option<LegacyAuth>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceConfig {
    pub api_key: String,
}

/// Old format — only used for migration
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LegacyAuth {
    pub api_key: Option<String>,
}

/// Returns the config directory: `~/.config/lin`
pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("lin"))
}

/// Returns the config file path: `~/.config/lin/config.toml`
pub fn config_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.toml"))
}

/// Load config, auto-migrating legacy `[auth]` format to workspaces.
pub fn load() -> anyhow::Result<Config> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(Config::default()),
    };

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&path)?;
    let mut config: Config = toml::from_str(&contents)?;

    // Migrate legacy [auth] section
    if let Some(legacy) = config.auth.take()
        && let Some(key) = legacy.api_key.filter(|k| !k.is_empty())
        && config.workspaces.is_empty()
    {
        config
            .workspaces
            .insert("default".to_string(), WorkspaceConfig { api_key: key });
        config.default_workspace = Some("default".to_string());
        let _ = save(&config);
    }

    Ok(config)
}

/// Save the config to disk.
pub fn save(config: &Config) -> anyhow::Result<()> {
    let dir =
        config_dir().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let path = dir.join("config.toml");

    fs::create_dir_all(&dir)?;
    let contents = toml::to_string_pretty(config)?;
    fs::write(&path, contents)?;
    Ok(())
}

/// Get the API key for a specific workspace or the default.
pub fn get_workspace_key(workspace: Option<&str>) -> Option<String> {
    let config = load().ok()?;
    let ws_name = workspace
        .map(|s| s.to_string())
        .or(config.default_workspace)?;
    config.workspaces.get(&ws_name).map(|ws| ws.api_key.clone())
}

/// Read the API key from the config file (default workspace). Backwards compat.
pub fn get_api_key() -> Option<String> {
    get_workspace_key(None)
}

/// Save an API key as a named workspace.
pub fn set_workspace_key(name: &str, key: &str) -> anyhow::Result<()> {
    let mut config = load().unwrap_or_default();
    config.workspaces.insert(
        name.to_string(),
        WorkspaceConfig {
            api_key: key.to_string(),
        },
    );
    if config.default_workspace.is_none() {
        config.default_workspace = Some(name.to_string());
    }
    save(&config)
}

/// Set the default workspace.
pub fn set_default_workspace(name: &str) -> anyhow::Result<()> {
    let mut config = load().unwrap_or_default();
    if !config.workspaces.contains_key(name) {
        anyhow::bail!("Workspace '{}' not found", name);
    }
    config.default_workspace = Some(name.to_string());
    save(&config)
}

/// List all configured workspaces.
pub fn list_workspaces() -> anyhow::Result<(Vec<String>, Option<String>)> {
    let config = load().unwrap_or_default();
    let names: Vec<String> = config.workspaces.keys().cloned().collect();
    Ok((names, config.default_workspace))
}

/// Save an API key to the config file as "default" workspace. Backwards compat.
pub fn set_api_key(key: &str) -> anyhow::Result<()> {
    set_workspace_key("default", key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_no_key() {
        let config = Config::default();
        assert!(config.workspaces.is_empty());
        assert!(config.default_workspace.is_none());
    }

    #[test]
    fn test_roundtrip_toml() {
        let config = Config {
            default_workspace: Some("myco".to_string()),
            workspaces: BTreeMap::from([(
                "myco".to_string(),
                WorkspaceConfig {
                    api_key: "lin_api_test123".to_string(),
                },
            )]),
            ..Config::default()
        };
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.default_workspace.as_deref(), Some("myco"));
        assert_eq!(
            deserialized.workspaces.get("myco").unwrap().api_key,
            "lin_api_test123"
        );
    }

    #[test]
    fn test_legacy_migration() {
        let legacy_toml = r#"
[auth]
api_key = "lin_api_legacy"
"#;
        let mut config: Config = toml::from_str(legacy_toml).unwrap();
        if let Some(legacy) = config.auth.take()
            && let Some(key) = legacy.api_key
        {
            config
                .workspaces
                .insert("default".to_string(), WorkspaceConfig { api_key: key });
            config.default_workspace = Some("default".to_string());
        }
        assert_eq!(
            config.workspaces.get("default").unwrap().api_key,
            "lin_api_legacy"
        );
    }

    #[test]
    fn test_config_path_exists() {
        let path = config_path();
        assert!(path.is_some());
        let p = path.unwrap();
        assert!(p.to_string_lossy().contains("lin"));
        assert!(p.to_string_lossy().ends_with("config.toml"));
    }
}
