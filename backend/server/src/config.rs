use std::env;

#[allow(dead_code)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub oauth_redirect_base_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: require_var("DATABASE_URL"),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            jwt_secret: require_var("JWT_SECRET"),
            google_client_id: require_var("GOOGLE_CLIENT_ID"),
            google_client_secret: require_var("GOOGLE_CLIENT_SECRET"),
            github_client_id: require_var("GITHUB_CLIENT_ID"),
            github_client_secret: require_var("GITHUB_CLIENT_SECRET"),
            oauth_redirect_base_url: require_var("OAUTH_REDIRECT_BASE_URL"),
        }
    }
}

fn require_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Missing required environment variable: {name}"))
}
