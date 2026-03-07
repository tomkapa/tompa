use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Project not found")]
    NotFound,
    #[error("Project name is required")]
    NameRequired,
    #[error("A project with that name already exists")]
    NameTaken,
    #[error("Invalid Q&A configuration: {0}")]
    InvalidQaConfig(String),
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub github_repo_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub github_repo_url: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateQaConfigRequest {
    pub qa_config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ListProjectsParams {
    /// Optional — if provided, must match the org_id in the session JWT.
    pub org_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub github_repo_url: Option<String>,
    pub qa_config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
