use chrono::{DateTime, Utc};
use sqlx::Postgres;
use uuid::Uuid;

use crate::db::new_id;

// ── Row types ─────────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
pub struct QaRoundRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub story_id: Uuid,
    pub task_id: Option<Uuid>,
    pub stage: String,
    pub round_number: i32,
    pub status: String,
    pub content: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Queries ───────────────────────────────────────────────────────────────────

const ROUND_COLUMNS: &str =
    "id, org_id, story_id, task_id, stage, round_number, status, content, created_at, updated_at";

/// List rounds for a story (task_id IS NULL) or for a specific task, with
/// optional stage filter. Ordered by round_number ascending.
pub async fn list_rounds(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    story_id: Option<Uuid>,
    task_id: Option<Uuid>,
    stage: Option<&str>,
) -> Result<Vec<QaRoundRow>, sqlx::Error> {
    // Build query dynamically based on which filter combination is provided.
    // Either task_id or story_id (story-level, task_id IS NULL) must be present.
    let mut conditions: Vec<String> = Vec::new();
    let mut idx = 1u32;

    if let Some(_) = task_id {
        conditions.push(format!("task_id = ${idx}"));
        idx += 1;
    } else if let Some(_) = story_id {
        conditions.push(format!("story_id = ${idx}"));
        idx += 1;
        conditions.push("task_id IS NULL".to_string());
    }

    if let Some(_) = stage {
        conditions.push(format!("stage = ${idx}"));
    }

    let where_clause = if conditions.is_empty() {
        "TRUE".to_string()
    } else {
        conditions.join(" AND ")
    };

    let sql = format!(
        "SELECT {ROUND_COLUMNS} FROM qa_rounds WHERE {where_clause} ORDER BY round_number"
    );

    let mut q = sqlx::query_as::<_, QaRoundRow>(&sql);
    if let Some(tid) = task_id {
        q = q.bind(tid);
    } else if let Some(sid) = story_id {
        q = q.bind(sid);
    }
    if let Some(s) = stage {
        q = q.bind(s);
    }
    q.fetch_all(&mut **tx).await
}

/// Fetch a single round by id (RLS-scoped to current org).
pub async fn get_round(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<QaRoundRow>, sqlx::Error> {
    sqlx::query_as::<_, QaRoundRow>(&format!(
        "SELECT {ROUND_COLUMNS} FROM qa_rounds WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
}

/// Get the highest round_number for a given story/task/stage scope.
/// Returns None if no rounds exist yet (MAX returns NULL).
pub async fn get_max_round_number(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    story_id: Uuid,
    task_id: Option<Uuid>,
    stage: &str,
) -> Result<Option<i32>, sqlx::Error> {
    // MAX() always returns one row; use Option<i32> to handle NULL when no rows exist.
    let row: (Option<i32>,) = if let Some(tid) = task_id {
        sqlx::query_as(
            "SELECT MAX(round_number) FROM qa_rounds
             WHERE story_id = $1 AND task_id = $2 AND stage = $3",
        )
        .bind(story_id)
        .bind(tid)
        .bind(stage)
        .fetch_one(&mut **tx)
        .await?
    } else {
        sqlx::query_as(
            "SELECT MAX(round_number) FROM qa_rounds
             WHERE story_id = $1 AND task_id IS NULL AND stage = $2",
        )
        .bind(story_id)
        .bind(stage)
        .fetch_one(&mut **tx)
        .await?
    };
    Ok(row.0)
}

/// Insert a new QA round.
pub async fn create_round(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    story_id: Uuid,
    task_id: Option<Uuid>,
    stage: &str,
    round_number: i32,
    content: &serde_json::Value,
) -> Result<QaRoundRow, sqlx::Error> {
    let id = new_id();
    sqlx::query_as::<_, QaRoundRow>(&format!(
        "INSERT INTO qa_rounds
             (id, org_id, story_id, task_id, stage, round_number, content)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING {ROUND_COLUMNS}"
    ))
    .bind(id)
    .bind(org_id)
    .bind(story_id)
    .bind(task_id)
    .bind(stage)
    .bind(round_number)
    .bind(content)
    .fetch_one(&mut **tx)
    .await
}

/// Update the JSONB content of a round and refresh updated_at.
pub async fn update_round_content(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    content: &serde_json::Value,
) -> Result<Option<QaRoundRow>, sqlx::Error> {
    sqlx::query_as::<_, QaRoundRow>(&format!(
        "UPDATE qa_rounds SET content = $2, updated_at = now()
         WHERE id = $1
         RETURNING {ROUND_COLUMNS}"
    ))
    .bind(id)
    .bind(content)
    .fetch_optional(&mut **tx)
    .await
}

/// Set all rounds with round_number > given number (same story/task/stage)
/// to status = 'superseded'.
pub async fn supersede_rounds_after(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    story_id: Uuid,
    task_id: Option<Uuid>,
    stage: &str,
    after_round_number: i32,
) -> Result<u64, sqlx::Error> {
    let result = if let Some(tid) = task_id {
        sqlx::query(
            "UPDATE qa_rounds SET status = 'superseded', updated_at = now()
             WHERE story_id = $1 AND task_id = $2 AND stage = $3
               AND round_number > $4 AND status = 'active'",
        )
        .bind(story_id)
        .bind(tid)
        .bind(stage)
        .bind(after_round_number)
        .execute(&mut **tx)
        .await?
    } else {
        sqlx::query(
            "UPDATE qa_rounds SET status = 'superseded', updated_at = now()
             WHERE story_id = $1 AND task_id IS NULL AND stage = $2
               AND round_number > $3 AND status = 'active'",
        )
        .bind(story_id)
        .bind(stage)
        .bind(after_round_number)
        .execute(&mut **tx)
        .await?
    };
    Ok(result.rows_affected())
}
