use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::{
    auth::{middleware::require_auth, types::AuthContext},
    errors::ApiError,
    state::AppState,
};

use super::{
    service,
    types::{CreateProjectRequest, ListProjectsParams, ProjectResponse, UpdateProjectRequest},
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/projects", get(list_projects).post(create_project))
        .route(
            "/api/v1/projects/:id",
            get(get_project).patch(update_project).delete(delete_project),
        )
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// GET /api/v1/projects?org_id=X
#[utoipa::path(
    get,
    path = "/api/v1/projects",
    tag = "projects",
    params(
        ("org_id" = Option<Uuid>, Query, description = "Optional org filter (must match session org)"),
    ),
    responses(
        (status = 200, description = "List of projects in the org", body = [ProjectResponse]),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — org_id does not match session"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn list_projects(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(params): Query<ListProjectsParams>,
) -> Result<Json<Vec<ProjectResponse>>, ApiError> {
    if let Some(org_id) = params.org_id {
        if org_id != auth.org_id {
            return Err(ApiError::Forbidden);
        }
    }
    let projects = service::list_projects(&state, auth.org_id).await?;
    Ok(Json(projects))
}

/// POST /api/v1/projects
#[utoipa::path(
    post,
    path = "/api/v1/projects",
    tag = "projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created", body = ProjectResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn create_project(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), ApiError> {
    let project = service::create_project(&state, auth.org_id, req).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

/// GET /api/v1/projects/:id
#[utoipa::path(
    get,
    path = "/api/v1/projects/{id}",
    tag = "projects",
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project detail", body = ProjectResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Project not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn get_project(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let project = service::get_project(&state, auth.org_id, id).await?;
    Ok(Json(project))
}

/// PATCH /api/v1/projects/:id
#[utoipa::path(
    patch,
    path = "/api/v1/projects/{id}",
    tag = "projects",
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Updated project", body = ProjectResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Project not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn update_project(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let project = service::update_project(&state, auth.org_id, id, req).await?;
    Ok(Json(project))
}

/// DELETE /api/v1/projects/:id — soft delete (sets deleted_at).
#[utoipa::path(
    delete,
    path = "/api/v1/projects/{id}",
    tag = "projects",
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    responses(
        (status = 204, description = "Project deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Project not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn delete_project(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    service::delete_project(&state, auth.org_id, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
