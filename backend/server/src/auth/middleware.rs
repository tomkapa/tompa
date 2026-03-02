use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::{errors::ApiError, state::AppState};

use super::{service::validate_jwt, types::AuthContext};

/// Axum middleware that:
/// 1. Reads the JWT from the `session` HTTP-only cookie
/// 2. Validates signature and expiry
/// 3. Extracts AuthContext (user_id, org_id, role)
/// 4. Injects AuthContext as a request extension
///
/// RLS context is set automatically by the `OrgTx` extractor (see `db.rs`),
/// which begins a transaction and calls `set_config('app.org_id', ...)`.
pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_session_cookie(request.headers()).ok_or(ApiError::Unauthorized)?;

    let claims =
        validate_jwt(&token, &state.config.jwt_secret).map_err(|_| ApiError::Unauthorized)?;

    request.extensions_mut().insert(AuthContext {
        user_id: claims.sub,
        org_id: claims.org_id,
        role: claims.role,
    });

    Ok(next.run(request).await)
}

fn extract_session_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    let raw = headers.get("cookie")?.to_str().ok()?;
    raw.split(';')
        .map(str::trim)
        .find(|s| s.starts_with("session="))
        .map(|s| s["session=".len()..].to_string())
}
