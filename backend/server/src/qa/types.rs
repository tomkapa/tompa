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
    #[error("Member is not part of this organization")]
    InvalidAssignee,
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
    pub assigned_to: Option<Uuid>,
}

/// Minimal summary of a decision pattern that was injected into a Q&A prompt.
/// Stored inside `QaContent` so provenance is preserved with the round.
/// `override_count` is a snapshot at injection time so the Q&A view can
/// show an "outdated?" alert without an extra fetch.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AppliedPatternSummary {
    pub id: Uuid,
    pub domain: String,
    pub pattern: String,
    pub confidence: f32,
    pub override_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct QaContent {
    pub questions: Vec<QaQuestion>,
    pub course_correction: Option<String>,
    /// Patterns injected into the LLM prompt that generated this round's questions.
    #[serde(default)]
    pub applied_patterns: Vec<AppliedPatternSummary>,
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignQuestionRequest {
    pub member_id: Uuid,
}

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
    /// Number of decision patterns that were injected into this round's prompt.
    pub applied_pattern_count: usize,
    /// The patterns that were injected (id, domain, pattern text, confidence).
    pub applied_patterns: Vec<AppliedPatternSummary>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Validation helpers ────────────────────────────────────────────────────────

pub const VALID_STAGES: &[&str] = &["grooming", "planning", "task_qa", "implementation"];
