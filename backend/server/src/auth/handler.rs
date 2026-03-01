use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{errors::ApiError, state::AppState};

use super::{
    service::{
        create_jwt, exchange_github_code, exchange_google_code, find_or_create_user, make_claims,
    },
    types::{AuthContext, MeResponse},
};

const SESSION_MAX_AGE: u64 = 7 * 24 * 3600; // 7 days

// ── Login ─────────────────────────────────────────────────────────────────────

/// GET /api/v1/auth/login/:provider
/// Redirects the browser to the OAuth consent screen.
#[utoipa::path(
    get,
    path = "/api/v1/auth/login/{provider}",
    tag = "auth",
    params(
        ("provider" = String, Path, description = "OAuth provider: `google` or `github`"),
    ),
    responses(
        (status = 302, description = "Redirect to OAuth consent screen"),
        (status = 400, description = "Unknown provider"),
    )
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Response, ApiError> {
    let url = match provider.as_str() {
        "google" => {
            let redirect_uri = format!(
                "{}/api/v1/auth/callback/google",
                state.config.oauth_redirect_base_url
            );
            format!(
                "https://accounts.google.com/o/oauth2/v2/auth\
                 ?client_id={}\
                 &redirect_uri={}\
                 &response_type=code\
                 &scope=openid%20email%20profile",
                percent_encode(&state.config.google_client_id),
                percent_encode(&redirect_uri),
            )
        }
        "github" => {
            let redirect_uri = format!(
                "{}/api/v1/auth/callback/github",
                state.config.oauth_redirect_base_url
            );
            format!(
                "https://github.com/login/oauth/authorize\
                 ?client_id={}\
                 &redirect_uri={}\
                 &scope=user:email",
                percent_encode(&state.config.github_client_id),
                percent_encode(&redirect_uri),
            )
        }
        _ => {
            return Err(ApiError::BadRequest(format!(
                "Unknown provider: {provider}"
            )));
        }
    };

    Ok(Redirect::to(&url).into_response())
}

// ── Callback ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
}

/// GET /api/v1/auth/callback/:provider
/// Exchanges the OAuth code, upserts the user, sets the session cookie.
#[utoipa::path(
    get,
    path = "/api/v1/auth/callback/{provider}",
    tag = "auth",
    params(
        ("provider" = String, Path, description = "OAuth provider: `google` or `github`"),
        ("code" = String, Query, description = "Authorization code from the OAuth provider"),
    ),
    responses(
        (status = 302, description = "Redirect to app root with session cookie set"),
        (status = 400, description = "Unknown provider"),
    )
)]
pub(crate) async fn callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(params): Query<CallbackParams>,
) -> Result<Response, ApiError> {
    let profile = match provider.as_str() {
        "google" => exchange_google_code(&params.code, &state.config).await,
        "github" => exchange_github_code(&params.code, &state.config).await,
        _ => {
            return Err(ApiError::BadRequest(format!(
                "Unknown provider: {provider}"
            )));
        }
    }
    .map_err(ApiError::Internal)?;

    let user_org = find_or_create_user(&state.pool, &profile)
        .await
        .map_err(ApiError::Internal)?;

    let claims = make_claims(user_org.user_id, user_org.org_id, &user_org.role);
    let token = create_jwt(&claims, &state.config.jwt_secret).map_err(ApiError::Internal)?;

    let is_secure = state.config.oauth_redirect_base_url.starts_with("https://");
    let cookie = build_session_cookie(&token, is_secure);

    let mut response = Redirect::to("/").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("JWT cookie value always forms a valid header"),
    );
    Ok(response)
}

// ── Logout ────────────────────────────────────────────────────────────────────

/// POST /api/v1/auth/logout
/// Clears the session cookie.
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Session cookie cleared"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn logout() -> impl IntoResponse {
    (
        [(
            header::SET_COOKIE,
            "session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0",
        )],
        StatusCode::OK,
    )
}

// ── Me ────────────────────────────────────────────────────────────────────────

/// GET /api/v1/auth/me
/// Returns the current user's profile and org membership info.
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current user profile", body = MeResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("cookieAuth" = []))
)]
pub(crate) async fn me(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<MeResponse>, ApiError> {
    let user = sqlx::query!(
        "SELECT id, email, display_name, avatar_url FROM users WHERE id = $1 AND deleted_at IS NULL",
        auth.user_id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?
    .ok_or(ApiError::Unauthorized)?;

    let org = sqlx::query!(
        "SELECT id, name FROM organizations WHERE id = $1 AND deleted_at IS NULL",
        auth.org_id,
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    Ok(Json(MeResponse {
        user_id: user.id,
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        org_id: auth.org_id,
        org_name: org.map(|o| o.name),
        role: auth.role,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_session_cookie(token: &str, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "session={token}; HttpOnly{secure_flag}; SameSite=Lax; Path=/; Max-Age={SESSION_MAX_AGE}"
    )
}

/// Percent-encode a string for use as a URL query parameter value.
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
