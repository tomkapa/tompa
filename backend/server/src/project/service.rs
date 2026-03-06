use uuid::Uuid;

use crate::{agents::prompts::grooming::GROOMING_ROLES, db::OrgTx, errors::ApiError};

use super::{
    repo,
    types::{CreateProjectRequest, ProjectError, ProjectResponse, UpdateProjectRequest},
};

fn unique_violation_to_name_taken(e: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(ref db_err) = e
        && db_err.code().as_deref() == Some("23505")
    {
        return ProjectError::NameTaken.into();
    }
    e.into()
}

fn to_response(row: repo::ProjectRow) -> ProjectResponse {
    ProjectResponse {
        id: row.id,
        org_id: row.org_id,
        name: row.name,
        description: row.description,
        github_repo_url: row.github_repo_url,
        grooming_roles: row.grooming_roles,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn validate_grooming_roles(roles: &[String]) -> Result<(), ApiError> {
    if !roles.iter().any(|r| r == "business_analyst") {
        return Err(ProjectError::BusinessAnalystRequired.into());
    }
    for role in roles {
        if !GROOMING_ROLES.iter().any(|gr| gr.id == role.as_str()) {
            return Err(ProjectError::InvalidRoleId.into());
        }
    }
    Ok(())
}

pub async fn list_projects(tx: &mut OrgTx) -> Result<Vec<ProjectResponse>, ApiError> {
    let org_id = tx.org_id;
    let rows = repo::list_projects(tx, org_id).await?;
    Ok(rows.into_iter().map(to_response).collect())
}

pub async fn get_project(tx: &mut OrgTx, id: Uuid) -> Result<ProjectResponse, ApiError> {
    let org_id = tx.org_id;
    let row = repo::get_project(tx, id, org_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(to_response(row))
}

pub async fn create_project(
    tx: &mut OrgTx,
    req: CreateProjectRequest,
) -> Result<ProjectResponse, ApiError> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(ProjectError::NameRequired.into());
    }
    let org_id = tx.org_id;
    let row = repo::create_project(
        tx,
        org_id,
        &name,
        req.description.as_deref(),
        req.github_repo_url.as_deref(),
    )
    .await
    .map_err(unique_violation_to_name_taken)?;
    Ok(to_response(row))
}

pub async fn update_project(
    tx: &mut OrgTx,
    id: Uuid,
    req: UpdateProjectRequest,
) -> Result<ProjectResponse, ApiError> {
    if let Some(ref n) = req.name
        && n.trim().is_empty()
    {
        return Err(ProjectError::NameRequired.into());
    }
    if let Some(ref roles) = req.grooming_roles {
        validate_grooming_roles(roles)?;
    }
    let org_id = tx.org_id;
    let row = repo::update_project(
        tx,
        id,
        org_id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.github_repo_url.as_deref(),
        req.grooming_roles,
    )
    .await
    .map_err(unique_violation_to_name_taken)?
    .ok_or(ApiError::NotFound)?;
    Ok(to_response(row))
}

pub async fn delete_project(tx: &mut OrgTx, id: Uuid) -> Result<(), ApiError> {
    let org_id = tx.org_id;
    let deleted = repo::soft_delete_project(tx, id, org_id).await?;
    if !deleted {
        return Err(ApiError::NotFound);
    }
    Ok(())
}
