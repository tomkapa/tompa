use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, patch},
};
use uuid::Uuid;

use crate::{auth::middleware::require_auth, db::OrgTx, errors::ApiError, state::AppState};

use super::{
    service,
    types::{
        CreateKnowledgeRequest, KnowledgeResponse, ListKnowledgeParams, UpdateKnowledgeRequest,
    },
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/knowledge",
            get(list_knowledge).post(create_knowledge),
        )
        .route(
            "/api/v1/knowledge/{id}",
            patch(update_knowledge).delete(delete_knowledge),
        )
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// GET /api/v1/knowledge?project_id=X[&story_id=Y]
#[utoipa::path(
    get,
    path = "/api/v1/knowledge",
    tag = "knowledge",
    params(
        ("project_id" = Option<Uuid>, Query, description = "Filter by project ID"),
        ("story_id" = Option<Uuid>, Query, description = "Filter by story ID"),
    ),
    responses(
        (status = 200, description = "List of knowledge entries", body = [KnowledgeResponse]),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn list_knowledge(
    mut tx: OrgTx,
    Query(params): Query<ListKnowledgeParams>,
) -> Result<Json<Vec<KnowledgeResponse>>, ApiError> {
    let entries = service::list_knowledge(&mut tx, params.project_id, params.story_id).await?;
    tx.commit().await?;
    Ok(Json(entries))
}

/// POST /api/v1/knowledge
#[utoipa::path(
    post,
    path = "/api/v1/knowledge",
    tag = "knowledge",
    request_body = CreateKnowledgeRequest,
    responses(
        (status = 201, description = "Knowledge entry created", body = KnowledgeResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn create_knowledge(
    mut tx: OrgTx,
    Json(req): Json<CreateKnowledgeRequest>,
) -> Result<(StatusCode, Json<KnowledgeResponse>), ApiError> {
    let entry = service::create_knowledge(&mut tx, req).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

/// PATCH /api/v1/knowledge/:id
#[utoipa::path(
    patch,
    path = "/api/v1/knowledge/{id}",
    tag = "knowledge",
    params(
        ("id" = Uuid, Path, description = "Knowledge entry ID"),
    ),
    request_body = UpdateKnowledgeRequest,
    responses(
        (status = 200, description = "Updated knowledge entry", body = KnowledgeResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Entry not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn update_knowledge(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateKnowledgeRequest>,
) -> Result<Json<KnowledgeResponse>, ApiError> {
    let entry = service::update_knowledge(&mut tx, id, req).await?;
    tx.commit().await?;
    Ok(Json(entry))
}

/// DELETE /api/v1/knowledge/:id — soft delete (sets deleted_at).
#[utoipa::path(
    delete,
    path = "/api/v1/knowledge/{id}",
    tag = "knowledge",
    params(
        ("id" = Uuid, Path, description = "Knowledge entry ID"),
    ),
    responses(
        (status = 204, description = "Entry deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Entry not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn delete_knowledge(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    service::delete_knowledge(&mut tx, id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
