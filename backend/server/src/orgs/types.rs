use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum OrgError {
    #[error("Organization not found")]
    NotFound,
    #[error("Organization name is required")]
    NameRequired,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOrgRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OrgResponse {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}
