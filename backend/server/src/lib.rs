pub mod agents;
pub mod auth;
pub mod config;
pub mod container_keys;
pub mod db;
pub mod errors;
pub mod knowledge;
pub mod openapi;
pub mod orgs;
pub mod project;
pub mod qa;
pub mod sse;
pub mod state;
pub mod story;
pub mod task;

use axum::{routing::get, Router};

use state::AppState;

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/v1/openapi.json", get(openapi::openapi_handler))
        .merge(auth::router(state.clone()))
        .merge(orgs::handler::router(state.clone()))
        .merge(project::handler::router(state.clone()))
        .merge(knowledge::handler::router(state.clone()))
        .merge(container_keys::handler::router(state.clone()))
        .merge(story::handler::router(state.clone()))
        .merge(task::handler::router(state.clone()))
        .merge(qa::handler::router(state.clone()))
        .merge(agents::handler::router(state.clone()))
        .merge(sse::handler::router(state.clone()))
        .with_state(state)
}
