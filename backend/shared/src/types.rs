use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::KnowledgeCategory;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QuestionOption {
    pub label: String,
    pub pros: String,
    pub cons: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Question {
    pub id: Uuid,
    pub text: String,
    pub domain: String,
    pub rationale: String,
    pub options: Vec<QuestionOption>,
    pub recommended_option_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Answer {
    pub question_id: Uuid,
    pub selected_answer_index: Option<i32>,
    pub selected_answer_text: String,
    pub answered_by: Uuid,
    pub answered_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QaDecision {
    pub question_text: String,
    pub answer_text: String,
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KnowledgeEntry {
    pub title: String,
    pub content: String,
    pub category: KnowledgeCategory,
}
