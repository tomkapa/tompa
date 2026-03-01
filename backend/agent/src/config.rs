use anyhow::{Context, Result};
use serde::Deserialize;
use shared::enums::ContainerMode;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub mode: ContainerMode,
    pub server_url: String,
    pub api_key: String,
    pub github_repo_url: Option<String>,
    pub github_access_token: Option<String>,
    pub setup_ui_port: Option<u16>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Config file not found or unreadable: {path}"))?;
        let config: Config = toml::from_str(&content).context("Failed to parse config.toml")?;
        Ok(config)
    }
}
