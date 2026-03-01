use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::enums::{KnowledgeCategory, TaskType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GroomingContext {
    pub story_description: String,
    pub knowledge: Vec<KnowledgeEntry>,
    pub codebase_context: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanningContext {
    pub story_description: String,
    pub grooming_decisions: Vec<QaDecision>,
    pub knowledge: Vec<KnowledgeEntry>,
    pub codebase_context: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskContext {
    pub task_description: String,
    pub story_decisions: Vec<QaDecision>,
    pub sibling_decisions: Vec<QaDecision>,
    pub knowledge: Vec<KnowledgeEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QaRoundContent {
    pub questions: Vec<Question>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Question {
    pub id: Uuid,
    pub text: String,
    pub domain: String,
    pub options: Vec<String>,
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
pub struct ProposedTask {
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub position: i32,
    pub depends_on: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PauseQuestion {
    pub text: String,
    pub domain: String,
    pub options: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KnowledgeEntry {
    pub title: String,
    pub content: String,
    pub category: KnowledgeCategory,
}
