use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::db::new_id;

#[derive(sqlx::FromRow)]
pub struct KeyRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub project_id: Uuid,
    pub key_hash: String,
    pub label: String,
    pub container_mode: String,
    pub last_connected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// List all keys for a project (including revoked ones so the UI can show
/// revocation timestamps). RLS ensures only the org's keys are visible.
pub async fn list_keys(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    project_id: Uuid,
) -> Result<Vec<KeyRow>, sqlx::Error> {
    sqlx::query_as::<_, KeyRow>(
        r#"
        SELECT id, org_id, project_id, key_hash, label, container_mode,
               last_connected_at, created_at, revoked_at
        FROM container_api_keys
        WHERE project_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(project_id)
    .fetch_all(&mut **tx)
    .await
}

/// Insert a new key row and return it.
pub async fn insert_key(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    org_id: Uuid,
    project_id: Uuid,
    key_hash: &str,
    label: &str,
    container_mode: &str,
) -> Result<KeyRow, sqlx::Error> {
    let id = new_id();
    sqlx::query_as::<_, KeyRow>(
        r#"
        INSERT INTO container_api_keys
            (id, org_id, project_id, key_hash, label, container_mode)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, org_id, project_id, key_hash, label, container_mode,
                  last_connected_at, created_at, revoked_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(project_id)
    .bind(key_hash)
    .bind(label)
    .bind(container_mode)
    .fetch_one(&mut **tx)
    .await
}

/// Set `revoked_at = now()` on a non-revoked key.
/// Returns `true` if a row was updated.
pub async fn revoke_key(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE container_api_keys
        SET revoked_at = now()
        WHERE id = $1
          AND revoked_at IS NULL
        "#,
    )
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Fetch all non-revoked keys across all orgs for verification.
///
/// This runs directly on the pool without a transaction-scoped `SET LOCAL
/// app.org_id`, so Row-Level Security does not filter rows. The DB user must
/// own the `container_api_keys` table (or hold BYPASSRLS) for this to work —
/// which is the standard setup since migrations run as the app user.
pub async fn list_active_key_hashes(pool: &PgPool) -> Result<Vec<KeyRow>, sqlx::Error> {
    sqlx::query_as::<_, KeyRow>(
        r#"
        SELECT id, org_id, project_id, key_hash, label, container_mode,
               last_connected_at, created_at, revoked_at
        FROM container_api_keys
        WHERE revoked_at IS NULL
        "#,
    )
    .fetch_all(pool)
    .await
}

/// Stamp `last_connected_at` when a container successfully authenticates.
pub async fn update_last_connected(pool: &PgPool, key_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE container_api_keys
        SET last_connected_at = now()
        WHERE id = $1
        "#,
    )
    .bind(key_id)
    .execute(pool)
    .await?;
    Ok(())
}
