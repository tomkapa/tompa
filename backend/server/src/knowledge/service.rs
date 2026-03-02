use uuid::Uuid;

use crate::{db::OrgTx, errors::ApiError};

use super::{
    repo,
    types::{
        CreateKnowledgeRequest, KnowledgeError, KnowledgeResponse, UpdateKnowledgeRequest,
        is_valid_category,
    },
};

fn to_response(row: repo::KnowledgeRow) -> KnowledgeResponse {
    KnowledgeResponse {
        id: row.id,
        org_id: row.org_id,
        project_id: row.project_id,
        story_id: row.story_id,
        category: row.category,
        title: row.title,
        content: row.content,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

pub async fn list_knowledge(
    tx: &mut OrgTx,
    project_id: Option<Uuid>,
    story_id: Option<Uuid>,
) -> Result<Vec<KnowledgeResponse>, ApiError> {
    let org_id = tx.auth.org_id;
    let rows = repo::list_knowledge(tx, org_id, project_id, story_id).await?;
    Ok(rows.into_iter().map(to_response).collect())
}

pub async fn create_knowledge(
    tx: &mut OrgTx,
    req: CreateKnowledgeRequest,
) -> Result<KnowledgeResponse, ApiError> {
    let title = req.title.trim().to_string();
    if title.is_empty() {
        return Err(KnowledgeError::TitleRequired.into());
    }
    let content = req.content.trim().to_string();
    if content.is_empty() {
        return Err(KnowledgeError::ContentRequired.into());
    }
    let category = req.category.trim().to_string();
    if !is_valid_category(&category) {
        return Err(KnowledgeError::InvalidCategory.into());
    }
    let org_id = tx.auth.org_id;
    let row = repo::create_knowledge(
        tx,
        org_id,
        req.project_id,
        req.story_id,
        &category,
        &title,
        &content,
    )
    .await?;
    Ok(to_response(row))
}

pub async fn update_knowledge(
    tx: &mut OrgTx,
    id: Uuid,
    req: UpdateKnowledgeRequest,
) -> Result<KnowledgeResponse, ApiError> {
    if let Some(ref t) = req.title
        && t.trim().is_empty()
    {
        return Err(KnowledgeError::TitleRequired.into());
    }
    if let Some(ref c) = req.content
        && c.trim().is_empty()
    {
        return Err(KnowledgeError::ContentRequired.into());
    }
    if let Some(ref cat) = req.category
        && !is_valid_category(cat.trim())
    {
        return Err(KnowledgeError::InvalidCategory.into());
    }
    let org_id = tx.auth.org_id;
    let row = repo::update_knowledge(
        tx,
        id,
        org_id,
        req.title.as_deref(),
        req.content.as_deref(),
        req.category.as_deref(),
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok(to_response(row))
}

pub async fn delete_knowledge(tx: &mut OrgTx, id: Uuid) -> Result<(), ApiError> {
    let org_id = tx.auth.org_id;
    let deleted = repo::soft_delete_knowledge(tx, id, org_id).await?;
    if !deleted {
        return Err(ApiError::NotFound);
    }
    Ok(())
}
