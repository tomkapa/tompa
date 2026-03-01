use chrono::{DateTime, Utc};
use sqlx::Postgres;
use uuid::Uuid;

use crate::db::new_id;

#[derive(sqlx::FromRow)]
pub struct KnowledgeRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Option<Uuid>,
    pub story_id: Option<Uuid>,
    pub category: String,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// List knowledge entries for the given scope.
///
/// Hierarchy returned:
/// - Always includes org-level entries (project_id IS NULL).
/// - If `project_id` is Some, also includes project-level entries.
/// - If both `project_id` and `story_id` are Some, also includes story-level entries.
pub async fn list_knowledge(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    project_id: Option<Uuid>,
    story_id: Option<Uuid>,
) -> Result<Vec<KnowledgeRow>, sqlx::Error> {
    sqlx::query_as::<_, KnowledgeRow>(
        r#"
        SELECT id, org_id, project_id, story_id, category, title, content, created_at, updated_at
        FROM knowledge_entries
        WHERE deleted_at IS NULL
          AND (
              (project_id IS NULL AND story_id IS NULL)
              OR ($1::uuid IS NOT NULL AND project_id = $1 AND story_id IS NULL)
              OR ($1::uuid IS NOT NULL AND $2::uuid IS NOT NULL AND project_id = $1 AND story_id = $2)
          )
        ORDER BY created_at
        "#,
    )
    .bind(project_id)
    .bind(story_id)
    .fetch_all(&mut **tx)
    .await
}

/// Fetch a single knowledge entry by id.
/// RLS ensures it belongs to the org in the session context.
pub async fn get_knowledge(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<Option<KnowledgeRow>, sqlx::Error> {
    sqlx::query_as::<_, KnowledgeRow>(
        r#"
        SELECT id, org_id, project_id, story_id, category, title, content, created_at, updated_at
        FROM knowledge_entries
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
}

/// Insert a new knowledge entry.
pub async fn create_knowledge(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    project_id: Option<Uuid>,
    story_id: Option<Uuid>,
    category: &str,
    title: &str,
    content: &str,
) -> Result<KnowledgeRow, sqlx::Error> {
    let id = new_id();
    sqlx::query_as::<_, KnowledgeRow>(
        r#"
        INSERT INTO knowledge_entries (id, org_id, project_id, story_id, category, title, content)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, org_id, project_id, story_id, category, title, content, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(project_id)
    .bind(story_id)
    .bind(category)
    .bind(title)
    .bind(content)
    .fetch_one(&mut **tx)
    .await
}

/// Partial update of a knowledge entry.
/// Fields that are `None` are left unchanged (COALESCE semantics).
pub async fn update_knowledge(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    title: Option<&str>,
    content: Option<&str>,
    category: Option<&str>,
) -> Result<Option<KnowledgeRow>, sqlx::Error> {
    sqlx::query_as::<_, KnowledgeRow>(
        r#"
        UPDATE knowledge_entries
        SET
            title      = COALESCE($2, title),
            content    = COALESCE($3, content),
            category   = COALESCE($4, category),
            updated_at = now()
        WHERE id = $1
          AND deleted_at IS NULL
        RETURNING id, org_id, project_id, story_id, category, title, content, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(content)
    .bind(category)
    .fetch_optional(&mut **tx)
    .await
}

/// Soft-delete a knowledge entry by setting deleted_at.
/// Returns true if a row was affected.
pub async fn soft_delete_knowledge(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE knowledge_entries
        SET deleted_at = now()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}
