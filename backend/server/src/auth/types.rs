use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: Uuid,     // user_id
    pub org_id: Uuid,
    pub role: String,  // "owner", "admin", "member"
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthProfile {
    pub provider: String,
    pub provider_id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

/// Response body for GET /api/v1/auth/me
#[derive(Debug, Serialize, ToSchema)]
pub struct MeResponse {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub org_id: Uuid,
    pub org_name: Option<String>,
    pub role: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid or expired token")]
    InvalidToken,
}
