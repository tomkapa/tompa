use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
};
use uuid::Uuid;

use crate::{auth::middleware::require_auth, db::OrgTx, errors::ApiError, state::AppState};

use super::{
    service,
    types::{CourseCorrectionRequest, ListQaRoundsParams, QaRoundResponse, SubmitAnswerRequest},
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/qa-rounds", get(list_rounds))
        // Static path must be registered before the parameterized :id routes.
        .route("/api/v1/qa-rounds/course-correct", post(course_correct))
        .route("/api/v1/qa-rounds/{id}/answer", post(submit_answer))
        .route("/api/v1/qa-rounds/{id}/rollback", post(rollback))
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
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
    Json(req): Json<SubmitAnswerRequest>,
) -> Result<Json<QaRoundResponse>, ApiError> {
    let round = service::submit_answer(&mut tx, id, req).await?;
    tx.commit().await?;
    Ok(Json(round))
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
