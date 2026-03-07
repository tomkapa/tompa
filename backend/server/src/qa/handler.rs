use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
};
use uuid::Uuid;

use crate::{
    agents, auth::middleware::require_auth, db::OrgTx,
    decision_patterns::service as dp_service, errors::ApiError,
    sse::broadcaster::SseEvent, state::AppState,
};

use super::{
    service,
    types::{
        AssignQuestionRequest, CourseCorrectionRequest, ListQaRoundsParams, QaRoundResponse,
        SubmitAnswerRequest,
    },
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/qa-rounds", get(list_rounds))
        // Static path must be registered before the parameterized :id routes.
        .route("/api/v1/qa-rounds/course-correct", post(course_correct))
        .route("/api/v1/qa-rounds/{id}/answer", post(submit_answer))
        .route("/api/v1/qa-rounds/{id}/rollback", post(rollback))
        .route(
            "/api/v1/qa-rounds/{round_id}/questions/{question_id}/assignee",
            put(assign_question).delete(unassign_question),
        )
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// GET /api/v1/qa-rounds?story_id=X[&task_id=Y][&stage=Z]
#[utoipa::path(
    get,
    path = "/api/v1/qa-rounds",
    tag = "qa",
    params(
        ("story_id" = Option<Uuid>, Query, description = "Filter by story ID"),
        ("task_id" = Option<Uuid>, Query, description = "Filter by task ID"),
        ("stage" = Option<String>, Query, description = "Filter by stage: grooming, planning, task_qa, implementation"),
    ),
    responses(
        (status = 200, description = "List of Q&A rounds", body = [QaRoundResponse]),
        (status = 400, description = "story_id or task_id is required"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn list_rounds(
    mut tx: OrgTx,
    Query(params): Query<ListQaRoundsParams>,
) -> Result<Json<Vec<QaRoundResponse>>, ApiError> {
    let rounds = service::list_rounds(&mut tx, params).await?;
    tx.commit().await?;
    Ok(Json(rounds))
}

/// POST /api/v1/qa-rounds/:id/answer — submit an answer for a question in a round.
#[utoipa::path(
    post,
    path = "/api/v1/qa-rounds/{id}/answer",
    tag = "qa",
    params(
        ("id" = Uuid, Path, description = "QA round ID"),
    ),
    request_body = SubmitAnswerRequest,
    responses(
        (status = 200, description = "Updated QA round with the answer recorded", body = QaRoundResponse),
        (status = 400, description = "Round not active or question already answered"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Round or question not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn submit_answer(
    State(state): State<AppState>,
    auth: axum::Extension<crate::auth::types::AuthContext>,
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
    Json(req): Json<SubmitAnswerRequest>,
) -> Result<Json<QaRoundResponse>, ApiError> {
    let org_id = tx.org_id;
    let result = service::submit_answer(&mut tx, auth.user_id, id, req).await?;
    tx.commit().await?;

    if let Some(payload) = result.notify {
        let s = state.clone();
        let answer_pairs: Vec<(String, String)> = result
            .response
            .content
            .questions
            .iter()
            .filter_map(|q| {
                q.selected_answer_text
                    .as_ref()
                    .map(|a| (q.text.clone(), a.clone()))
            })
            .collect();
        let project_id = payload.project_id;

        tokio::spawn(async move {
            s.broadcaster.broadcast(
                org_id,
                SseEvent::AnswersForwarded {
                    story_id: payload.story_id,
                    task_id: payload.task_id,
                    round_id: payload.round_id,
                },
            );

            // Feedback loop: update pattern confidence based on answer alignment
            dp_service::process_answer_feedback(
                &s.pool,
                org_id,
                project_id,
                &answer_pairs,
            )
            .await;

            agents::service::dispatch_next_round(
                &s,
                org_id,
                payload.project_id,
                payload.story_id,
                &payload.stage,
                payload.task_id,
            )
            .await;
        });
    }

    Ok(Json(result.response))
}

/// POST /api/v1/qa-rounds/:id/rollback — checkpoint rollback to this round.
#[utoipa::path(
    post,
    path = "/api/v1/qa-rounds/{id}/rollback",
    tag = "qa",
    params(
        ("id" = Uuid, Path, description = "QA round ID to roll back to"),
    ),
    responses(
        (status = 200, description = "Round marked as rollback point; subsequent rounds superseded", body = QaRoundResponse),
        (status = 400, description = "Invalid rollback"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Round not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn rollback(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<Json<QaRoundResponse>, ApiError> {
    let round = service::rollback(&mut tx, id).await?;
    tx.commit().await?;
    Ok(Json(round))
}

/// PUT /api/v1/qa-rounds/:round_id/questions/:question_id/assignee — assign a question to an org member.
#[utoipa::path(
    put,
    path = "/api/v1/qa-rounds/{round_id}/questions/{question_id}/assignee",
    tag = "qa",
    params(
        ("round_id" = Uuid, Path, description = "QA round ID"),
        ("question_id" = Uuid, Path, description = "Question ID"),
    ),
    request_body = AssignQuestionRequest,
    responses(
        (status = 200, description = "Updated QA round with assignment recorded", body = QaRoundResponse),
        (status = 400, description = "Round not active or member not in org"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Round or question not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn assign_question(
    State(state): State<AppState>,
    auth: axum::Extension<crate::auth::types::AuthContext>,
    mut tx: OrgTx,
    Path((round_id, question_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<AssignQuestionRequest>,
) -> Result<Json<QaRoundResponse>, ApiError> {
    let org_id = tx.org_id;
    let result =
        service::assign_question(&mut tx, round_id, question_id, req.member_id, auth.user_id)
            .await?;
    tx.commit().await?;

    let s = state.clone();
    let sse = result.sse;
    tokio::spawn(async move {
        s.broadcaster.broadcast(
            org_id,
            SseEvent::QuestionAssigned {
                story_id: sse.story_id,
                task_id: sse.task_id,
                round_id: sse.round_id,
                question_id: sse.question_id,
                assigned_to: sse.assigned_to,
                assigned_by: sse.assigned_by,
                question_text_preview: sse.question_text_preview,
            },
        );
    });

    Ok(Json(result.response))
}

/// DELETE /api/v1/qa-rounds/:round_id/questions/:question_id/assignee — remove assignment from a question.
#[utoipa::path(
    delete,
    path = "/api/v1/qa-rounds/{round_id}/questions/{question_id}/assignee",
    tag = "qa",
    params(
        ("round_id" = Uuid, Path, description = "QA round ID"),
        ("question_id" = Uuid, Path, description = "Question ID"),
    ),
    responses(
        (status = 200, description = "Updated QA round with assignment cleared", body = QaRoundResponse),
        (status = 400, description = "Round not active"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Round or question not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn unassign_question(
    mut tx: OrgTx,
    Path((round_id, question_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<QaRoundResponse>, ApiError> {
    let round = service::unassign_question(&mut tx, round_id, question_id).await?;
    tx.commit().await?;
    Ok(Json(round))
}

/// POST /api/v1/qa-rounds/course-correct — free-form course correction.
#[utoipa::path(
    post,
    path = "/api/v1/qa-rounds/course-correct",
    tag = "qa",
    request_body = CourseCorrectionRequest,
    responses(
        (status = 201, description = "Course correction round created", body = QaRoundResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn course_correct(
    mut tx: OrgTx,
    Json(req): Json<CourseCorrectionRequest>,
) -> Result<(StatusCode, Json<QaRoundResponse>), ApiError> {
    let round = service::course_correct(&mut tx, req).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(round)))
}
