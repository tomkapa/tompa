use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ProjectProfileError {
    #[error("Project profile not found")]
    NotFound,
    #[error("Profile content is required")]
    ContentRequired,
}

/// The structured JSON content of a project profile.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectProfileContent {
    pub identity: String,
    pub tech_stack: HashMap<String, String>,
    pub architectural_patterns: Vec<String>,
    pub conventions: Vec<String>,
    pub team_preferences: Vec<String>,
    pub domain_knowledge: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectProfileResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub content: ProjectProfileContent,
    pub patterns_at_generation: i32,
    pub generated_by: String,
    pub generated_at: Option<DateTime<Utc>>,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub content: ProjectProfileContent,
}
