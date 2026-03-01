use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

pub const VALID_MODES: &[&str] = &["project", "dev", "standalone"];

#[derive(Debug, Error)]
pub enum ContainerKeyError {
    #[error("Label is required")]
    LabelRequired,
    #[error("Invalid container mode: must be project, dev, or standalone")]
    InvalidMode,
    #[error("Project not found")]
    ProjectNotFound,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateKeyRequest {
    pub project_id: Uuid,
    pub label: String,
    pub container_mode: String,
}

/// Returned once on creation — `api_key` is never stored in plaintext.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateKeyResponse {
    pub id: Uuid,
    pub api_key: String,
    pub label: String,
    pub container_mode: String,
    pub created_at: DateTime<Utc>,
}

/// Safe list view — never includes the key hash.
#[derive(Debug, Serialize, ToSchema)]
pub struct KeyListItem {
    pub id: Uuid,
    pub label: String,
    pub container_mode: String,
    pub last_connected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ListKeysParams {
    pub project_id: Uuid,
}

/// Returned by `verify_api_key` to the WebSocket handler (T16).
pub struct ContainerKeyInfo {
    pub key_id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub container_mode: String,
}
