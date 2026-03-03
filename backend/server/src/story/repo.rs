use chrono::{DateTime, Utc};
use sqlx::Postgres;
use uuid::Uuid;

use crate::db::new_id;

// ── Row types ─────────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
pub struct StoryRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: String,
    pub story_type: String,
    pub status: String,
    pub owner_id: Uuid,
    pub rank: String,
    pub pipeline_stage: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub owner_name: String,
}

#[derive(sqlx::FromRow)]
pub struct TaskSummaryRow {
    pub id: Uuid,
    pub name: String,
    pub task_type: String,
    pub state: String,
    pub position: i32,
}

// ── Queries ───────────────────────────────────────────────────────────────────

const STORY_COLUMNS: &str = r#"
    id, org_id, project_id, title, description, story_type,
    status, owner_id, rank, pipeline_stage, created_at, updated_at,
    COALESCE((SELECT display_name FROM users WHERE users.id = owner_id), '') as owner_name
"#;

/// List non-deleted stories for a project, ordered by rank ascending.
pub async fn list_stories(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    project_id: Uuid,
) -> Result<Vec<StoryRow>, sqlx::Error> {
    sqlx::query_as::<_, StoryRow>(&format!(
        "SELECT {STORY_COLUMNS} FROM stories
         WHERE project_id = $1 AND org_id = $2 AND deleted_at IS NULL
         ORDER BY rank"
    ))
    .bind(project_id)
    .bind(org_id)
    .fetch_all(&mut **tx)
    .await
}

/// Fetch a single story by id, scoped to org_id.
pub async fn get_story(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
) -> Result<Option<StoryRow>, sqlx::Error> {
    sqlx::query_as::<_, StoryRow>(&format!(
        "SELECT {STORY_COLUMNS} FROM stories
         WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL"
    ))
    .bind(id)
    .bind(org_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Fetch the rank of the last (highest) story in a project.
pub async fn get_max_rank(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT rank FROM stories
         WHERE project_id = $1 AND deleted_at IS NULL
         ORDER BY rank DESC
         LIMIT 1",
    )
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.map(|(r,)| r))
}

/// Fetch tasks for a story ordered by position (for the detail response).
pub async fn get_tasks_for_story(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    story_id: Uuid,
) -> Result<Vec<TaskSummaryRow>, sqlx::Error> {
    sqlx::query_as::<_, TaskSummaryRow>(
        "SELECT id, name, task_type, state, position
         FROM tasks
         WHERE story_id = $1 AND deleted_at IS NULL
         ORDER BY position",
    )
    .bind(story_id)
    .fetch_all(&mut **tx)
    .await
}

/// Insert a new story.
#[allow(clippy::too_many_arguments)]
pub async fn create_story(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    project_id: Uuid,
    title: &str,
    description: &str,
    story_type: &str,
    owner_id: Uuid,
    rank: &str,
) -> Result<StoryRow, sqlx::Error> {
    let id = new_id();
    sqlx::query_as::<_, StoryRow>(&format!(
        "INSERT INTO stories
             (id, org_id, project_id, title, description, story_type, owner_id, rank)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING {STORY_COLUMNS}"
    ))
    .bind(id)
    .bind(org_id)
    .bind(project_id)
    .bind(title)
    .bind(description)
    .bind(story_type)
    .bind(owner_id)
    .bind(rank)
    .fetch_one(&mut **tx)
    .await
}

/// Partial update of title/description/status/owner/pipeline_stage.
/// Fields that are `None` are left unchanged (COALESCE semantics).
#[allow(clippy::too_many_arguments)]
pub async fn update_story(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
    title: Option<&str>,
    description: Option<&str>,
    status: Option<&str>,
    owner_id: Option<Uuid>,
    pipeline_stage: Option<&str>,
) -> Result<Option<StoryRow>, sqlx::Error> {
    sqlx::query_as::<_, StoryRow>(&format!(
        "UPDATE stories SET
             title          = COALESCE($2, title),
             description    = COALESCE($3, description),
             status         = COALESCE($4, status),
             owner_id       = COALESCE($5, owner_id),
             pipeline_stage = COALESCE($6, pipeline_stage),
             updated_at     = now()
         WHERE id = $1 AND org_id = $7 AND deleted_at IS NULL
         RETURNING {STORY_COLUMNS}"
    ))
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(status)
    .bind(owner_id)
    .bind(pipeline_stage)
    .bind(org_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Move to in_progress and set the pipeline_stage.
pub async fn start_story(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
    pipeline_stage: &str,
) -> Result<Option<StoryRow>, sqlx::Error> {
    sqlx::query_as::<_, StoryRow>(&format!(
        "UPDATE stories SET
             status         = 'in_progress',
             pipeline_stage = $2,
             updated_at     = now()
         WHERE id = $1 AND org_id = $3 AND deleted_at IS NULL
         RETURNING {STORY_COLUMNS}"
    ))
    .bind(id)
    .bind(pipeline_stage)
    .bind(org_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Update the rank of a story (used for reordering).
pub async fn update_rank(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
    rank: &str,
) -> Result<Option<StoryRow>, sqlx::Error> {
    sqlx::query_as::<_, StoryRow>(&format!(
        "UPDATE stories SET rank = $2, updated_at = now()
         WHERE id = $1 AND org_id = $3 AND deleted_at IS NULL
         RETURNING {STORY_COLUMNS}"
    ))
    .bind(id)
    .bind(rank)
    .bind(org_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Soft-delete a story. Returns true if a row was affected.
pub async fn soft_delete_story(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE stories SET deleted_at = now()
         WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(org_id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}
