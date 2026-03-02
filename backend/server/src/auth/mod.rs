pub mod handler;
pub mod middleware;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

use self::{
    handler::{callback, login, logout, me},
    middleware::require_auth,
};

pub fn router(state: AppState) -> Router<AppState> {
    let protected = Router::new()
        .route("/api/v1/auth/me", get(me))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth));

    Router::new()
        .route("/api/v1/auth/login/{provider}", get(login))
        .route("/api/v1/auth/callback/{provider}", get(callback))
        .route("/api/v1/auth/logout", post(logout))
        .merge(protected)
}
