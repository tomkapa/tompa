use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{
    Answer, AnswerContext, GroomingContext, PauseQuestion, PlanningContext, ProposedTask,
    QaRoundContent, TaskContext,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ServerToContainer {
    StartGrooming {
        story_id: Uuid,
        context: GroomingContext,
    },
    StartPlanning {
        story_id: Uuid,
        context: PlanningContext,
    },
    AnswerReceived {
        round_id: Uuid,
        answers: Vec<Answer>,
        context: AnswerContext,
    },
    StartTask {
        story_id: Uuid,
        task_id: Uuid,
        session_id: String,
        context: TaskContext,
    },
    ResumeTask {
        task_id: Uuid,
        session_id: String,
        answer: Answer,
    },
    CancelTask {
        task_id: Uuid,
    },
    DescriptionApproved {
        story_id: Uuid,
        stage: String,
        description: String,
    },
    Ping,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ContainerToServer {
    QuestionBatch {
        story_id: Uuid,
        task_id: Option<Uuid>,
        round: QaRoundContent,
    },
    TaskDecomposition {
        story_id: Uuid,
        proposed_tasks: Vec<ProposedTask>,
    },
    TaskPaused {
        task_id: Uuid,
        question: PauseQuestion,
    },
    TaskCompleted {
        task_id: Uuid,
        commit_sha: String,
    },
    TaskFailed {
        task_id: Uuid,
        error: String,
    },
    RefinedDescription {
        story_id: Uuid,
        stage: String,
        refined_description: String,
    },
    StatusUpdate {
        task_id: Uuid,
        status_text: String,
    },
    Pong,
}
