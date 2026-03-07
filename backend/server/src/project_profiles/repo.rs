use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::db::new_id;

#[derive(Debug, sqlx::FromRow)]
pub struct ProjectProfileRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub content: serde_json::Value,
    pub patterns_at_generation: i32,
    pub generated_by: String,
    pub generated_at: Option<DateTime<Utc>>,
    pub edited_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fetch the current project profile (pool-level, no RLS — for use in agents/service).
pub async fn get_profile_by_project(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
) -> Result<Option<ProjectProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectProfileRow>(
        r#"
        SELECT id, org_id, project_id, content, patterns_at_generation,
               generated_by, generated_at, edited_at, created_at, updated_at
        FROM project_profiles
        WHERE org_id = $1 AND project_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(org_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await
}

/// Fetch profile within a transaction (RLS-scoped).
pub async fn get_profile(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    project_id: Uuid,
) -> Result<Option<ProjectProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectProfileRow>(
        r#"
        SELECT id, org_id, project_id, content, patterns_at_generation,
               generated_by, generated_at, edited_at, created_at, updated_at
        FROM project_profiles
        WHERE org_id = $1 AND project_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(org_id)
    .bind(project_id)
    .fetch_optional(&mut **tx)
    .await
}

/// UPSERT a project profile (auto-generated).
pub async fn upsert_profile(
    pool: &PgPool,
    org_id: Uuid,
    project_id: Uuid,
    content: &serde_json::Value,
    patterns_at_generation: i32,
    generated_by: &str,
) -> Result<ProjectProfileRow, sqlx::Error> {
    let id = new_id();
    sqlx::query_as::<_, ProjectProfileRow>(
        r#"
        INSERT INTO project_profiles
            (id, org_id, project_id, content, patterns_at_generation, generated_by, generated_at)
        VALUES ($1, $2, $3, $4, $5, $6, now())
        ON CONFLICT (project_id)
        DO UPDATE SET
            content = EXCLUDED.content,
            patterns_at_generation = EXCLUDED.patterns_at_generation,
            generated_by = EXCLUDED.generated_by,
            generated_at = now(),
            updated_at = now()
        RETURNING id, org_id, project_id, content, patterns_at_generation,
                  generated_by, generated_at, edited_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(project_id)
    .bind(content)
    .bind(patterns_at_generation)
    .bind(generated_by)
    .fetch_one(pool)
    .await
}

/// User manual edit of a profile.
pub async fn update_profile_content(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    project_id: Uuid,
    content: &serde_json::Value,
) -> Result<Option<ProjectProfileRow>, sqlx::Error> {
    // First try to update existing
    let updated = sqlx::query_as::<_, ProjectProfileRow>(
        r#"
        UPDATE project_profiles
        SET content = $3, edited_at = now(), updated_at = now(), generated_by = 'manual'
        WHERE org_id = $1 AND project_id = $2 AND deleted_at IS NULL
        RETURNING id, org_id, project_id, content, patterns_at_generation,
                  generated_by, generated_at, edited_at, created_at, updated_at
        "#,
    )
    .bind(org_id)
    .bind(project_id)
    .bind(content)
    .fetch_optional(&mut **tx)
    .await?;

    if updated.is_some() {
        return Ok(updated);
    }

    // If no profile exists yet, create one
    let id = new_id();
    let row = sqlx::query_as::<_, ProjectProfileRow>(
        r#"
        INSERT INTO project_profiles
            (id, org_id, project_id, content, generated_by, edited_at)
        VALUES ($1, $2, $3, $4, 'manual', now())
        RETURNING id, org_id, project_id, content, patterns_at_generation,
                  generated_by, generated_at, edited_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(project_id)
    .bind(content)
    .fetch_one(&mut **tx)
    .await?;

    Ok(Some(row))
}
