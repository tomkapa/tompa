use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

use crate::{
    auth::types::AuthError, container_keys::types::ContainerKeyError,
    knowledge::types::KnowledgeError, orgs::types::OrgError, project::types::ProjectError,
    qa::types::QaError, story::types::StoryError, task::types::TaskError,
};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    Story(#[from] StoryError),
    #[error(transparent)]
    Task(#[from] TaskError),
    #[error(transparent)]
    Qa(#[from] QaError),
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error(transparent)]
    Org(#[from] OrgError),
    #[error(transparent)]
    Knowledge(#[from] KnowledgeError),
    #[error(transparent)]
    ContainerKey(#[from] ContainerKeyError),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not found")]
    NotFound,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

fn map_domain_error(err: &ApiError) -> (StatusCode, String) {
    match err {
        ApiError::Auth(_) => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
        ApiError::Story(e) => match e {
            StoryError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
            StoryError::InvalidTransition { .. } => (StatusCode::BAD_REQUEST, e.to_string()),
            StoryError::HasActiveTasks => (StatusCode::BAD_REQUEST, e.to_string()),
            StoryError::TitleRequired => (StatusCode::BAD_REQUEST, e.to_string()),
            StoryError::InvalidStoryType => (StatusCode::BAD_REQUEST, e.to_string()),
            StoryError::InvalidPipelineStage => (StatusCode::BAD_REQUEST, e.to_string()),
        },
        ApiError::Task(e) => match e {
            TaskError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
            TaskError::StoryNotFound => (StatusCode::NOT_FOUND, e.to_string()),
            TaskError::InvalidState { .. } => (StatusCode::BAD_REQUEST, e.to_string()),
            TaskError::CyclicDependency => (StatusCode::BAD_REQUEST, e.to_string()),
            TaskError::NotRunning => (StatusCode::BAD_REQUEST, e.to_string()),
            TaskError::DifferentStory => (StatusCode::BAD_REQUEST, e.to_string()),
        },
        ApiError::Qa(e) => match e {
            QaError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
            QaError::RoundNotActive => (StatusCode::BAD_REQUEST, e.to_string()),
            QaError::AlreadyAnswered => (StatusCode::BAD_REQUEST, e.to_string()),
            QaError::InvalidRollback => (StatusCode::BAD_REQUEST, e.to_string()),
            QaError::QuestionNotFound => (StatusCode::NOT_FOUND, e.to_string()),
            QaError::MissingFilter => (StatusCode::BAD_REQUEST, e.to_string()),
        },
        ApiError::Project(e) => match e {
            ProjectError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
            ProjectError::NameRequired => (StatusCode::BAD_REQUEST, e.to_string()),
            ProjectError::NameTaken => (StatusCode::CONFLICT, e.to_string()),
        },
        ApiError::Org(e) => match e {
            OrgError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
            OrgError::NameRequired => (StatusCode::BAD_REQUEST, e.to_string()),
        },
        ApiError::Knowledge(e) => match e {
            KnowledgeError::NotFound => (StatusCode::NOT_FOUND, e.to_string()),
            KnowledgeError::InvalidCategory => (StatusCode::BAD_REQUEST, e.to_string()),
            KnowledgeError::TitleRequired => (StatusCode::BAD_REQUEST, e.to_string()),
            KnowledgeError::ContentRequired => (StatusCode::BAD_REQUEST, e.to_string()),
        },
        ApiError::ContainerKey(e) => match e {
            ContainerKeyError::LabelRequired => (StatusCode::BAD_REQUEST, e.to_string()),
            ContainerKeyError::InvalidMode => (StatusCode::BAD_REQUEST, e.to_string()),
            ContainerKeyError::ProjectNotFound => (StatusCode::NOT_FOUND, e.to_string()),
        },
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        ),
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::Internal(anyhow::anyhow!(e))
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            _ => map_domain_error(&self),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
