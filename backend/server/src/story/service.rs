use uuid::Uuid;

use crate::{auth::middleware::set_org_context, errors::ApiError, state::AppState};

use super::{
    rank,
    repo::{self, StoryRow, TaskSummaryRow},
    types::{
        CreateStoryRequest, RankUpdateRequest, StoryError, StoryResponse, TaskSummary,
        UpdateStoryRequest, VALID_PIPELINE_STAGES, VALID_STORY_TYPES,
    },
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn to_response(row: StoryRow, tasks: Vec<TaskSummaryRow>) -> StoryResponse {
    StoryResponse {
        id: row.id,
        org_id: row.org_id,
        project_id: row.project_id,
        title: row.title,
        description: row.description,
        story_type: row.story_type,
        status: row.status,
        owner_id: row.owner_id,
        rank: row.rank,
        pipeline_stage: row.pipeline_stage,
        created_at: row.created_at,
        updated_at: row.updated_at,
        tasks: tasks
            .into_iter()
            .map(|t| TaskSummary {
                id: t.id,
                name: t.name,
                task_type: t.task_type,
                state: t.state,
                position: t.position,
            })
            .collect(),
    }
}

// ── Public service functions ──────────────────────────────────────────────────

pub async fn list_stories(
    state: &AppState,
    org_id: Uuid,
    project_id: Uuid,
) -> Result<Vec<StoryResponse>, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;
    let rows = repo::list_stories(&mut tx, project_id).await?;
    // For list, return empty task slices (not needed for list view)
    let stories = rows.into_iter().map(|r| to_response(r, vec![])).collect();
    tx.commit().await?;
    Ok(stories)
}

pub async fn get_story(
    state: &AppState,
    org_id: Uuid,
    id: Uuid,
) -> Result<StoryResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;
    let row = repo::get_story(&mut tx, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let tasks = repo::get_tasks_for_story(&mut tx, id).await?;
    tx.commit().await?;
    Ok(to_response(row, tasks))
}

pub async fn create_story(
    state: &AppState,
    org_id: Uuid,
    req: CreateStoryRequest,
) -> Result<StoryResponse, ApiError> {
    let title = req.title.trim().to_string();
    if title.is_empty() {
        return Err(StoryError::TitleRequired.into());
    }
    if !VALID_STORY_TYPES.contains(&req.story_type.as_str()) {
        return Err(StoryError::InvalidStoryType.into());
    }

    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    // New stories are appended after the current last story in the project
    let max_rank = repo::get_max_rank(&mut tx, req.project_id).await?;
    let new_rank = rank::generate_key_between(max_rank.as_deref(), None)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let row = repo::create_story(
        &mut tx,
        org_id,
        req.project_id,
        &title,
        &req.description,
        &req.story_type,
        req.owner_id,
        &new_rank,
    )
    .await?;
    tx.commit().await?;
    Ok(to_response(row, vec![]))
}

pub async fn update_story(
    state: &AppState,
    org_id: Uuid,
    id: Uuid,
    req: UpdateStoryRequest,
) -> Result<StoryResponse, ApiError> {
    // Validate inputs before touching the DB
    if let Some(ref t) = req.title {
        if t.trim().is_empty() {
            return Err(StoryError::TitleRequired.into());
        }
    }
    if let Some(ref stage) = req.pipeline_stage {
        if !VALID_PIPELINE_STAGES.contains(&stage.as_str()) {
            return Err(StoryError::InvalidPipelineStage.into());
        }
    }

    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    // Fetch current story to validate the status transition
    let current = repo::get_story(&mut tx, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if let Some(ref new_status) = req.status {
        validate_status_transition(&current.status, new_status)?;
    }

    let trimmed_title = req.title.as_deref().map(str::trim);
    let updated = repo::update_story(
        &mut tx,
        id,
        trimmed_title,
        req.description.as_deref(),
        req.status.as_deref(),
        req.owner_id,
        req.pipeline_stage.as_deref(),
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    let tasks = repo::get_tasks_for_story(&mut tx, id).await?;
    tx.commit().await?;
    Ok(to_response(updated, tasks))
}

pub async fn delete_story(state: &AppState, org_id: Uuid, id: Uuid) -> Result<(), ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;
    let deleted = repo::soft_delete_story(&mut tx, id).await?;
    tx.commit().await?;
    if !deleted {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

pub async fn update_rank(
    state: &AppState,
    org_id: Uuid,
    id: Uuid,
    req: RankUpdateRequest,
) -> Result<StoryResponse, ApiError> {
    if req.before_id.is_none() && req.after_id.is_none() {
        return Err(ApiError::BadRequest(
            "at least one of before_id or after_id must be provided".into(),
        ));
    }

    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    // Verify the target story exists
    repo::get_story(&mut tx, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Resolve the rank of after_id (lower bound)
    let lo_rank: Option<String> = if let Some(after_id) = req.after_id {
        let s = repo::get_story(&mut tx, after_id)
            .await?
            .ok_or(ApiError::NotFound)?;
        Some(s.rank)
    } else {
        None
    };

    // Resolve the rank of before_id (upper bound)
    let hi_rank: Option<String> = if let Some(before_id) = req.before_id {
        let s = repo::get_story(&mut tx, before_id)
            .await?
            .ok_or(ApiError::NotFound)?;
        Some(s.rank)
    } else {
        None
    };

    let new_rank = rank::generate_key_between(lo_rank.as_deref(), hi_rank.as_deref())
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let updated = repo::update_rank(&mut tx, id, &new_rank)
        .await?
        .ok_or(ApiError::NotFound)?;
    let tasks = repo::get_tasks_for_story(&mut tx, id).await?;
    tx.commit().await?;
    Ok(to_response(updated, tasks))
}

pub async fn start_story(
    state: &AppState,
    org_id: Uuid,
    id: Uuid,
) -> Result<StoryResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let current = repo::get_story(&mut tx, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if current.status != "todo" {
        return Err(StoryError::InvalidTransition {
            from: current.status.clone(),
            to: "in_progress".into(),
        }
        .into());
    }

    // Feature / refactor start with grooming; bugs skip straight to implementation
    let stage = if current.story_type == "bug" {
        "implementation"
    } else {
        "grooming"
    };

    let updated = repo::start_story(&mut tx, id, stage)
        .await?
        .ok_or(ApiError::NotFound)?;
    let tasks = repo::get_tasks_for_story(&mut tx, id).await?;
    tx.commit().await?;
    Ok(to_response(updated, tasks))
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Enforce the status state machine for PATCH updates.
///
/// Valid transitions (PATCH only):
/// - `in_progress` → `done`
/// - `in_progress` → `todo`  (cancel / revert)
///
/// `todo` → `in_progress` is intentionally excluded — use the `/start` endpoint.
fn validate_status_transition(from: &str, to: &str) -> Result<(), ApiError> {
    let valid = matches!(
        (from, to),
        ("in_progress", "done") | ("in_progress", "todo")
    );
    if !valid {
        return Err(StoryError::InvalidTransition {
            from: from.to_string(),
            to: to.to_string(),
        }
        .into());
    }
    Ok(())
}
