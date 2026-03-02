use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{auth::middleware::set_org_context, errors::ApiError, state::AppState};

use shared::enums::TaskState;

use super::{
    repo::{self, DependencyRow, TaskRow},
    types::{
        CreateDependencyRequest, CreateTaskRequest, DependencyResponse, TaskError, TaskResponse,
        UpdateTaskRequest, VALID_TASK_TYPES,
    },
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn to_task_response(row: TaskRow, deps: Vec<DependencyRow>) -> TaskResponse {
    TaskResponse {
        id: row.id,
        org_id: row.org_id,
        story_id: row.story_id,
        name: row.name,
        description: row.description,
        task_type: row.task_type,
        state: row.state,
        position: row.position,
        assignee_id: row.assignee_id,
        claude_session_id: row.claude_session_id,
        ai_status_text: row.ai_status_text,
        created_at: row.created_at,
        updated_at: row.updated_at,
        dependencies: deps
            .into_iter()
            .map(|d| DependencyResponse {
                id: d.id,
                task_id: d.task_id,
                depends_on_task_id: d.depends_on_task_id,
            })
            .collect(),
    }
}

/// Parse a task state string from the DB to the shared enum.
fn parse_state(s: &str) -> TaskState {
    match s {
        "pending" => TaskState::Pending,
        "qa" => TaskState::Qa,
        "running" => TaskState::Running,
        "paused" => TaskState::Paused,
        "blocked" => TaskState::Blocked,
        "done" => TaskState::Done,
        _ => TaskState::Pending,
    }
}

/// Enforce valid PATCH state transitions.
///
/// Any state → `pending` is always valid (reset/cancel).
/// `running` → `done` is reserved for the `/done` endpoint.
fn validate_state_transition(from: &str, to: &str) -> Result<(), ApiError> {
    if to == "pending" {
        return Ok(());
    }
    let valid = matches!(
        (from, to),
        ("pending", "qa")
            | ("qa", "running")
            | ("running", "paused")
            | ("paused", "running")
            | ("running", "blocked")
            | ("blocked", "running")
    );
    if !valid {
        return Err(TaskError::InvalidState {
            from: parse_state(from),
            to: parse_state(to),
        }
        .into());
    }
    Ok(())
}

/// DFS cycle detection. Returns `true` if adding edge (task_id → depends_on)
/// would create a cycle in the existing edge set.
fn would_create_cycle(edges: &[(Uuid, Uuid)], task_id: Uuid, depends_on: Uuid) -> bool {
    // If they're the same node it's a self-loop — always a cycle.
    if task_id == depends_on {
        return true;
    }

    // Build adjacency: from → [to] using existing edges
    let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for &(from, to) in edges {
        adj.entry(from).or_default().push(to);
    }

    // DFS from `depends_on` — if we reach `task_id`, adding the new edge
    // would create a cycle (depends_on → ... → task_id → depends_on).
    let mut visited = HashSet::new();
    let mut stack = vec![depends_on];
    while let Some(curr) = stack.pop() {
        if curr == task_id {
            return true;
        }
        if visited.insert(curr)
            && let Some(neighbors) = adj.get(&curr)
        {
            stack.extend(neighbors);
        }
    }
    false
}

// ── Public service functions ──────────────────────────────────────────────────

pub async fn list_tasks(
    state: &AppState,
    org_id: Uuid,
    story_id: Uuid,
) -> Result<Vec<TaskResponse>, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let tasks = repo::list_tasks(&mut tx, story_id).await?;
    let all_deps = repo::list_dependencies_for_story(&mut tx, story_id).await?;

    // Group deps by task_id
    let mut deps_by_task: HashMap<Uuid, Vec<DependencyRow>> = HashMap::new();
    for dep in all_deps {
        deps_by_task.entry(dep.task_id).or_default().push(dep);
    }

    let result = tasks
        .into_iter()
        .map(|t| {
            let deps = deps_by_task.remove(&t.id).unwrap_or_default();
            to_task_response(t, deps)
        })
        .collect();

    tx.commit().await?;
    Ok(result)
}

pub async fn get_task(state: &AppState, org_id: Uuid, id: Uuid) -> Result<TaskResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let row = repo::get_task(&mut tx, id, org_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let deps = repo::get_dependencies_for_task(&mut tx, id).await?;
    tx.commit().await?;
    Ok(to_task_response(row, deps))
}

