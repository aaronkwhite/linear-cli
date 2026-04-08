use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthConfig {
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

/// Load the config from disk. Returns default config if the file doesn't exist.
pub fn load() -> anyhow::Result<Config> {
    let path = match config_path() {
        Some(p) => p,
        None => return Ok(Config::default()),
    };

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

/// Save the config to disk, creating the directory if needed.
pub fn save(config: &Config) -> anyhow::Result<()> {
    let dir = config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let path = dir.join("config.toml");

    fs::create_dir_all(&dir)?;
    let contents = toml::to_string_pretty(config)?;
    fs::write(&path, contents)?;
    Ok(())
}

/// Read the API key from the config file, if present.
pub fn get_api_key() -> Option<String> {
    let config = load().ok()?;
    config.auth.api_key.filter(|k| !k.is_empty())
}

/// Save an API key to the config file, preserving other settings.
pub fn set_api_key(key: &str) -> anyhow::Result<()> {
    let mut config = load().unwrap_or_default();
    config.auth.api_key = Some(key.to_string());
    save(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_no_key() {
        let config = Config::default();
        assert!(config.auth.api_key.is_none());
    }

    #[test]
    fn test_roundtrip_toml() {
        let mut config = Config::default();
        config.auth.api_key = Some("lin_api_test123".to_string());
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.auth.api_key.as_deref(), Some("lin_api_test123"));
    }

    #[test]
    fn test_config_path_exists() {
        // config_path should return Some on any platform with a home dir
        let path = config_path();
        assert!(path.is_some());
        let p = path.unwrap();
        assert!(p.to_string_lossy().contains("lin"));
        assert!(p.to_string_lossy().ends_with("config.toml"));
    }
}
