//! Integration tests for the container-keys CRUD endpoints and verify_api_key.
//!
//! Requires a live Postgres instance. Run with:
//!   BCRYPT_COST=4 DATABASE_URL=postgres://... cargo test --test container_keys
//!
//! Set BCRYPT_COST=4 to keep bcrypt fast in tests (default is 12, ~300 ms/op).

mod common;

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use server::{
    agents::registry::DashMapRegistry,
    auth::{service::make_claims, types::AuthClaims},
    build_app,
    config::Config,
    container_keys::service::verify_api_key,
    sse::broadcaster::SseBroadcaster,
    state::AppState,
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

const TEST_SECRET: &str = "test-jwt-secret-container-keys";

// ── Helpers ───────────────────────────────────────────────────────────────────

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        config: Arc::new(Config {
            database_url: String::new(),
            port: 0,
            jwt_secret: TEST_SECRET.to_string(),
            google_client_id: String::new(),
            google_client_secret: String::new(),
            github_client_id: String::new(),
            github_client_secret: String::new(),
            oauth_redirect_base_url: String::new(),
        }),
        registry: Arc::new(DashMapRegistry::new()),
        broadcaster: Arc::new(SseBroadcaster::new()),
    }
}

fn test_jwt(user_id: Uuid, org_id: Uuid) -> String {
    let claims = make_claims(user_id, org_id, "owner");
    let claims = AuthClaims {
        exp: 9_999_999_999,
        ..claims
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
    )
    .unwrap()
}

async fn seed_user(pool: &PgPool) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, oauth_provider, oauth_provider_id)
         VALUES ($1, $2, $3, 'test', $4)",
    )
    .bind(id)
    .bind(format!("u-{id}@t.example"))
    .bind("Test User")
    .bind(id.to_string())
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn seed_org(pool: &PgPool, user_id: Uuid) -> Uuid {
    let org_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind(format!("Org-{org_id}"))
        .execute(pool)
        .await
        .unwrap();
    let mid = Uuid::now_v7();
    sqlx::query("INSERT INTO org_members (id, org_id, user_id, role) VALUES ($1, $2, $3, 'owner')")
        .bind(mid)
        .bind(org_id)
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    org_id
}

async fn seed_project(pool: &PgPool, org_id: Uuid) -> Uuid {
    let pid = Uuid::now_v7();
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(pid)
        .bind(org_id)
        .bind("Test Project")
        .execute(pool)
        .await
        .unwrap();
    pid
}

fn json_req(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let bytes = body
        .map(|v| serde_json::to_vec(&v).unwrap())
        .unwrap_or_default();
    Request::builder()
        .method(method)
        .uri(uri)
        .header("cookie", format!("session={token}"))
        .header("content-type", "application/json")
        .body(Body::from(bytes))
        .unwrap()
}

