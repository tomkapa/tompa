use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
};
use uuid::Uuid;

use crate::{auth::middleware::require_auth, db::OrgTx, errors::ApiError, state::AppState};

use super::{
    service,
    types::{
        CreateProjectRequest, ListProjectsParams, ProjectResponse, UpdateProjectRequest,
        UpdateQaConfigRequest,
    },
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/projects", get(list_projects).post(create_project))
        .route(
            "/api/v1/projects/{id}",
            get(get_project)
                .patch(update_project)
                .delete(delete_project),
        )
        .route(
            "/api/v1/projects/{id}/qa-config",
            axum::routing::patch(update_qa_config),
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
    mut tx: OrgTx,
    Query(params): Query<ListProjectsParams>,
) -> Result<Json<Vec<ProjectResponse>>, ApiError> {
    if let Some(org_id) = params.org_id
        && org_id != tx.org_id
    {
        return Err(ApiError::Forbidden);
    }
    let projects = service::list_projects(&mut tx).await?;
    tx.commit().await?;
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
    mut tx: OrgTx,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), ApiError> {
    let project = service::create_project(&mut tx, req).await?;
    tx.commit().await?;
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
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let project = service::get_project(&mut tx, id).await?;
    tx.commit().await?;
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
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let project = service::update_project(&mut tx, id, req).await?;
    tx.commit().await?;
    Ok(Json(project))
}

/// PATCH /api/v1/projects/:id/qa-config
#[utoipa::path(
    patch,
    path = "/api/v1/projects/{id}/qa-config",
    tag = "projects",
    params(
        ("id" = Uuid, Path, description = "Project ID"),
    ),
    request_body = UpdateQaConfigRequest,
    responses(
        (status = 200, description = "Updated project with new qa_config", body = ProjectResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Project not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn update_qa_config(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateQaConfigRequest>,
) -> Result<Json<ProjectResponse>, ApiError> {
    let project = service::update_qa_config(&mut tx, id, req).await?;
    tx.commit().await?;
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
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    service::delete_project(&mut tx, id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
