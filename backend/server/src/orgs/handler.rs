use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    routing::get,
};

use crate::{
    auth::{middleware::require_auth, types::AuthContext},
    errors::ApiError,
    state::AppState,
};

use super::{
    service,
    types::{CreateOrgRequest, OrgResponse},
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/orgs", get(list_orgs).post(create_org))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// GET /api/v1/orgs — list all orgs the authenticated user belongs to.
#[utoipa::path(
    get,
    path = "/api/v1/orgs",
    tag = "orgs",
    responses(
        (status = 200, description = "List of orgs the user belongs to", body = [OrgResponse]),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn list_orgs(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<OrgResponse>>, ApiError> {
    let orgs = service::list_orgs(&state, auth.user_id).await?;
    Ok(Json(orgs))
}

/// POST /api/v1/orgs — create a new org and add the creator as owner.
#[utoipa::path(
    post,
    path = "/api/v1/orgs",
    tag = "orgs",
    request_body = CreateOrgRequest,
    responses(
        (status = 201, description = "Org created", body = OrgResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn create_org(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateOrgRequest>,
) -> Result<(StatusCode, Json<OrgResponse>), ApiError> {
    let org = service::create_org(&state, auth.user_id, req).await?;
    Ok((StatusCode::CREATED, Json(org)))
}
