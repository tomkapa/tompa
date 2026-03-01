use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::new_id;

#[derive(sqlx::FromRow)]
pub struct OrgRow {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

/// List all orgs the user belongs to, ordered by membership creation time.
/// Queries with an explicit user_id join — does not rely on RLS for scoping.
pub async fn list_orgs_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<OrgRow>, sqlx::Error> {
    sqlx::query_as::<_, OrgRow>(
        r#"
        SELECT o.id, o.name, om.role, o.created_at
        FROM organizations o
        JOIN org_members om ON o.id = om.org_id
        WHERE om.user_id = $1
          AND o.deleted_at IS NULL
        ORDER BY om.created_at
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Insert a new organization row and return its id and created_at.
pub async fn create_org(pool: &PgPool, name: &str) -> Result<(Uuid, DateTime<Utc>), sqlx::Error> {
    let id = new_id();
    let created_at: DateTime<Utc> = sqlx::query_scalar(
        "INSERT INTO organizations (id, name) VALUES ($1, $2) RETURNING created_at",
    )
    .bind(id)
    .bind(name)
    .fetch_one(pool)
    .await?;
    Ok((id, created_at))
}

/// Add a user to an org with the given role.
pub async fn add_org_member(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> Result<(), sqlx::Error> {
    let id = new_id();
    sqlx::query("INSERT INTO org_members (id, org_id, user_id, role) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(org_id)
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await?;
    Ok(())
}

/// Return true if user_id is a member of org_id.
pub async fn is_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM org_members WHERE org_id = $1 AND user_id = $2)",
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}
