use uuid::Uuid;

use crate::{db::OrgTx, errors::ApiError};

use super::{
    repo,
    types::{ProjectProfileContent, ProjectProfileError, ProjectProfileResponse, UpdateProfileRequest},
};

fn to_response(row: repo::ProjectProfileRow) -> Result<ProjectProfileResponse, ApiError> {
    let content: ProjectProfileContent = serde_json::from_value(row.content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to parse profile content: {e}")))?;
    Ok(ProjectProfileResponse {
        id: row.id,
        org_id: row.org_id,
        project_id: row.project_id,
        content,
        patterns_at_generation: row.patterns_at_generation,
        generated_by: row.generated_by,
        generated_at: row.generated_at,
        edited_at: row.edited_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub async fn get_profile(
    tx: &mut OrgTx,
    project_id: Uuid,
) -> Result<ProjectProfileResponse, ApiError> {
    let org_id = tx.org_id;
    let row = repo::get_profile(tx, org_id, project_id)
        .await?
        .ok_or(ProjectProfileError::NotFound)?;
    to_response(row)
}

pub async fn update_profile(
    tx: &mut OrgTx,
    project_id: Uuid,
    req: UpdateProfileRequest,
) -> Result<ProjectProfileResponse, ApiError> {
    let org_id = tx.org_id;
    let content_json = serde_json::to_value(&req.content)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("failed to serialize profile content: {e}")))?;
    let row = repo::update_profile_content(tx, org_id, project_id, &content_json)
        .await?
        .ok_or(ProjectProfileError::NotFound)?;
    to_response(row)
}

/// Fetch the project profile content for prompt injection (pool-level, no RLS).
pub async fn fetch_project_profile(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    project_id: Uuid,
) -> Result<Option<ProjectProfileContent>, ApiError> {
    let row = repo::get_profile_by_project(pool, org_id, project_id).await?;
    match row {
        Some(r) => {
            let content: ProjectProfileContent = serde_json::from_value(r.content)
                .map_err(|e| {
                    ApiError::Internal(anyhow::anyhow!("failed to parse profile content: {e}"))
                })?;
            Ok(Some(content))
        }
        None => Ok(None),
    }
}
