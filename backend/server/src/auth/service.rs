use anyhow::anyhow;
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use reqwest::Client;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;

use super::types::{AuthClaims, AuthError, OAuthProfile};

const JWT_EXPIRY_SECS: i64 = 7 * 24 * 3600;

pub fn create_jwt(claims: &AuthClaims, secret: &str) -> anyhow::Result<String> {
    encode(
        &Header::default(), // HS256
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(Into::into)
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<AuthClaims, AuthError> {
    decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::InvalidToken)
}

pub fn make_claims(user_id: Uuid, org_id: Uuid, role: &str) -> AuthClaims {
    let now = Utc::now().timestamp();
    AuthClaims {
        sub: user_id,
        org_id,
        role: role.to_string(),
        iat: now,
        exp: now + JWT_EXPIRY_SECS,
    }
}

// ── Google OAuth ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    id: String,
    email: String,
    name: String,
    picture: Option<String>,
}

pub async fn exchange_google_code(code: &str, config: &Config) -> anyhow::Result<OAuthProfile> {
    let client = Client::new();
    let redirect_uri = format!(
        "{}/api/v1/auth/callback/google",
        config.oauth_redirect_base_url
    );

    let token: GoogleTokenResponse = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", &config.google_client_id),
            ("client_secret", &config.google_client_secret),
            ("redirect_uri", &redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let user: GoogleUserInfo = client
        .get("https://www.googleapis.com/oauth2/v1/userinfo")
        .bearer_auth(&token.access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(OAuthProfile {
        provider: "google".to_string(),
        provider_id: user.id,
        email: user.email,
        display_name: user.name,
        avatar_url: user.picture,
    })
}

// ── GitHub OAuth ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GitHubUser {
    id: i64,
    login: String,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

pub async fn exchange_github_code(code: &str, config: &Config) -> anyhow::Result<OAuthProfile> {
    let client = Client::new();
    let redirect_uri = format!(
        "{}/api/v1/auth/callback/github",
        config.oauth_redirect_base_url
    );

    let token: GitHubTokenResponse = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("code", code),
            ("client_id", &config.github_client_id),
            ("client_secret", &config.github_client_secret),
            ("redirect_uri", &redirect_uri),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let user: GitHubUser = client
        .get("https://api.github.com/user")
        .bearer_auth(&token.access_token)
        .header("User-Agent", "tompa-server/1.0")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let email = match user.email {
        Some(e) => e,
        None => {
            let emails: Vec<GitHubEmail> = client
                .get("https://api.github.com/user/emails")
                .bearer_auth(&token.access_token)
                .header("User-Agent", "tompa-server/1.0")
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;

            emails
                .into_iter()
                .find(|e| e.primary && e.verified)
                .map(|e| e.email)
                .ok_or_else(|| anyhow!("No verified primary email on GitHub account"))?
        }
    };

    Ok(OAuthProfile {
        provider: "github".to_string(),
        provider_id: user.id.to_string(),
        email,
        display_name: user.name.unwrap_or(user.login),
        avatar_url: user.avatar_url,
    })
}

// ── User upsert ───────────────────────────────────────────────────────────────

pub struct UserOrg {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub role: String,
}

/// Upsert the user from an OAuth profile, auto-creating a personal org on first login.
pub async fn find_or_create_user(pool: &PgPool, profile: &OAuthProfile) -> anyhow::Result<UserOrg> {
    let mut tx = pool.begin().await?;

    // Upsert user — conflict on (oauth_provider, oauth_provider_id)
    let user_id: Uuid = sqlx::query_scalar!(
        r#"
        INSERT INTO users (id, email, display_name, avatar_url, oauth_provider, oauth_provider_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (oauth_provider, oauth_provider_id) DO UPDATE
            SET email        = EXCLUDED.email,
                display_name = EXCLUDED.display_name,
                avatar_url   = EXCLUDED.avatar_url,
                updated_at   = now()
        RETURNING id
        "#,
        Uuid::now_v7(),
        profile.email,
        profile.display_name,
        profile.avatar_url,
        profile.provider,
        profile.provider_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    // Check for existing org membership (oldest / primary org)
    let membership = sqlx::query!(
        r#"
        SELECT org_id, role
        FROM org_members
        WHERE user_id = $1
        ORDER BY created_at ASC
        LIMIT 1
        "#,
        user_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let (org_id, role) = match membership {
        Some(m) => (m.org_id, m.role),
        None => {
            // First login — auto-create a personal workspace org
            let org_id = Uuid::now_v7();
            sqlx::query!(
                "INSERT INTO organizations (id, name) VALUES ($1, $2)",
                org_id,
                format!("{}'s Workspace", profile.display_name),
            )
            .execute(&mut *tx)
            .await?;

            sqlx::query!(
                "INSERT INTO org_members (id, org_id, user_id, role) VALUES ($1, $2, $3, 'owner')",
                Uuid::now_v7(),
                org_id,
                user_id,
            )
            .execute(&mut *tx)
            .await?;

            (org_id, "owner".to_string())
        }
    };

    tx.commit().await?;
    Ok(UserOrg {
        user_id,
        org_id,
        role,
    })
}
