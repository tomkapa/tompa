use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum KnowledgeError {
    #[error("Knowledge entry not found")]
    NotFound,
    #[error("Invalid category: must be one of convention, adr, api_doc, design_system, custom")]
    InvalidCategory,
    #[error("Title is required")]
    TitleRequired,
    #[error("Content is required")]
    ContentRequired,
}

const VALID_CATEGORIES: &[&str] = &["convention", "adr", "api_doc", "design_system", "custom"];

pub fn is_valid_category(cat: &str) -> bool {
    VALID_CATEGORIES.contains(&cat)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateKnowledgeRequest {
    pub project_id: Option<Uuid>,
    pub story_id: Option<Uuid>,
    pub category: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateKnowledgeRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListKnowledgeParams {
    pub project_id: Option<Uuid>,
    pub story_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct KnowledgeResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Option<Uuid>,
    pub story_id: Option<Uuid>,
    pub category: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
