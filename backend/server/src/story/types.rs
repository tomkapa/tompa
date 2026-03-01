use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum StoryError {
    #[error("Story not found")]
    NotFound,
    #[error("Invalid status transition from '{from}' to '{to}'")]
    InvalidTransition { from: String, to: String },
    #[error("Story has active tasks")]
    HasActiveTasks,
    #[error("Story title is required")]
    TitleRequired,
    #[error("Invalid story type; must be 'feature', 'bug', or 'refactor'")]
    InvalidStoryType,
    #[error("Invalid pipeline stage")]
    InvalidPipelineStage,
}

// ── Request types ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateStoryRequest {
    pub project_id: Uuid,
    pub title: String,
    pub description: String,
    pub story_type: String,
    pub owner_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStoryRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub pipeline_stage: Option<String>,
}

/// Used for the `PATCH /stories/:id/rank` endpoint.
///
/// The target story is repositioned so that it appears **after** `after_id`
/// and **before** `before_id` in the ordered list.
///
/// - Both `None` → error (at least one must be provided)
/// - Only `before_id` → target moves to the top, before `before_id`
/// - Only `after_id` → target moves to the end, after `after_id`
/// - Both set → target is placed between `after_id` and `before_id`
#[derive(Debug, Deserialize, ToSchema)]
pub struct RankUpdateRequest {
    pub before_id: Option<Uuid>,
    pub after_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListStoriesParams {
    pub project_id: Uuid,
}

// ── Response types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskSummary {
    pub id: Uuid,
    pub name: String,
    pub task_type: String,
    pub state: String,
    pub position: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StoryResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: String,
    pub story_type: String,
    pub status: String,
    pub owner_id: Uuid,
    pub rank: String,
    pub pipeline_stage: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tasks: Vec<TaskSummary>,
}

// ── Validation helpers ────────────────────────────────────────────────────────

pub const VALID_STORY_TYPES: &[&str] = &["feature", "bug", "refactor"];

pub const VALID_PIPELINE_STAGES: &[&str] = &[
    "grooming",
    "planning",
    "decomposition",
    "implementation",
    "testing",
    "review",
];
