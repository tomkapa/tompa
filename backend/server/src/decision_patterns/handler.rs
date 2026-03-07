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
    types::{
        DecisionPatternResponse, ListPatternsParams, SupersedePatternRequest, UpdatePatternRequest,
    },
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/projects/{project_id}/patterns",
            get(list_patterns),
        )
        .route(
            "/api/v1/projects/{project_id}/patterns/{pattern_id}",
            get(get_pattern).patch(update_pattern),
        )
        .route(
            "/api/v1/projects/{project_id}/patterns/{pattern_id}/retire",
            post(retire_pattern),
        )
        .route(
            "/api/v1/projects/{project_id}/patterns/{pattern_id}/supersede",
            post(supersede_pattern),
        )
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// GET /api/v1/projects/:project_id/patterns
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/patterns",
    tag = "decision-patterns",
    params(
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("domain" = Option<String>, Query, description = "Filter by domain"),
        ("min_confidence" = Option<f32>, Query, description = "Minimum confidence threshold"),
    ),
    responses(
        (status = 200, description = "List of decision patterns", body = [DecisionPatternResponse]),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn list_patterns(
    mut tx: OrgTx,
    Path(project_id): Path<Uuid>,
    Query(params): Query<ListPatternsParams>,
) -> Result<Json<Vec<DecisionPatternResponse>>, ApiError> {
    let patterns =
        service::list_patterns(&mut tx, project_id, params.domain.as_deref(), params.min_confidence)
            .await?;
    tx.commit().await?;
    Ok(Json(patterns))
}

/// GET /api/v1/projects/:project_id/patterns/:pattern_id
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/patterns/{pattern_id}",
    tag = "decision-patterns",
    params(
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID"),
    ),
    responses(
        (status = 200, description = "Decision pattern detail", body = DecisionPatternResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Pattern not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn get_pattern(
    mut tx: OrgTx,
    Path((_project_id, pattern_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DecisionPatternResponse>, ApiError> {
    let pattern = service::get_pattern(&mut tx, pattern_id).await?;
    tx.commit().await?;
    Ok(Json(pattern))
}

/// PATCH /api/v1/projects/:project_id/patterns/:pattern_id
#[utoipa::path(
    patch,
    path = "/api/v1/projects/{project_id}/patterns/{pattern_id}",
    tag = "decision-patterns",
    params(
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID"),
    ),
    request_body = UpdatePatternRequest,
    responses(
        (status = 200, description = "Updated pattern", body = DecisionPatternResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Pattern not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn update_pattern(
    mut tx: OrgTx,
    Path((_project_id, pattern_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdatePatternRequest>,
) -> Result<Json<DecisionPatternResponse>, ApiError> {
    let pattern = service::update_pattern(&mut tx, pattern_id, req).await?;
    tx.commit().await?;
    Ok(Json(pattern))
}

/// POST /api/v1/projects/:project_id/patterns/:pattern_id/retire
#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_id}/patterns/{pattern_id}/retire",
    tag = "decision-patterns",
    params(
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID"),
    ),
    responses(
        (status = 204, description = "Pattern retired"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Pattern not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn retire_pattern(
    mut tx: OrgTx,
    Path((_project_id, pattern_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    service::retire_pattern(&mut tx, pattern_id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/projects/:project_id/patterns/:pattern_id/supersede
#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_id}/patterns/{pattern_id}/supersede",
    tag = "decision-patterns",
    params(
        ("project_id" = Uuid, Path, description = "Project ID"),
        ("pattern_id" = Uuid, Path, description = "Pattern ID"),
    ),
    request_body = SupersedePatternRequest,
    responses(
        (status = 201, description = "New replacement pattern created", body = DecisionPatternResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Pattern not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn supersede_pattern(
    mut tx: OrgTx,
    Path((_project_id, pattern_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<SupersedePatternRequest>,
) -> Result<(StatusCode, Json<DecisionPatternResponse>), ApiError> {
    let pattern = service::supersede_pattern(&mut tx, pattern_id, req).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(pattern)))
}
