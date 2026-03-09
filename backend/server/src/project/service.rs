use uuid::Uuid;

use crate::{agents::prompts::grooming::GROOMING_CONFIG, db::OrgTx, errors::ApiError};

use super::{
    repo,
    types::{
        CreateProjectRequest, ProjectError, ProjectResponse, UpdateProjectRequest,
        UpdateQaConfigRequest,
    },
};

const VALID_MODELS: &[&str] = &["haiku", "sonnet", "opus"];

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
        qa_config: row.qa_config,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn validate_role_config(role_id: &str, cfg: &serde_json::Value) -> Result<(), ProjectError> {
    let model = cfg
        .get("model")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProjectError::InvalidQaConfig(format!("{role_id}.model is required")))?;
    if !VALID_MODELS.contains(&model) {
        return Err(ProjectError::InvalidQaConfig(format!(
            "{role_id}.model must be one of haiku, sonnet, opus"
        )));
    }

    let detail_level = cfg
        .get("detail_level")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| {
            ProjectError::InvalidQaConfig(format!("{role_id}.detail_level is required"))
        })?;
    if !(1..=5).contains(&detail_level) {
        return Err(ProjectError::InvalidQaConfig(format!(
            "{role_id}.detail_level must be 1–5"
        )));
    }

    let max_questions = cfg
        .get("max_questions")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| {
            ProjectError::InvalidQaConfig(format!("{role_id}.max_questions is required"))
        })?;
    if !(1..=5).contains(&max_questions) {
        return Err(ProjectError::InvalidQaConfig(format!(
            "{role_id}.max_questions must be 1–5"
        )));
    }

    Ok(())
}

fn validate_qa_config(qa_config: &serde_json::Value) -> Result<(), ApiError> {
    let grooming = qa_config
        .get("grooming")
        .and_then(|v| v.as_object())
        .ok_or_else(|| ProjectError::InvalidQaConfig("grooming object is required".into()))?;

    if !grooming.contains_key("business_analyst") {
        return Err(
            ProjectError::InvalidQaConfig("grooming.business_analyst is required".into()).into(),
        );
    }

    let valid_role_ids: Vec<&str> = GROOMING_CONFIG.roles.iter().map(|r| r.id.as_str()).collect();
    for role_id in grooming.keys() {
        if !valid_role_ids.contains(&role_id.as_str()) {
            return Err(
                ProjectError::InvalidQaConfig(format!("unknown grooming role: {role_id}")).into(),
            );
        }
        validate_role_config(role_id, &grooming[role_id]).map_err(ApiError::from)?;
    }

    let planning = qa_config
        .get("planning")
        .ok_or_else(|| ProjectError::InvalidQaConfig("planning config is required".into()))?;
    validate_role_config("planning", planning).map_err(ApiError::from)?;

    let implementation = qa_config
        .get("implementation")
        .ok_or_else(|| ProjectError::InvalidQaConfig("implementation config is required".into()))?;
    validate_role_config("implementation", implementation).map_err(ApiError::from)?;

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
    let org_id = tx.org_id;
    let row = repo::update_project(
        tx,
        id,
        org_id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.github_repo_url.as_deref(),
    )
    .await
    .map_err(unique_violation_to_name_taken)?
    .ok_or(ApiError::NotFound)?;
    Ok(to_response(row))
}

pub async fn update_qa_config(
    tx: &mut OrgTx,
    id: Uuid,
    req: UpdateQaConfigRequest,
) -> Result<ProjectResponse, ApiError> {
    validate_qa_config(&req.qa_config)?;
    let org_id = tx.org_id;
    let row = repo::update_qa_config(tx, id, org_id, &req.qa_config)
        .await?
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