async fn resp_json(resp: Response) -> Value {
    let b = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&b).unwrap_or(Value::Null)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_key_returns_raw_key_once(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_req(
            "POST",
            "/api/v1/container-keys",
            &token,
            Some(json!({
                "project_id": project_id,
                "label": "My Agent",
                "container_mode": "dev"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = resp_json(resp).await;
    let api_key = body["api_key"].as_str().unwrap();
    assert!(api_key.starts_with("cpk_"), "key should have cpk_ prefix");
    assert!(body["id"].is_string());
    assert_eq!(body["label"], "My Agent");
    assert_eq!(body["container_mode"], "dev");
    // raw key is returned only on creation — no hash exposed
    assert!(body.get("key_hash").is_none());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_keys_never_returns_raw_key_or_hash(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    // Create a key
    app.clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/container-keys",
            &token,
            Some(json!({
                "project_id": project_id,
                "label": "CI Runner",
                "container_mode": "standalone"
            })),
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(json_req(
            "GET",
            &format!("/api/v1/container-keys?project_id={project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(resp_json(resp).await).unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].get("api_key").is_none());
    assert!(list[0].get("key_hash").is_none());
    assert_eq!(list[0]["label"], "CI Runner");
    assert_eq!(list[0]["container_mode"], "standalone");
    assert!(list[0]["revoked_at"].is_null());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn revoke_key_sets_revoked_at(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    // Create
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/container-keys",
            &token,
            Some(json!({
                "project_id": project_id,
                "label": "Doomed Key",
                "container_mode": "project"
            })),
        ))
        .await
        .unwrap();
    let created = resp_json(resp).await;
    let key_id = created["id"].as_str().unwrap();

    // Revoke
    let resp = app
        .clone()
        .oneshot(json_req(
            "DELETE",
            &format!("/api/v1/container-keys/{key_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Second revoke → 404
    let resp = app
        .clone()
        .oneshot(json_req(
            "DELETE",
            &format!("/api/v1/container-keys/{key_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // List still shows it (with revoked_at set)
    let resp = app
        .oneshot(json_req(
            "GET",
            &format!("/api/v1/container-keys?project_id={project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    let list: Vec<Value> = serde_json::from_value(resp_json(resp).await).unwrap();
    assert_eq!(list.len(), 1);
    assert!(!list[0]["revoked_at"].is_null(), "revoked_at should be set");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_key_rejects_invalid_mode(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_req(
            "POST",
            "/api/v1/container-keys",
            &token,
            Some(json!({
                "project_id": project_id,
                "label": "Bad Mode",
                "container_mode": "turbo"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_key_rejects_empty_label(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_req(
            "POST",
            "/api/v1/container-keys",
            &token,
            Some(json!({
                "project_id": project_id,
                "label": "   ",
                "container_mode": "dev"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_key_rejects_unknown_project(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_req(
            "POST",
            "/api/v1/container-keys",
            &token,
            Some(json!({
                "project_id": Uuid::now_v7(),
                "label": "Ghost Key",
                "container_mode": "dev"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn container_keys_require_auth(pool: PgPool) {
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/container-keys?project_id=00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn verify_api_key_success_and_failure(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let state = test_state(pool.clone());
    let app = build_app(state);

    // Create a key via the HTTP endpoint (which uses the configured bcrypt cost)
    let resp = app
        .oneshot(json_req(
            "POST",
            "/api/v1/container-keys",
            &token,
            Some(json!({
                "project_id": project_id,
                "label": "Verifiable",
                "container_mode": "dev"
            })),
        ))
        .await
        .unwrap();
    let body = resp_json(resp).await;
    let raw_key = body["api_key"].as_str().unwrap().to_string();
    let key_id: Uuid = body["id"].as_str().unwrap().parse().unwrap();

    // Correct key → success
    let info = verify_api_key(&pool, &raw_key)
        .await
        .expect("verify should succeed");
    assert_eq!(info.key_id, key_id);
    assert_eq!(info.org_id, org_id);
    assert_eq!(info.project_id, project_id);
    assert_eq!(info.container_mode, "dev");

    // Wrong key → error
    assert!(
        verify_api_key(&pool, "cpk_wrongwrongwrongwrongwrongwrongwrongwrong")
            .await
            .is_err(),
        "wrong key must not verify"
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn revoked_key_is_excluded_from_verify(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    // Create
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/v1/container-keys",
            &token,
            Some(json!({
                "project_id": project_id,
                "label": "To Revoke",
                "container_mode": "project"
            })),
        ))
        .await
        .unwrap();
    let body = resp_json(resp).await;
    let raw_key = body["api_key"].as_str().unwrap().to_string();
    let key_id = body["id"].as_str().unwrap();

    // Revoke it
    app.oneshot(json_req(
        "DELETE",
        &format!("/api/v1/container-keys/{key_id}"),
        &token,
        None,
    ))
    .await
    .unwrap();

    // verify_api_key must now fail for the revoked key
    assert!(
        verify_api_key(&pool, &raw_key).await.is_err(),
        "revoked key must not verify"
    );
}
