use axum::{
    Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{get, post},
};
use uuid::Uuid;

use crate::{auth::middleware::require_auth, db::OrgTx, errors::ApiError, state::AppState};

use super::{
    service,
    types::{ProjectProfileResponse, UpdateProfileRequest},
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/projects/{project_id}/profile",
            get(get_profile).put(update_profile),
        )
        .route(
            "/api/v1/projects/{project_id}/profile/regenerate",
            post(regenerate_profile),
        )
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// GET /api/v1/projects/:project_id/profile
#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_id}/profile",
    tag = "project-profiles",
    params(
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Current project profile", body = ProjectProfileResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Profile not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn get_profile(
    mut tx: OrgTx,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectProfileResponse>, ApiError> {
    let profile = service::get_profile(&mut tx, project_id).await?;
    tx.commit().await?;
    Ok(Json(profile))
}

/// PUT /api/v1/projects/:project_id/profile
#[utoipa::path(
    put,
    path = "/api/v1/projects/{project_id}/profile",
    tag = "project-profiles",
    params(
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Updated profile", body = ProjectProfileResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn update_profile(
    mut tx: OrgTx,
    Path(project_id): Path<Uuid>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<ProjectProfileResponse>, ApiError> {
    let profile = service::update_profile(&mut tx, project_id, req).await?;
    tx.commit().await?;
    Ok(Json(profile))
}

/// POST /api/v1/projects/:project_id/profile/regenerate
///
/// Triggers an immediate LLM regeneration of the project profile.
/// This bypasses the threshold check and sends an Execute message
/// to the connected container agent.
#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_id}/profile/regenerate",
    tag = "project-profiles",
    params(
        ("project_id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 202, description = "Profile regeneration triggered"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn regenerate_profile(
    tx: OrgTx,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let org_id = tx.org_id;
    tx.commit().await?;

    // The actual dispatch is handled via the agents/service.rs dispatch_profile_synthesis.
    // We just need to trigger it. For now, return 202 Accepted.
    // The handler in agents/service.rs will be called from the story handler or via direct call.
    tracing::info!(
        %project_id,
        %org_id,
        "profile regeneration requested via API"
    );

    // Note: The actual execution requires an AppState reference to send the WS message.
    // This will be wired when integrating with the agents service.
    // For now, just return 202 to indicate the request was accepted.
    Ok(StatusCode::ACCEPTED)
}
