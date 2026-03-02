use uuid::Uuid;

use crate::{db::OrgTx, errors::ApiError};

use super::{
    repo,
    types::{
        CourseCorrectionRequest, ListQaRoundsParams, QaContent, QaError, QaRoundResponse,
        SubmitAnswerRequest, VALID_STAGES,
    },
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn to_response(row: repo::QaRoundRow) -> Result<QaRoundResponse, ApiError> {
    let content: QaContent = serde_json::from_value(row.content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to parse QA content: {e}")))?;
    Ok(QaRoundResponse {
        id: row.id,
        org_id: row.org_id,
        story_id: row.story_id,
        task_id: row.task_id,
        stage: row.stage,
        round_number: row.round_number,
        status: row.status,
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

    let org_id = tx.auth.org_id;
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
    round_id: Uuid,
    req: SubmitAnswerRequest,
) -> Result<QaRoundResponse, ApiError> {
    let org_id = tx.auth.org_id;
    let user_id = tx.auth.user_id;

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
        .find(|q| q.id == req.question_id)
        .ok_or(QaError::QuestionNotFound)?;

    if question.selected_answer_index.is_some() || question.selected_answer_text.is_some() {
        return Err(QaError::AlreadyAnswered.into());
    }

    question.selected_answer_index = req.selected_answer_index;
    question.selected_answer_text = Some(req.answer_text);
    question.answered_by = Some(user_id);
    question.answered_at = Some(chrono::Utc::now());

    let new_content = serde_json::to_value(&content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("Failed to serialize QA content: {e}")))?;

    let updated = repo::update_round_content(tx, round_id, org_id, &new_content)
        .await?
        .ok_or(QaError::NotFound)?;

    to_response(updated)
}

pub async fn rollback(tx: &mut OrgTx, round_id: Uuid) -> Result<QaRoundResponse, ApiError> {
    let org_id = tx.auth.org_id;

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

    let org_id = tx.auth.org_id;

    let max_round = repo::get_max_round_number(tx, req.story_id, req.task_id, &req.stage)
        .await?
        .unwrap_or(0);
    let next_round = max_round + 1;

    let content = QaContent {
        questions: Vec::new(),
        course_correction: Some(req.text),
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
