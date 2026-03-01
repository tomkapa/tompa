use uuid::Uuid;

use crate::{
    auth::middleware::set_org_context,
    errors::ApiError,
    state::AppState,
};

use super::{
    repo,
    types::{CreateProjectRequest, ProjectError, ProjectResponse, UpdateProjectRequest},
};

fn to_response(row: repo::ProjectRow) -> ProjectResponse {
    ProjectResponse {
        id: row.id,
        org_id: row.org_id,
        name: row.name,
        description: row.description,
        github_repo_url: row.github_repo_url,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

pub async fn list_projects(
    state: &AppState,
    org_id: Uuid,
) -> Result<Vec<ProjectResponse>, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;
    let rows = repo::list_projects(&mut tx).await?;
    tx.commit().await?;
    Ok(rows.into_iter().map(to_response).collect())
}

pub async fn get_project(
    state: &AppState,
    org_id: Uuid,
    id: Uuid,
) -> Result<ProjectResponse, ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;
    let row = repo::get_project(&mut tx, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    tx.commit().await?;
    Ok(to_response(row))
}

pub async fn create_project(
    state: &AppState,
    org_id: Uuid,
    req: CreateProjectRequest,
) -> Result<ProjectResponse, ApiError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(ProjectError::NameRequired.into());
    }
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;
    let row = repo::create_project(
        &mut tx,
        org_id,
        &name,
        req.description.as_deref(),
        req.github_repo_url.as_deref(),
    )
    .await?;
    tx.commit().await?;
    Ok(to_response(row))
}

pub async fn update_project(
    state: &AppState,
    org_id: Uuid,
    id: Uuid,
    req: UpdateProjectRequest,
) -> Result<ProjectResponse, ApiError> {
    if let Some(ref n) = req.name {
        if n.trim().is_empty() {
            return Err(ProjectError::NameRequired.into());
        }
    }
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;
    let row = repo::update_project(
        &mut tx,
        id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.github_repo_url.as_deref(),
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    tx.commit().await?;
    Ok(to_response(row))
}

pub async fn delete_project(
    state: &AppState,
    org_id: Uuid,
    id: Uuid,
) -> Result<(), ApiError> {
    let mut tx = state.pool.begin().await?;
    set_org_context(&mut tx, org_id).await?;
    let deleted = repo::soft_delete_project(&mut tx, id).await?;
    tx.commit().await?;
    if !deleted {
        return Err(ApiError::NotFound);
    }
    Ok(())
}
