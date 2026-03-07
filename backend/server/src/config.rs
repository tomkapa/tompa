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
    pub dev_mode: bool,
}

impl Config {
    pub fn from_env() -> Self {
        let dev_mode = env::var("DEV_MODE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let require_or_default = |name: &str| -> String {
            if dev_mode {
                // In dev mode OAuth credentials are optional; missing vars default to empty
                // so the server starts without them (only dev-login is used).
                env::var(name).unwrap_or_default()
            } else {
                require_var(name)
            }
        };

        Self {
            database_url: require_var("DATABASE_URL"),
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            jwt_secret: require_var("JWT_SECRET"),
            google_client_id: require_or_default("GOOGLE_CLIENT_ID"),
            google_client_secret: require_or_default("GOOGLE_CLIENT_SECRET"),
            github_client_id: require_or_default("GITHUB_CLIENT_ID"),
            github_client_secret: require_or_default("GITHUB_CLIENT_SECRET"),
            oauth_redirect_base_url: require_or_default("OAUTH_REDIRECT_BASE_URL"),
            dev_mode,
        }
    }
}

fn require_var(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Missing required environment variable: {name}"))
}
