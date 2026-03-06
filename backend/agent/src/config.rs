use std::env;

use shared::enums::ContainerMode;

pub struct Config {
    pub mode: ContainerMode,
    pub server_url: String,
    pub api_key: String,
    #[allow(dead_code)]
    pub github_repo_url: Option<String>,
    #[allow(dead_code)]
    pub github_access_token: Option<String>,
    pub setup_ui_port: u16,
    pub claude_cmd: String,
    pub max_concurrent_processes: usize,
}

impl Config {
    pub fn from_env() -> Self {
        let mode = match require_var("AGENT_MODE").to_lowercase().as_str() {
            "project" => ContainerMode::Project,
            "dev" => ContainerMode::Dev,
            "standalone" => ContainerMode::Standalone,
            other => panic!("Invalid AGENT_MODE: {other} (expected project|dev|standalone)"),
        };

        Self {
            mode,
            server_url: require_var("AGENT_SERVER_URL"),
            api_key: require_var("AGENT_API_KEY"),
            github_repo_url: env::var("AGENT_GITHUB_REPO_URL").ok(),
            github_access_token: env::var("AGENT_GITHUB_ACCESS_TOKEN").ok(),
            setup_ui_port: env::var("AGENT_SETUP_UI_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3001),
            claude_cmd: env::var("CLAUDE_CMD").unwrap_or_else(|_| "claude".into()),
            max_concurrent_processes: env::var("AGENT_MAX_CONCURRENT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        }
    }
}

fn require_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Missing required environment variable: {name}"))
}
