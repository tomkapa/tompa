use chrono::Utc;
use uuid::Uuid;

use crate::{enums::*, messages::*, types::*};

fn round_trip<T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq>(
    value: &T,
) {
    let json = serde_json::to_string(value).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        serde_json::to_value(&back).unwrap()
    );
}

// ── Enum round-trips ──────────────────────────────────────────────────────────

#[test]
fn story_type_variants() {
    round_trip(&StoryType::Feature);
    round_trip(&StoryType::Bug);
    round_trip(&StoryType::Refactor);
}

#[test]
fn story_status_variants() {
    round_trip(&StoryStatus::Todo);
    round_trip(&StoryStatus::InProgress);
    round_trip(&StoryStatus::Done);
}

#[test]
fn pipeline_stage_variants() {
    round_trip(&PipelineStage::Grooming);
    round_trip(&PipelineStage::Planning);
    round_trip(&PipelineStage::Decomposition);
    round_trip(&PipelineStage::Implementation);
    round_trip(&PipelineStage::Testing);
    round_trip(&PipelineStage::Review);
}

#[test]
fn task_type_variants() {
    round_trip(&TaskType::Design);
    round_trip(&TaskType::Test);
    round_trip(&TaskType::Code);
}

#[test]
fn task_state_variants() {
    round_trip(&TaskState::Pending);
    round_trip(&TaskState::Qa);
    round_trip(&TaskState::Running);
    round_trip(&TaskState::Paused);
    round_trip(&TaskState::Blocked);
    round_trip(&TaskState::Done);
}

#[test]
fn qa_stage_variants() {
    round_trip(&QaStage::Grooming);
    round_trip(&QaStage::Planning);
    round_trip(&QaStage::TaskQa);
    round_trip(&QaStage::Implementation);
}

#[test]
fn qa_round_status_variants() {
    round_trip(&QaRoundStatus::Active);
    round_trip(&QaRoundStatus::Superseded);
}

#[test]
fn container_mode_variants() {
    round_trip(&ContainerMode::Project);
    round_trip(&ContainerMode::Dev);
    round_trip(&ContainerMode::Standalone);
}

#[test]
fn org_role_variants() {
    round_trip(&OrgRole::Owner);
    round_trip(&OrgRole::Admin);
    round_trip(&OrgRole::Member);
}

#[test]
fn knowledge_category_variants() {
    round_trip(&KnowledgeCategory::Convention);
    round_trip(&KnowledgeCategory::Adr);
    round_trip(&KnowledgeCategory::ApiDoc);
    round_trip(&KnowledgeCategory::DesignSystem);
    round_trip(&KnowledgeCategory::Custom);
}

// ── Helper builders ───────────────────────────────────────────────────────────

fn make_knowledge() -> Vec<KnowledgeEntry> {
    vec![KnowledgeEntry {
        title: "style guide".into(),
        content: "use 4-space indents".into(),
        category: KnowledgeCategory::Convention,
    }]
}

fn make_answer() -> Answer {
    Answer {
        question_id: Uuid::now_v7(),
        selected_answer_index: Some(0),
        selected_answer_text: "yes".into(),
        answered_by: Uuid::now_v7(),
        answered_at: Utc::now(),
    }
}

fn make_question_option(label: &str) -> QuestionOption {
    QuestionOption {
        label: label.into(),
        pros: format!("Pros of {label}."),
        cons: format!("Cons of {label}."),
    }
}

// ── ServerToContainer round-trips ─────────────────────────────────────────────

#[test]
fn server_to_container_execute() {
    let msg = ServerToContainer::Execute {
        session_id: Uuid::now_v7(),
        system_prompt: "You are a helpful assistant.".into(),
        prompt: "Generate QA questions for this story.".into(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerToContainer = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&msg).unwrap(),
        serde_json::to_value(&back).unwrap()
    );
}

#[test]
fn server_to_container_ping() {
    let msg = ServerToContainer::Ping;
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerToContainer = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&msg).unwrap(),
        serde_json::to_value(&back).unwrap()
    );
}

// ── ContainerToServer round-trips ─────────────────────────────────────────────

#[test]
fn container_to_server_execution_result() {
    let output = serde_json::json!({
        "questions": [
            {
                "id": Uuid::now_v7().to_string(),
                "text": "Should we use REST?",
                "domain": "architecture",
                "rationale": "Affects the entire API surface.",
                "options": [
                    {"label": "yes", "pros": "Simple.", "cons": "Limited."},
                    {"label": "no", "pros": "Flexible.", "cons": "Complex."}
                ],
                "recommended_option_index": 0
            }
        ]
    });
    let msg = ContainerToServer::ExecutionResult {
        session_id: Uuid::now_v7(),
        output,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&msg).unwrap(),
        serde_json::to_value(&back).unwrap()
    );
}

#[test]
fn container_to_server_execution_failed() {
    let msg = ContainerToServer::ExecutionFailed {
        session_id: Uuid::now_v7(),
        error: "timeout after 300s".into(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&msg).unwrap(),
        serde_json::to_value(&back).unwrap()
    );
}

#[test]
fn container_to_server_pong() {
    let msg = ContainerToServer::Pong;
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&msg).unwrap(),
        serde_json::to_value(&back).unwrap()
    );
}

// ── Kept types round-trips ────────────────────────────────────────────────────

#[test]
fn answer_round_trip() {
    let a = make_answer();
    let json = serde_json::to_string(&a).unwrap();
    let back: Answer = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(&back).unwrap()
    );
}

#[test]
fn question_option_round_trip() {
    let opt = make_question_option("postgres");
    round_trip(&opt);
}

#[test]
fn qa_decision_round_trip() {
    let d = QaDecision {
        question_text: "REST?".into(),
        answer_text: "yes".into(),
        domain: "arch".into(),
    };
    let json = serde_json::to_string(&d).unwrap();
    let back: QaDecision = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&d).unwrap(),
        serde_json::to_value(&back).unwrap()
    );
}

#[test]
fn knowledge_entry_round_trip() {
    let entries = make_knowledge();
    let json = serde_json::to_string(&entries).unwrap();
    let back: Vec<KnowledgeEntry> = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&entries).unwrap(),
        serde_json::to_value(&back).unwrap()
    );
}
