use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, patch},
};
use uuid::Uuid;

use crate::{
    auth::{middleware::require_auth, types::AuthContext},
    errors::ApiError,
    state::AppState,
};

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
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(params): Query<ListKnowledgeParams>,
) -> Result<Json<Vec<KnowledgeResponse>>, ApiError> {
    let entries =
        service::list_knowledge(&state, auth.org_id, params.project_id, params.story_id).await?;
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
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateKnowledgeRequest>,
) -> Result<(StatusCode, Json<KnowledgeResponse>), ApiError> {
    let entry = service::create_knowledge(&state, auth.org_id, req).await?;
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
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateKnowledgeRequest>,
) -> Result<Json<KnowledgeResponse>, ApiError> {
    let entry = service::update_knowledge(&state, auth.org_id, id, req).await?;
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
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    service::delete_knowledge(&state, auth.org_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
