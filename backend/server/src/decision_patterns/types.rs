use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DecisionPatternError {
    #[error("Decision pattern not found")]
    NotFound,
    #[error("Pattern text is required")]
    PatternRequired,
    #[error("Rationale is required")]
    RationaleRequired,
    #[error("Invalid domain: must be one of development, security, design, business, marketing")]
    InvalidDomain,
    #[error("New pattern text is required for supersede")]
    SupersedePatternRequired,
}

pub const VALID_DOMAINS: &[&str] = &[
    "development",
    "security",
    "design",
    "business",
    "marketing",
    "planning",
];

pub fn is_valid_domain(domain: &str) -> bool {
    VALID_DOMAINS.contains(&domain)
}

/// Pattern extracted from LLM output (piggybacked on QA round).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPattern {
    pub domain: String,
    pub pattern: String,
    pub rationale: String,
    pub tags: Vec<String>,
}

/// Classification result from dedup logic.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternClassification {
    /// Similarity > 0.8 — skip insert.
    Duplicate { existing_id: Uuid },
    /// Similarity 0.5–0.8 with tag overlap — bump confidence.
    Reinforces { existing_id: Uuid },
    /// Similarity 0.5–0.8 with contradictory signal — flag for review.
    Contradicts { existing_id: Uuid },
    /// Similarity < 0.4 — insert as new.
    New,
}

/// Row from decision_patterns table.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DecisionPatternResponse {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Option<Uuid>,
    pub domain: String,
    pub pattern: String,
    pub rationale: String,
    pub tags: Vec<String>,
    pub confidence: f32,
    pub usage_count: i32,
    pub override_count: i32,
    pub source_story_id: Option<Uuid>,
    pub source_round_id: Option<Uuid>,
    pub superseded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListPatternsParams {
    pub domain: Option<String>,
    pub min_confidence: Option<f32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePatternRequest {
    pub pattern: Option<String>,
    pub rationale: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SupersedePatternRequest {
    pub pattern: String,
    pub rationale: String,
    pub tags: Option<Vec<String>>,
}
