use uuid::Uuid;

use shared::types::Answer;

use crate::{db::OrgTx, errors::ApiError, story::repo as story_repo};

use super::{
    repo,
    types::{
        CourseCorrectionRequest, ListQaRoundsParams, QaContent, QaError, QaRoundResponse,
        SubmitAnswerRequest, VALID_STAGES,
    },
};

// ── Assignment result types ───────────────────────────────────────────────────

pub struct QuestionAssignedPayload {
    pub story_id: Uuid,
    pub task_id: Option<Uuid>,
    pub round_id: Uuid,
    pub question_id: Uuid,
    pub assigned_to: Uuid,
    pub assigned_by: Uuid,
    pub question_text_preview: String,
}

pub struct AssignQuestionResult {
    pub response: QaRoundResponse,
    pub sse: QuestionAssignedPayload,
}

// ── Submit-answer result types ──────────────────────────────────────────────

/// Payload returned when all questions in a round have been answered.
pub struct AllAnsweredPayload {
    pub project_id: Uuid,
    pub round_id: Uuid,
    pub story_id: Uuid,
    pub task_id: Option<Uuid>,
    pub stage: String,
    pub answers: Vec<Answer>,
}

/// Result of `submit_answer`, carrying the API response and an optional
/// notification payload when the last question in the round was just answered.
pub struct SubmitAnswerResult {
    pub response: QaRoundResponse,
    pub notify: Option<AllAnsweredPayload>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn to_response(row: repo::QaRoundRow) -> Result<QaRoundResponse, ApiError> {
    let content: QaContent = serde_json::from_value(row.content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse QA content: {e}")))?;
    let applied_pattern_count = content.applied_patterns.len();
    let applied_patterns = content.applied_patterns.clone();
    Ok(QaRoundResponse {
        id: row.id,
        org_id: row.org_id,
        story_id: row.story_id,
        task_id: row.task_id,
        stage: row.stage,
        round_number: row.round_number,
        status: row.status,
        applied_pattern_count,
        applied_patterns,
        content,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

// ── Public service functions ──────────────────────────────────────────────────

pub async fn list_rounds(
    tx: &mut OrgTx,
    params: ListQaRoundsParams,
) -> Result<Vec<QaRoundResponse>, ApiError> {
    if params.story_id.is_none() && params.task_id.is_none() {
        return Err(QaError::MissingFilter.into());
    }

    let org_id = tx.org_id;
    let rows = repo::list_rounds(
        tx,
        org_id,
        params.story_id,
        params.task_id,
        params.stage.as_deref(),
    )
    .await?;

    rows.into_iter().map(to_response).collect()
}

pub async fn submit_answer(
    tx: &mut OrgTx,
    user_id: Uuid,
    round_id: Uuid,
    req: SubmitAnswerRequest,
) -> Result<SubmitAnswerResult, ApiError> {
    let org_id = tx.org_id;

    let row = repo::get_round(tx, round_id, org_id)
        .await?
        .ok_or(QaError::NotFound)?;

    if row.status != "active" {
        return Err(QaError::RoundNotActive.into());
    }

    let story_id = row.story_id;
    let task_id = row.task_id;
    let stage = row.stage.clone();

    let mut content: QaContent = serde_json::from_value(row.content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse QA content: {e}")))?;

    let question = content
        .questions
        .iter_mut()
        .find(|q| q.id == req.question_id)
        .ok_or(QaError::QuestionNotFound)?;

    question.selected_answer_index = req.selected_answer_index;
    question.selected_answer_text = Some(req.answer_text);
    question.answered_by = Some(user_id);
    question.answered_at = Some(chrono::Utc::now());
    question.assigned_to = Some(user_id);

    let new_content = serde_json::to_value(&content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to serialize QA content: {e}")))?;

    let updated = repo::update_round_content(tx, round_id, org_id, &new_content)
        .await?
        .ok_or(QaError::NotFound)?;

    // Check if all questions are now answered.
    let all_answered = content
        .questions
        .iter()
        .all(|q| q.selected_answer_text.is_some());

    let notify = if all_answered {
        let story = story_repo::get_story(tx, story_id, org_id)
            .await?
            .ok_or(QaError::NotFound)?;

        let answers: Vec<Answer> = content
            .questions
            .iter()
            .filter_map(|q| {
                Some(Answer {
                    question_id: q.id,
                    selected_answer_index: q.selected_answer_index,
                    selected_answer_text: q.selected_answer_text.clone()?,
                    answered_by: q.answered_by?,
                    answered_at: q.answered_at?,
                })
            })
            .collect();

        Some(AllAnsweredPayload {
            project_id: story.project_id,
            round_id,
            story_id,
            task_id,
            stage: stage.clone(),
            answers,
        })
    } else {
        None
    };

    Ok(SubmitAnswerResult {
        response: to_response(updated)?,
        notify,
    })
}

pub async fn rollback(tx: &mut OrgTx, round_id: Uuid) -> Result<QaRoundResponse, ApiError> {
    let org_id = tx.org_id;

    let row = repo::get_round(tx, round_id, org_id)
        .await?
        .ok_or(QaError::NotFound)?;

    if row.status != "active" {
        return Err(QaError::InvalidRollback.into());
    }

    // Supersede all rounds with a higher round_number in the same scope.
    repo::supersede_rounds_after(tx, row.story_id, row.task_id, &row.stage, row.round_number)
        .await?;

    // Clear all answers in the target round so it can be re-answered.
    let mut content: QaContent = serde_json::from_value(row.content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse QA content: {e}")))?;

    for q in content.questions.iter_mut() {
        q.selected_answer_index = None;
        q.selected_answer_text = None;
        q.answered_by = None;
        q.answered_at = None;
    }

    let new_content = serde_json::to_value(&content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to serialize QA content: {e}")))?;

    let updated = repo::update_round_content(tx, round_id, org_id, &new_content)
        .await?
        .ok_or(QaError::NotFound)?;

    to_response(updated)
}

pub async fn assign_question(
    tx: &mut OrgTx,
    round_id: Uuid,
    question_id: Uuid,
    member_id: Uuid,
    assigned_by: Uuid,
) -> Result<AssignQuestionResult, ApiError> {
    let org_id = tx.org_id;

    let row = repo::get_round(tx, round_id, org_id)
        .await?
        .ok_or(QaError::NotFound)?;

    if row.status != "active" {
        return Err(QaError::RoundNotActive.into());
    }

    if !repo::is_org_member(tx, org_id, member_id).await? {
        return Err(QaError::InvalidAssignee.into());
    }

    let story_id = row.story_id;
    let task_id = row.task_id;

    let mut content: QaContent = serde_json::from_value(row.content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse QA content: {e}")))?;

    let question = content
        .questions
        .iter_mut()
        .find(|q| q.id == question_id)
        .ok_or(QaError::QuestionNotFound)?;

    let preview: String = question.text.chars().take(100).collect();
    question.assigned_to = Some(member_id);

    let new_content = serde_json::to_value(&content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to serialize QA content: {e}")))?;

    let updated = repo::update_round_content(tx, round_id, org_id, &new_content)
        .await?
        .ok_or(QaError::NotFound)?;

    Ok(AssignQuestionResult {
        response: to_response(updated)?,
        sse: QuestionAssignedPayload {
            story_id,
            task_id,
            round_id,
            question_id,
            assigned_to: member_id,
            assigned_by,
            question_text_preview: preview,
        },
    })
}

pub async fn unassign_question(
    tx: &mut OrgTx,
    round_id: Uuid,
    question_id: Uuid,
) -> Result<QaRoundResponse, ApiError> {
    let org_id = tx.org_id;

    let row = repo::get_round(tx, round_id, org_id)
        .await?
        .ok_or(QaError::NotFound)?;

    if row.status != "active" {
        return Err(QaError::RoundNotActive.into());
    }

    let mut content: QaContent = serde_json::from_value(row.content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse QA content: {e}")))?;

    let question = content
        .questions
        .iter_mut()
        .find(|q| q.id == question_id)
        .ok_or(QaError::QuestionNotFound)?;

    question.assigned_to = None;

    let new_content = serde_json::to_value(&content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to serialize QA content: {e}")))?;

    let updated = repo::update_round_content(tx, round_id, org_id, &new_content)
        .await?
        .ok_or(QaError::NotFound)?;

    to_response(updated)
}

pub async fn course_correct(
    tx: &mut OrgTx,
    req: CourseCorrectionRequest,
) -> Result<QaRoundResponse, ApiError> {
    if !VALID_STAGES.contains(&req.stage.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "invalid stage '{}'; must be one of: {}",
            req.stage,
            VALID_STAGES.join(", ")
        )));
    }

    let org_id = tx.org_id;

    let max_round = repo::get_max_round_number(tx, req.story_id, req.task_id, &req.stage)
        .await?
        .unwrap_or(0);
    let next_round = max_round + 1;

    let content = QaContent {
        questions: Vec::new(),
        course_correction: Some(req.text),
        applied_patterns: Vec::new(),
    };
    let content_value = serde_json::to_value(&content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to serialize QA content: {e}")))?;

    let row = repo::create_round(
        tx,
        org_id,
        req.story_id,
        req.task_id,
        &req.stage,
        next_round,
        &content_value,
    )
    .await?;

    to_response(row)
}
