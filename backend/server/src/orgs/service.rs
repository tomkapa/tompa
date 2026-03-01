use uuid::Uuid;

use crate::{errors::ApiError, state::AppState};

use super::{
    repo,
    types::{CreateOrgRequest, OrgError, OrgResponse},
};

pub async fn list_orgs(state: &AppState, user_id: Uuid) -> Result<Vec<OrgResponse>, ApiError> {
    let rows = repo::list_orgs_for_user(&state.pool, user_id).await?;
    Ok(rows
        .into_iter()
        .map(|r| OrgResponse {
            id: r.id,
            name: r.name,
            role: r.role,
            created_at: r.created_at,
        })
        .collect())
}

pub async fn create_org(
    state: &AppState,
    user_id: Uuid,
    req: CreateOrgRequest,
) -> Result<OrgResponse, ApiError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(OrgError::NameRequired.into());
    }
    let (org_id, created_at) = repo::create_org(&state.pool, &name).await?;
    repo::add_org_member(&state.pool, org_id, user_id, "owner").await?;
    Ok(OrgResponse {
        id: org_id,
        name,
        role: "owner".to_string(),
        created_at,
    })
}
