use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::new_id;

#[derive(Debug, sqlx::FromRow)]
pub struct AgentSession {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub story_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub stage: String,
    pub role: Option<String>,
    pub session_id: Uuid,
    /// The `qa_rounds.id` this session will write questions into.
    /// Only set for grooming sessions that share a single round across all roles.
    pub qa_round_id: Option<Uuid>,
    pub responded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Creates a new agent session and returns the generated `session_id` (UUIDv7).
pub async fn create_session(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
    story_id: Option<Uuid>,
    task_id: Option<Uuid>,
    stage: &str,
    role: Option<&str>,
    qa_round_id: Option<Uuid>,
) -> Result<Uuid, sqlx::Error> {
    let id = new_id();
    let session_id = new_id();

    sqlx::query(
        r#"INSERT INTO agent_sessions
               (id, org_id, project_id, story_id, task_id, stage, role, session_id, qa_round_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(id)
    .bind(org_id)
    .bind(project_id)
    .bind(story_id)
    .bind(task_id)
    .bind(stage)
    .bind(role)
    .bind(session_id)
    .bind(qa_round_id)
    .execute(pool)
    .await?;

    Ok(session_id)
}

/// Loads an agent session by its unique `session_id`.
pub async fn load_session(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<Option<AgentSession>, sqlx::Error> {
    sqlx::query_as::<_, AgentSession>(
        "SELECT id, org_id, project_id, story_id, task_id, stage, role, session_id,
                qa_round_id, responded_at, created_at
         FROM agent_sessions WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
}

/// Mark a session as having delivered its output (set `responded_at = now()`).
pub async fn mark_session_responded(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE agent_sessions SET responded_at = now() WHERE session_id = $1",
    )
    .bind(session_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Returns `true` when every session that belongs to `qa_round_id` has already
/// responded (i.e. no session has `responded_at IS NULL`).
pub async fn all_sessions_for_round_responded(
    pool: &PgPool,
    qa_round_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM agent_sessions
         WHERE qa_round_id = $1 AND responded_at IS NULL",
    )
    .bind(qa_round_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0 == 0)
}
