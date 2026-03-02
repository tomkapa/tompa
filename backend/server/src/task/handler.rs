use axum::{
    Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{delete, get, post},
};
use uuid::Uuid;

use crate::{auth::middleware::require_auth, db::OrgTx, errors::ApiError, state::AppState};

use super::{
    service,
    types::{
        CreateDependencyRequest, CreateTaskRequest, DependencyResponse, ListDependenciesParams,
        ListTasksParams, TaskResponse, UpdateTaskRequest,
    },
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/tasks", get(list_tasks).post(create_task))
        .route(
            "/api/v1/tasks/{id}",
            get(get_task).patch(update_task).delete(delete_task),
        )
        .route("/api/v1/tasks/{id}/done", post(mark_done))
        .route(
            "/api/v1/task-dependencies",
            get(list_dependencies).post(create_dependency),
        )
        .route("/api/v1/task-dependencies/{id}", delete(delete_dependency))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// GET /api/v1/tasks?story_id=X — list tasks ordered by position.
#[utoipa::path(
    get,
    path = "/api/v1/tasks",
    tag = "tasks",
    params(
        ("story_id" = Uuid, Query, description = "Story ID to list tasks for"),
    ),
    responses(
        (status = 200, description = "List of tasks ordered by position", body = [TaskResponse]),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn list_tasks(
    mut tx: OrgTx,
    Query(params): Query<ListTasksParams>,
) -> Result<Json<Vec<TaskResponse>>, ApiError> {
    let tasks = service::list_tasks(&mut tx, params.story_id).await?;
    tx.commit().await?;
    Ok(Json(tasks))
}

/// POST /api/v1/tasks — create a task.
#[utoipa::path(
    post,
    path = "/api/v1/tasks",
    tag = "tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = TaskResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Story not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn create_task(
    mut tx: OrgTx,
    Json(req): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), ApiError> {
    let task = service::create_task(&mut tx, req).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(task)))
}

/// GET /api/v1/tasks/:id — task detail.
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{id}",
    tag = "tasks",
    params(
        ("id" = Uuid, Path, description = "Task ID"),
    ),
    responses(
        (status = 200, description = "Task detail", body = TaskResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn get_task(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, ApiError> {
    let task = service::get_task(&mut tx, id).await?;
    tx.commit().await?;
    Ok(Json(task))
}

/// PATCH /api/v1/tasks/:id — partial update.
#[utoipa::path(
    patch,
    path = "/api/v1/tasks/{id}",
    tag = "tasks",
    params(
        ("id" = Uuid, Path, description = "Task ID"),
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Updated task", body = TaskResponse),
        (status = 400, description = "Invalid state transition or validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn update_task(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, ApiError> {
    let task = service::update_task(&mut tx, id, req).await?;
    tx.commit().await?;
    Ok(Json(task))
}

/// DELETE /api/v1/tasks/:id — soft delete.
#[utoipa::path(
    delete,
    path = "/api/v1/tasks/{id}",
    tag = "tasks",
    params(
        ("id" = Uuid, Path, description = "Task ID"),
    ),
    responses(
        (status = 204, description = "Task deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn delete_task(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    service::delete_task(&mut tx, id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/tasks/:id/done — human sign-off (task must be in 'running' state).
#[utoipa::path(
    post,
    path = "/api/v1/tasks/{id}/done",
    tag = "tasks",
    params(
        ("id" = Uuid, Path, description = "Task ID"),
    ),
    responses(
        (status = 200, description = "Task marked as done", body = TaskResponse),
        (status = 400, description = "Task is not in running state"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn mark_done(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, ApiError> {
    let task = service::mark_done(&mut tx, id).await?;
    tx.commit().await?;
    Ok(Json(task))
}

/// GET /api/v1/task-dependencies?story_id=X — list dependency edges for a story.
#[utoipa::path(
    get,
    path = "/api/v1/task-dependencies",
    tag = "tasks",
    params(
        ("story_id" = Uuid, Query, description = "Story ID to list dependencies for"),
    ),
    responses(
        (status = 200, description = "List of dependency edges", body = [DependencyResponse]),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn list_dependencies(
    mut tx: OrgTx,
    Query(params): Query<ListDependenciesParams>,
) -> Result<Json<Vec<DependencyResponse>>, ApiError> {
    let deps = service::list_dependencies(&mut tx, params.story_id).await?;
    tx.commit().await?;
    Ok(Json(deps))
}

/// POST /api/v1/task-dependencies — create a dependency edge.
#[utoipa::path(
    post,
    path = "/api/v1/task-dependencies",
    tag = "tasks",
    request_body = CreateDependencyRequest,
    responses(
        (status = 201, description = "Dependency created", body = DependencyResponse),
        (status = 400, description = "Cyclic dependency or tasks in different stories"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn create_dependency(
    mut tx: OrgTx,
    Json(req): Json<CreateDependencyRequest>,
) -> Result<(StatusCode, Json<DependencyResponse>), ApiError> {
    let dep = service::create_dependency(&mut tx, req).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(dep)))
}

/// DELETE /api/v1/task-dependencies/:id — remove a dependency edge.
#[utoipa::path(
    delete,
    path = "/api/v1/task-dependencies/{id}",
    tag = "tasks",
    params(
        ("id" = Uuid, Path, description = "Dependency edge ID"),
    ),
    responses(
        (status = 204, description = "Dependency removed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Dependency not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn delete_dependency(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    service::delete_dependency(&mut tx, id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
