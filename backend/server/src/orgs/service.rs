use uuid::Uuid;

use crate::{errors::ApiError, state::AppState};

use super::{
    repo,
    types::{CreateOrgRequest, OrgError, OrgMemberResponse, OrgResponse, UpdateOrgRequest},
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

pub async fn update_org(
    state: &AppState,
    org_id: Uuid,
    req: UpdateOrgRequest,
) -> Result<(), ApiError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(OrgError::NameRequired.into());
    }
    repo::rename_org(&state.pool, org_id, &name).await?;
    Ok(())
}

pub async fn list_members(
    state: &AppState,
    org_id: Uuid,
) -> Result<Vec<OrgMemberResponse>, ApiError> {
    let rows = repo::list_org_members(&state.pool, org_id).await?;
    Ok(rows
        .into_iter()
        .map(|r| OrgMemberResponse {
            user_id: r.user_id,
            display_name: r.display_name,
            avatar_url: r.avatar_url,
            role: r.role,
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
