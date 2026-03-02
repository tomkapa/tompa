use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{delete, get},
};
use uuid::Uuid;

use crate::{auth::middleware::require_auth, db::OrgTx, errors::ApiError, state::AppState};

use super::{
    service,
    types::{CreateKeyRequest, CreateKeyResponse, KeyListItem, ListKeysParams},
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/container-keys", get(list_keys).post(create_key))
        .route("/api/v1/container-keys/{id}", delete(revoke_key))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// GET /api/v1/container-keys?project_id=X
#[utoipa::path(
    get,
    path = "/api/v1/container-keys",
    tag = "container-keys",
    params(
        ("project_id" = Uuid, Query, description = "Project ID to list keys for"),
    ),
    responses(
        (status = 200, description = "List of container API keys (key hash never included)", body = [KeyListItem]),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn list_keys(
    mut tx: OrgTx,
    Query(params): Query<ListKeysParams>,
) -> Result<Json<Vec<KeyListItem>>, ApiError> {
    let keys = service::list_keys(&mut tx, params.project_id).await?;
    tx.commit().await?;
    Ok(Json(keys))
}

/// POST /api/v1/container-keys — returns the raw key once.
#[utoipa::path(
    post,
    path = "/api/v1/container-keys",
    tag = "container-keys",
    request_body = CreateKeyRequest,
    responses(
        (status = 201, description = "Key created — raw api_key returned only once", body = CreateKeyResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Project not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn create_key(
    mut tx: OrgTx,
    Json(req): Json<CreateKeyRequest>,
) -> Result<(StatusCode, Json<CreateKeyResponse>), ApiError> {
    let key = service::create_key(&mut tx, req).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(key)))
}

/// DELETE /api/v1/container-keys/:id — soft revoke (sets revoked_at).
#[utoipa::path(
    delete,
    path = "/api/v1/container-keys/{id}",
    tag = "container-keys",
    params(
        ("id" = Uuid, Path, description = "Key ID to revoke"),
    ),
    responses(
        (status = 204, description = "Key revoked"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Key not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn revoke_key(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    service::revoke_key(&mut tx, id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
