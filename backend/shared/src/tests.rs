use chrono::Utc;
use uuid::Uuid;

use crate::{
    enums::*,
    messages::*,
    types::*,
};

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

fn make_qa_round() -> QaRoundContent {
    QaRoundContent {
        questions: vec![Question {
            id: Uuid::now_v7(),
            text: "Should we use REST?".into(),
            domain: "architecture".into(),
            options: vec!["yes".into(), "no".into()],
        }],
    }
}

// ── ServerToContainer round-trips ─────────────────────────────────────────────

#[test]
fn server_to_container_start_grooming() {
    let msg = ServerToContainer::StartGrooming {
        story_id: Uuid::now_v7(),
        context: GroomingContext {
            story_description: "As a user I want...".into(),
            knowledge: make_knowledge(),
            codebase_context: "rust monorepo".into(),
        },
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerToContainer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn server_to_container_start_planning() {
    let msg = ServerToContainer::StartPlanning {
        story_id: Uuid::now_v7(),
        context: PlanningContext {
            story_description: "plan this".into(),
            grooming_decisions: vec![QaDecision {
                question_text: "REST?".into(),
                answer_text: "yes".into(),
                domain: "arch".into(),
            }],
            knowledge: make_knowledge(),
            codebase_context: "monorepo".into(),
        },
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerToContainer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn server_to_container_answer_received() {
    let msg = ServerToContainer::AnswerReceived {
        round_id: Uuid::now_v7(),
        answers: vec![make_answer()],
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerToContainer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn server_to_container_start_task() {
    let msg = ServerToContainer::StartTask {
        story_id: Uuid::now_v7(),
        task_id: Uuid::now_v7(),
        session_id: "sess-abc".into(),
        context: TaskContext {
            task_description: "implement auth".into(),
            story_decisions: vec![],
            sibling_decisions: vec![],
            knowledge: make_knowledge(),
        },
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerToContainer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn server_to_container_resume_task() {
    let msg = ServerToContainer::ResumeTask {
        task_id: Uuid::now_v7(),
        session_id: "sess-xyz".into(),
        answer: make_answer(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerToContainer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn server_to_container_cancel_task() {
    let msg = ServerToContainer::CancelTask { task_id: Uuid::now_v7() };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerToContainer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn server_to_container_ping() {
    let msg = ServerToContainer::Ping;
    let json = serde_json::to_string(&msg).unwrap();
    let back: ServerToContainer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

// ── ContainerToServer round-trips ─────────────────────────────────────────────

#[test]
fn container_to_server_question_batch() {
    let msg = ContainerToServer::QuestionBatch {
        story_id: Uuid::now_v7(),
        task_id: Some(Uuid::now_v7()),
        round: make_qa_round(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn container_to_server_question_batch_no_task() {
    let msg = ContainerToServer::QuestionBatch {
        story_id: Uuid::now_v7(),
        task_id: None,
        round: make_qa_round(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn container_to_server_task_decomposition() {
    let msg = ContainerToServer::TaskDecomposition {
        story_id: Uuid::now_v7(),
        proposed_tasks: vec![ProposedTask {
            name: "write tests".into(),
            description: "tdd".into(),
            task_type: TaskType::Test,
            position: 0,
            depends_on: vec![],
        }],
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn container_to_server_task_paused() {
    let msg = ContainerToServer::TaskPaused {
        task_id: Uuid::now_v7(),
        question: PauseQuestion {
            text: "Which db?".into(),
            domain: "infra".into(),
            options: vec!["postgres".into(), "sqlite".into()],
        },
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn container_to_server_task_completed() {
    let msg = ContainerToServer::TaskCompleted {
        task_id: Uuid::now_v7(),
        commit_sha: "abc1234".into(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn container_to_server_task_failed() {
    let msg = ContainerToServer::TaskFailed {
        task_id: Uuid::now_v7(),
        error: "timeout".into(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn container_to_server_status_update() {
    let msg = ContainerToServer::StatusUpdate {
        task_id: Uuid::now_v7(),
        status_text: "running tests".into(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}

#[test]
fn container_to_server_pong() {
    let msg = ContainerToServer::Pong;
    let json = serde_json::to_string(&msg).unwrap();
    let back: ContainerToServer = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&msg).unwrap(), serde_json::to_value(&back).unwrap());
}
