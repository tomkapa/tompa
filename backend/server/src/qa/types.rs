use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum QaError {
    #[error("QA round not found")]
    NotFound,
    #[error("QA round is not active")]
    RoundNotActive,
    #[error("Question already answered")]
    AlreadyAnswered,
    #[error("Invalid rollback")]
    InvalidRollback,
    #[error("Question not found in round")]
    QuestionNotFound,
    #[error("story_id or task_id query parameter is required")]
    MissingFilter,
}

// ── JSONB content structures ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct QaQuestionOption {
    pub label: String,
    pub pros: String,
    pub cons: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct QaQuestion {
    pub id: Uuid,
    pub text: String,
    pub domain: String,
    pub rationale: String,
    pub options: Vec<QaQuestionOption>,
    pub recommended_option_index: usize,
    pub selected_answer_index: Option<i32>,
    pub selected_answer_text: Option<String>,
    pub answered_by: Option<Uuid>,
    pub answered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct QaContent {
    pub questions: Vec<QaQuestion>,
    pub course_correction: Option<String>,
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmitAnswerRequest {
    pub question_id: Uuid,
    pub selected_answer_index: Option<i32>,
    pub answer_text: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CourseCorrectionRequest {
    pub story_id: Uuid,
    pub task_id: Option<Uuid>,
    pub stage: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQaRoundsParams {
    pub story_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub stage: Option<String>,
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct QaRoundResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub story_id: Uuid,
    pub task_id: Option<Uuid>,
    pub stage: String,
    pub round_number: i32,
    pub status: String,
    pub content: QaContent,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Validation helpers ────────────────────────────────────────────────────────

pub const VALID_STAGES: &[&str] = &["grooming", "planning", "task_qa", "implementation"];
