use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use shared::enums::TaskState;

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Task not found")]
    NotFound,
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidState { from: TaskState, to: TaskState },
    #[error("Cyclic dependency detected")]
    CyclicDependency,
    #[error("Story not found")]
    StoryNotFound,
    #[error("Task must be in 'running' state to mark done")]
    NotRunning,
    #[error("Both tasks must belong to the same story")]
    DifferentStory,
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub story_id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub position: i32,
    pub assignee_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTaskRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub position: Option<i32>,
    pub assignee_id: Option<Uuid>,
    pub state: Option<String>,
    pub claude_session_id: Option<String>,
    pub ai_status_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTasksParams {
    pub story_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ListDependenciesParams {
    pub story_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDependencyRequest {
    pub task_id: Uuid,
    pub depends_on_task_id: Uuid,
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct DependencyResponse {
    pub id: Uuid,
    pub task_id: Uuid,
    pub depends_on_task_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub story_id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub state: String,
    pub position: i32,
    pub assignee_id: Option<Uuid>,
    pub claude_session_id: Option<String>,
    pub ai_status_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub dependencies: Vec<DependencyResponse>,
}

// ── Validation constants ──────────────────────────────────────────────────────

pub const VALID_TASK_TYPES: &[&str] = &["design", "test", "code"];
