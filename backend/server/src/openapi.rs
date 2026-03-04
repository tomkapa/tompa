use axum::{Json, response::IntoResponse};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};

pub async fn openapi_handler() -> impl IntoResponse {
    Json(ApiDoc::openapi())
}

struct CookieAuth;

impl Modify for CookieAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "cookieAuth",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("session"))),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Tompa API", version = "1.0.0"),
    paths(
        crate::auth::handler::login,
        crate::auth::handler::callback,
        crate::auth::handler::logout,
        crate::auth::handler::me,
        crate::auth::handler::dev_login,
        crate::orgs::handler::list_orgs,
        crate::orgs::handler::create_org,
        crate::project::handler::list_projects,
        crate::project::handler::create_project,
        crate::project::handler::get_project,
        crate::project::handler::update_project,
        crate::project::handler::delete_project,
        crate::story::handler::list_stories,
        crate::story::handler::create_story,
        crate::story::handler::get_story,
        crate::story::handler::update_story,
        crate::story::handler::delete_story,
        crate::story::handler::update_rank,
        crate::story::handler::start_story,
        crate::story::handler::approve_description,
        crate::task::handler::list_tasks,
        crate::task::handler::create_task,
        crate::task::handler::get_task,
        crate::task::handler::update_task,
        crate::task::handler::delete_task,
        crate::task::handler::mark_done,
        crate::task::handler::list_dependencies,
        crate::task::handler::create_dependency,
        crate::task::handler::delete_dependency,
        crate::qa::handler::list_rounds,
        crate::qa::handler::submit_answer,
        crate::qa::handler::rollback,
        crate::qa::handler::course_correct,
        crate::knowledge::handler::list_knowledge,
        crate::knowledge::handler::create_knowledge,
        crate::knowledge::handler::update_knowledge,
        crate::knowledge::handler::delete_knowledge,
        crate::container_keys::handler::list_keys,
        crate::container_keys::handler::create_key,
        crate::container_keys::handler::revoke_key,
    ),
    components(schemas(
        crate::auth::types::MeResponse,
        crate::auth::handler::DevLoginRequest,
        crate::orgs::types::CreateOrgRequest,
        crate::orgs::types::OrgResponse,
        crate::project::types::CreateProjectRequest,
        crate::project::types::UpdateProjectRequest,
        crate::project::types::ProjectResponse,
        crate::story::types::CreateStoryRequest,
        crate::story::types::UpdateStoryRequest,
        crate::story::types::RankUpdateRequest,
        crate::story::types::StoryResponse,
        crate::story::types::ApproveRefinedDescriptionRequest,
        crate::story::types::TaskSummary,
        crate::task::types::CreateTaskRequest,
        crate::task::types::UpdateTaskRequest,
        crate::task::types::TaskResponse,
        crate::task::types::CreateDependencyRequest,
        crate::task::types::DependencyResponse,
        crate::qa::types::SubmitAnswerRequest,
        crate::qa::types::CourseCorrectionRequest,
        crate::qa::types::QaRoundResponse,
        crate::qa::types::QaContent,
        crate::qa::types::QaQuestionOption,
        crate::qa::types::QaQuestion,
        crate::knowledge::types::CreateKnowledgeRequest,
        crate::knowledge::types::UpdateKnowledgeRequest,
        crate::knowledge::types::KnowledgeResponse,
        crate::container_keys::types::CreateKeyRequest,
        crate::container_keys::types::CreateKeyResponse,
        crate::container_keys::types::KeyListItem,
    )),
    modifiers(&CookieAuth),
    tags(
        (name = "auth", description = "Authentication — OAuth login, logout, current user"),
        (name = "orgs", description = "Organizations — create and list orgs"),
        (name = "projects", description = "Projects — CRUD for projects within an org"),
        (name = "stories", description = "Stories — backlog management with fractional ranking"),
        (name = "tasks", description = "Tasks — atomic implementation units with dependency edges"),
        (name = "qa", description = "Q&A Rounds — structured question/answer pipeline"),
        (name = "knowledge", description = "Knowledge Base — org/project/story-scoped knowledge entries"),
        (name = "container-keys", description = "Container API Keys — keys for agent container authentication"),
    )
)]
pub struct ApiDoc;