pub async fn create_task(
    state: &AppState,
    org_id: Uuid,
    req: CreateTaskRequest,
) -> Result<TaskResponse, ApiError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    if !VALID_TASK_TYPES.contains(&req.task_type.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "invalid task_type; must be one of: {}",
            VALID_TASK_TYPES.join(", ")
        )));
    }

    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    // Verify the story exists in this org
    let story_exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM stories WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL")
            .bind(req.story_id)
            .bind(org_id)
            .fetch_optional(&mut *tx)
            .await?;

    if story_exists.is_none() {
        return Err(TaskError::StoryNotFound.into());
    }

    let row = repo::create_task(
        &mut tx,
        org_id,
        req.story_id,
        &name,
        &req.description,
        &req.task_type,
        req.position,
        req.assignee_id,
    )
    .await?;

    tx.commit().await?;
    Ok(to_task_response(row, vec![]))
}

pub async fn update_task(
    state: &AppState,
    org_id: Uuid,
    id: Uuid,
    req: UpdateTaskRequest,
) -> Result<TaskResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let current = repo::get_task(&mut tx, id, org_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if let Some(ref new_state) = req.state {
        validate_state_transition(&current.state, new_state)?;
    }

    let updated = repo::update_task(
        &mut tx,
        id,
        org_id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.position,
        req.assignee_id,
        req.state.as_deref(),
        req.claude_session_id.as_deref(),
        req.ai_status_text.as_deref(),
    )
    .await?
    .ok_or(ApiError::NotFound)?;

    let deps = repo::get_dependencies_for_task(&mut tx, id).await?;
    tx.commit().await?;
    Ok(to_task_response(updated, deps))
}

pub async fn delete_task(state: &AppState, org_id: Uuid, id: Uuid) -> Result<(), ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let deleted = repo::soft_delete_task(&mut tx, id, org_id).await?;
    tx.commit().await?;

    if !deleted {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

pub async fn mark_done(state: &AppState, org_id: Uuid, id: Uuid) -> Result<TaskResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let current = repo::get_task(&mut tx, id, org_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if current.state != "running" {
        return Err(TaskError::NotRunning.into());
    }

    let updated = repo::mark_done(&mut tx, id, org_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let deps = repo::get_dependencies_for_task(&mut tx, id).await?;
    tx.commit().await?;
    Ok(to_task_response(updated, deps))
}

// ── Dependency service functions ──────────────────────────────────────────────

pub async fn list_dependencies(
    state: &AppState,
    org_id: Uuid,
    story_id: Uuid,
) -> Result<Vec<DependencyResponse>, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let rows = repo::list_dependencies_for_story(&mut tx, story_id).await?;
    tx.commit().await?;

    Ok(rows
        .into_iter()
        .map(|d| DependencyResponse {
            id: d.id,
            task_id: d.task_id,
            depends_on_task_id: d.depends_on_task_id,
        })
        .collect())
}

pub async fn create_dependency(
    state: &AppState,
    org_id: Uuid,
    req: CreateDependencyRequest,
) -> Result<DependencyResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    // Verify both tasks exist and share a story
    let task = repo::get_task(&mut tx, req.task_id, org_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let depends_on = repo::get_task(&mut tx, req.depends_on_task_id, org_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    if task.story_id != depends_on.story_id {
        return Err(TaskError::DifferentStory.into());
    }

    // Load all existing edges for the story and check for cycles
    let existing = repo::list_dependencies_for_story(&mut tx, task.story_id).await?;
    let edges: Vec<(Uuid, Uuid)> = existing
        .iter()
        .map(|d| (d.task_id, d.depends_on_task_id))
        .collect();

    if would_create_cycle(&edges, req.task_id, req.depends_on_task_id) {
        return Err(TaskError::CyclicDependency.into());
    }

    let row = repo::create_dependency(&mut tx, req.task_id, req.depends_on_task_id).await?;
    tx.commit().await?;

    Ok(DependencyResponse {
        id: row.id,
        task_id: row.task_id,
        depends_on_task_id: row.depends_on_task_id,
    })
}

pub async fn delete_dependency(state: &AppState, org_id: Uuid, id: Uuid) -> Result<(), ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;

    let deleted = repo::delete_dependency(&mut tx, id).await?;
    tx.commit().await?;

    if !deleted {
        return Err(ApiError::NotFound);
    }
    Ok(())
}
