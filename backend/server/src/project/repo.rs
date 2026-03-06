use chrono::{DateTime, Utc};
use sqlx::Postgres;
use uuid::Uuid;

use crate::db::new_id;

#[derive(sqlx::FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub github_repo_url: Option<String>,
    pub grooming_roles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// List projects in an org, scoped via RLS transaction.
/// Membership enforcement is delegated to RLS (app.org_id already set by caller).
pub async fn list_projects(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, org_id, name, description, github_repo_url, grooming_roles, created_at, updated_at
        FROM projects
        WHERE deleted_at IS NULL
          AND org_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(org_id)
    .fetch_all(&mut **tx)
    .await
}

/// Fetch a single project by id.
/// RLS ensures the project belongs to the org in the session context.
pub async fn get_project(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, org_id, name, description, github_repo_url, grooming_roles, created_at, updated_at
        FROM projects
        WHERE id = $1
          AND deleted_at IS NULL
          AND org_id = $2
        "#,
    )
    .bind(id)
    .bind(org_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Insert a new project.
pub async fn create_project(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    name: &str,
    description: Option<&str>,
    github_repo_url: Option<&str>,
) -> Result<ProjectRow, sqlx::Error> {
    let id = new_id();
    sqlx::query_as::<_, ProjectRow>(
        r#"
        INSERT INTO projects (id, org_id, name, description, github_repo_url)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, org_id, name, description, github_repo_url, grooming_roles, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(name)
    .bind(description)
    .bind(github_repo_url)
    .fetch_one(&mut **tx)
    .await
}

/// Partial update of a project.
/// Fields that are `None` are left unchanged (COALESCE semantics).
pub async fn update_project(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
    name: Option<&str>,
    description: Option<&str>,
    github_repo_url: Option<&str>,
    grooming_roles: Option<Vec<String>>,
) -> Result<Option<ProjectRow>, sqlx::Error> {
    sqlx::query_as::<_, ProjectRow>(
        r#"
        UPDATE projects
        SET
            name            = COALESCE($2, name),
            description     = COALESCE($3, description),
            github_repo_url = COALESCE($4, github_repo_url),
            grooming_roles  = COALESCE($5, grooming_roles),
            updated_at      = now()
        WHERE id = $1
          AND deleted_at IS NULL
          AND org_id = $6
        RETURNING id, org_id, name, description, github_repo_url, grooming_roles, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .bind(github_repo_url)
    .bind(grooming_roles)
    .bind(org_id)
    .fetch_optional(&mut **tx)
    .await
}

/// Soft-delete a project by setting deleted_at.
/// Returns true if a row was affected.
pub async fn soft_delete_project(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
    org_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE projects
        SET deleted_at = now()
        WHERE id = $1
          AND deleted_at IS NULL
          AND org_id = $2
        "#,
    )
    .bind(id)
    .bind(org_id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}
