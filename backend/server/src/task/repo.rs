use chrono::{DateTime, Utc};
use sqlx::Postgres;
use uuid::Uuid;

use crate::db::new_id;

// ── Row types ─────────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
pub struct TaskRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub story_id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub state: String,
    pub position: i32,
    pub assignee_id: Option<Uuid>,
    pub claude_session_id: Option<String>,
    pub ai_status_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct DependencyRow {
    pub id: Uuid,
    pub task_id: Uuid,
    pub depends_on_task_id: Uuid,
}

// ── Queries ───────────────────────────────────────────────────────────────────

const TASK_COLUMNS: &str = r#"
    id, org_id, story_id, name, description, task_type, state, position,
    assignee_id, claude_session_id, ai_status_text, created_at, updated_at
"#;

/// List non-deleted tasks for a story, ordered by position.
pub async fn list_tasks(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    story_id: Uuid,
) -> Result<Vec<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(&format!(
        "SELECT {TASK_COLUMNS} FROM tasks
         WHERE story_id = $1 AND deleted_at IS NULL
         ORDER BY position"
    ))
    .bind(story_id)
    .fetch_all(&mut **tx)
    .await
}

/// Fetch a single task by id (RLS-scoped to current org).
pub async fn get_task(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(&format!(
        "SELECT {TASK_COLUMNS} FROM tasks
         WHERE id = $1 AND deleted_at IS NULL"
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
}

/// Insert a new task.
#[allow(clippy::too_many_arguments)]
pub async fn create_task(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    story_id: Uuid,
    name: &str,
    description: &str,
    task_type: &str,
    position: i32,
    assignee_id: Option<Uuid>,
) -> Result<TaskRow, sqlx::Error> {
    let id = new_id();
    sqlx::query_as::<_, TaskRow>(&format!(
        "INSERT INTO tasks
             (id, org_id, story_id, name, description, task_type, position, assignee_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING {TASK_COLUMNS}"
    ))
    .bind(id)
    .bind(org_id)
    .bind(story_id)
    .bind(name)
    .bind(description)
    .bind(task_type)
    .bind(position)
    .bind(assignee_id)
    .fetch_one(&mut **tx)
    .await
}

/// Partial update via COALESCE semantics — None fields are left unchanged.
#[allow(clippy::too_many_arguments)]
pub async fn update_task(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    position: Option<i32>,
    assignee_id: Option<Uuid>,
    state: Option<&str>,
    claude_session_id: Option<&str>,
    ai_status_text: Option<&str>,
) -> Result<Option<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(&format!(
        "UPDATE tasks SET
             name              = COALESCE($2, name),
             description       = COALESCE($3, description),
             position          = COALESCE($4, position),
             assignee_id       = COALESCE($5, assignee_id),
             state             = COALESCE($6, state),
             claude_session_id = COALESCE($7, claude_session_id),
             ai_status_text    = COALESCE($8, ai_status_text),
             updated_at        = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING {TASK_COLUMNS}"
    ))
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(position)
    .bind(assignee_id)
    .bind(state)
    .bind(claude_session_id)
    .bind(ai_status_text)
    .fetch_optional(&mut **tx)
    .await
}

/// Soft-delete a task. Returns true if a row was affected.
pub async fn soft_delete_task(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE tasks SET deleted_at = now()
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Set state to 'done' and clear ai_status_text unconditionally.
/// The caller is responsible for checking that the task is in 'running' state.
pub async fn mark_done(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<TaskRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskRow>(&format!(
        "UPDATE tasks SET
             state          = 'done',
             ai_status_text = NULL,
             updated_at     = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING {TASK_COLUMNS}"
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
}

/// All dependency edges for tasks belonging to a story (for cycle detection + list).
pub async fn list_dependencies_for_story(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    story_id: Uuid,
) -> Result<Vec<DependencyRow>, sqlx::Error> {
    sqlx::query_as::<_, DependencyRow>(
        "SELECT td.id, td.task_id, td.depends_on_task_id
         FROM task_dependencies td
         JOIN tasks t ON t.id = td.task_id
         WHERE t.story_id = $1 AND t.deleted_at IS NULL",
    )
    .bind(story_id)
    .fetch_all(&mut **tx)
    .await
}

/// Dependency edges for a specific task.
pub async fn get_dependencies_for_task(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    task_id: Uuid,
) -> Result<Vec<DependencyRow>, sqlx::Error> {
    sqlx::query_as::<_, DependencyRow>(
        "SELECT id, task_id, depends_on_task_id
         FROM task_dependencies
         WHERE task_id = $1",
    )
    .bind(task_id)
    .fetch_all(&mut **tx)
    .await
}

/// Insert a dependency edge.
pub async fn create_dependency(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    task_id: Uuid,
    depends_on_task_id: Uuid,
) -> Result<DependencyRow, sqlx::Error> {
    let id = new_id();
    sqlx::query_as::<_, DependencyRow>(
        "INSERT INTO task_dependencies (id, task_id, depends_on_task_id)
         VALUES ($1, $2, $3)
         RETURNING id, task_id, depends_on_task_id",
    )
    .bind(id)
    .bind(task_id)
    .bind(depends_on_task_id)
    .fetch_one(&mut **tx)
    .await
}

/// Remove a dependency edge by id; scoped to the current org via tasks RLS.
/// Returns true if a row was deleted.
pub async fn delete_dependency(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM task_dependencies
         WHERE id = $1
           AND task_id IN (SELECT id FROM tasks WHERE deleted_at IS NULL)",
    )
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}
