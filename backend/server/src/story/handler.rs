use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::{auth::middleware::require_auth, db::OrgTx, errors::ApiError, state::AppState};

use super::{
    service,
    types::{
        CreateStoryRequest, ListStoriesParams, RankUpdateRequest, StoryResponse, UpdateStoryRequest,
    },
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/stories", get(list_stories).post(create_story))
        .route(
            "/api/v1/stories/{id}",
            get(get_story).patch(update_story).delete(delete_story),
        )
        .route("/api/v1/stories/{id}/rank", patch(update_rank))
        .route("/api/v1/stories/{id}/start", post(start_story))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}

/// GET /api/v1/stories?project_id=X — list stories ordered by rank.
#[utoipa::path(
    get,
    path = "/api/v1/stories",
    tag = "stories",
    params(
        ("project_id" = Uuid, Query, description = "Project ID to list stories for"),
    ),
    responses(
        (status = 200, description = "List of stories ordered by rank", body = [StoryResponse]),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn list_stories(
    mut tx: OrgTx,
    Query(params): Query<ListStoriesParams>,
) -> Result<Json<Vec<StoryResponse>>, ApiError> {
    let stories = service::list_stories(&mut tx, params.project_id).await?;
    tx.commit().await?;
    Ok(Json(stories))
}

/// POST /api/v1/stories — create a story (appended to end of project backlog).
#[utoipa::path(
    post,
    path = "/api/v1/stories",
    tag = "stories",
    request_body = CreateStoryRequest,
    responses(
        (status = 201, description = "Story created", body = StoryResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn create_story(
    mut tx: OrgTx,
    Json(req): Json<CreateStoryRequest>,
) -> Result<(StatusCode, Json<StoryResponse>), ApiError> {
    let story = service::create_story(&mut tx, req).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(story)))
}

/// GET /api/v1/stories/:id — story detail including task list.
#[utoipa::path(
    get,
    path = "/api/v1/stories/{id}",
    tag = "stories",
    params(
        ("id" = Uuid, Path, description = "Story ID"),
    ),
    responses(
        (status = 200, description = "Story detail with task list", body = StoryResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Story not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn get_story(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<Json<StoryResponse>, ApiError> {
    let story = service::get_story(&mut tx, id).await?;
    tx.commit().await?;
    Ok(Json(story))
}

/// PATCH /api/v1/stories/:id — partial update (title, description, status, owner, pipeline_stage).
#[utoipa::path(
    patch,
    path = "/api/v1/stories/{id}",
    tag = "stories",
    params(
        ("id" = Uuid, Path, description = "Story ID"),
    ),
    request_body = UpdateStoryRequest,
    responses(
        (status = 200, description = "Updated story", body = StoryResponse),
        (status = 400, description = "Invalid status transition or validation error"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Story not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn update_story(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStoryRequest>,
) -> Result<Json<StoryResponse>, ApiError> {
    let story = service::update_story(&mut tx, id, req).await?;
    tx.commit().await?;
    Ok(Json(story))
}

/// DELETE /api/v1/stories/:id — soft delete.
#[utoipa::path(
    delete,
    path = "/api/v1/stories/{id}",
    tag = "stories",
    params(
        ("id" = Uuid, Path, description = "Story ID"),
    ),
    responses(
        (status = 204, description = "Story deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Story not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn delete_story(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    service::delete_story(&mut tx, id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /api/v1/stories/:id/rank — reorder via fractional indexing.
#[utoipa::path(
    patch,
    path = "/api/v1/stories/{id}/rank",
    tag = "stories",
    params(
        ("id" = Uuid, Path, description = "Story ID to reorder"),
    ),
    request_body = RankUpdateRequest,
    responses(
        (status = 200, description = "Story with updated rank", body = StoryResponse),
        (status = 400, description = "Invalid rank parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Story not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn update_rank(
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
    Json(req): Json<RankUpdateRequest>,
) -> Result<Json<StoryResponse>, ApiError> {
    let story = service::update_rank(&mut tx, id, req).await?;
    tx.commit().await?;
    Ok(Json(story))
}

/// POST /api/v1/stories/:id/start — move from "todo" to "in_progress".
#[utoipa::path(
    post,
    path = "/api/v1/stories/{id}/start",
    tag = "stories",
    params(
        ("id" = Uuid, Path, description = "Story ID"),
    ),
    responses(
        (status = 200, description = "Story moved to in_progress", body = StoryResponse),
        (status = 400, description = "Invalid status transition"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Story not found"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn start_story(
    State(_state): State<AppState>,
    mut tx: OrgTx,
    Path(id): Path<Uuid>,
) -> Result<Json<StoryResponse>, ApiError> {
    let story = service::start_story(&mut tx, id).await?;
    tx.commit().await?;
    // TODO: trigger agents::send_start_grooming via _state when container pipeline is wired
    Ok(Json(story))
}
