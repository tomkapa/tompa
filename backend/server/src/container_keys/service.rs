use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::types::AuthError, db::OrgTx, errors::ApiError};

use super::{
    repo,
    types::{
        ContainerKeyError, ContainerKeyInfo, CreateKeyRequest, CreateKeyResponse, KeyListItem,
        VALID_MODES,
    },
};

/// bcrypt cost factor. Override via `BCRYPT_COST` env var (e.g. set to 4 in
/// integration tests to avoid ~300 ms per hash):
///   BCRYPT_COST=4 cargo test --test container_keys
fn bcrypt_cost() -> u32 {
    std::env::var("BCRYPT_COST")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(12)
}

/// Generate a 32-byte random key and format it as `cpk_<base64url>`.
fn generate_raw_key() -> String {
    let bytes: [u8; 32] = rand::random();
    format!("cpk_{}", URL_SAFE_NO_PAD.encode(bytes))
}

fn to_list_item(row: repo::KeyRow) -> KeyListItem {
    KeyListItem {
        id: row.id,
        label: row.label,
        container_mode: row.container_mode,
        last_connected_at: row.last_connected_at,
        created_at: row.created_at,
        revoked_at: row.revoked_at,
    }
}

pub async fn list_keys(tx: &mut OrgTx, project_id: Uuid) -> Result<Vec<KeyListItem>, ApiError> {
    let rows = repo::list_keys(tx, project_id).await?;
    Ok(rows.into_iter().map(to_list_item).collect())
}

pub async fn create_key(
    tx: &mut OrgTx,
    req: CreateKeyRequest,
) -> Result<CreateKeyResponse, ApiError> {
    let label = req.label.trim().to_string();
    if label.is_empty() {
        return Err(ContainerKeyError::LabelRequired.into());
    }
    if !VALID_MODES.contains(&req.container_mode.as_str()) {
        return Err(ContainerKeyError::InvalidMode.into());
    }

    // Hash on a blocking thread — bcrypt is CPU-intensive.
    let raw_key = generate_raw_key();
    let key_for_hash = raw_key.clone();
    let cost = bcrypt_cost();
    let hash = tokio::task::spawn_blocking(move || bcrypt::hash(&key_for_hash, cost))
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
        .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    let org_id = tx.org_id;

    // Validate the project exists in this org (RLS enforces org isolation).
    crate::project::repo::get_project(tx, req.project_id, org_id)
        .await?
        .ok_or(ContainerKeyError::ProjectNotFound)?;

    let row = repo::insert_key(
        tx,
        org_id,
        req.project_id,
        &hash,
        &label,
        &req.container_mode,
    )
    .await?;

    Ok(CreateKeyResponse {
        id: row.id,
        api_key: raw_key,
        label: row.label,
        container_mode: row.container_mode,
        created_at: row.created_at,
    })
}

pub async fn revoke_key(tx: &mut OrgTx, id: Uuid) -> Result<(), ApiError> {
    let revoked = repo::revoke_key(tx, id).await?;
    if !revoked {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

/// Verify a raw container API key against all non-revoked keys in the database.
///
/// Called by the WebSocket handler (T16) before the org context is known.
/// Bypasses RLS by running directly on the pool (the table owner skips RLS
/// when `FORCE ROW LEVEL SECURITY` is not set, which is our setup).
///
/// bcrypt is intentionally slow; each candidate is checked on a blocking
/// thread. Consider adding a key-prefix column for O(1) lookup if the key
/// count grows large.
pub async fn verify_api_key(pool: &PgPool, raw_key: &str) -> Result<ContainerKeyInfo, AuthError> {
    let rows = repo::list_active_key_hashes(pool)
        .await
        .map_err(|_| AuthError::InvalidToken)?;

    for row in rows {
        let hash = row.key_hash.clone();
        let key = raw_key.to_string();
        let matches = tokio::task::spawn_blocking(move || bcrypt::verify(&key, &hash))
            .await
            .map_err(|_| AuthError::InvalidToken)?
            .map_err(|_| AuthError::InvalidToken)?;

        if matches {
            return Ok(ContainerKeyInfo {
                key_id: row.id,
                org_id: row.org_id,
                project_id: row.project_id,
                container_mode: row.container_mode,
            });
        }
    }

    Err(AuthError::InvalidToken)
}
