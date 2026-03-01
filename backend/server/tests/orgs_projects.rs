//! Integration tests for the orgs and projects CRUD endpoints.
//!
//! Requires a live Postgres instance. `sqlx::test` creates an isolated database
//! per test and runs all migrations automatically.
//!
//! Run with:
//!   DATABASE_URL=postgres://... cargo test --test orgs_projects

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use server::{
    agents::registry::DashMapRegistry,
    sse::broadcaster::SseBroadcaster,
    auth::{service::make_claims, types::AuthClaims},
    build_app,
    config::Config,
    state::AppState,
};

/// Migrator used by all `#[sqlx::test]` cases in this file.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

// ── Test helpers ──────────────────────────────────────────────────────────────

const TEST_SECRET: &str = "test-jwt-secret-for-integration-tests";

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

fn test_jwt(user_id: Uuid, org_id: Uuid, role: &str) -> String {
    let claims = make_claims(user_id, org_id, role);
    // Override exp to be far in the future so tests don't expire
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
        r#"
        INSERT INTO users (id, email, display_name, oauth_provider, oauth_provider_id)
        VALUES ($1, $2, $3, 'test', $4)
        "#,
    )
    .bind(id)
    .bind(format!("user-{}@test.example", id))
    .bind("Test User")
    .bind(id.to_string())
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn seed_org(pool: &PgPool, user_id: Uuid, name: &str) -> Uuid {
    let org_id = Uuid::now_v7();
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
    let member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO org_members (id, org_id, user_id, role) VALUES ($1, $2, $3, 'owner')",
    )
    .bind(member_id)
    .bind(org_id)
    .bind(user_id)
    .execute(pool)
    .await
    .unwrap();
    org_id
}

fn json_request(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let body_bytes = body.map(|v| serde_json::to_vec(&v).unwrap()).unwrap_or_default();
    Request::builder()
        .method(method)
        .uri(uri)
        .header("cookie", format!("session={token}"))
        .header("content-type", "application/json")
        .body(Body::from(body_bytes))
        .unwrap()
}

async fn response_json(resp: Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

// ── Org tests ─────────────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_orgs_returns_empty_for_new_user(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let dummy_org = Uuid::now_v7(); // not a real org
    let token = test_jwt(user_id, dummy_org, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request("GET", "/api/v1/orgs", &token, None))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert_eq!(body, json!([]));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_org_and_list(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let dummy_org = Uuid::now_v7();
    let token = test_jwt(user_id, dummy_org, "owner");
    let app = build_app(test_state(pool));

    // Create
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/orgs",
            &token,
            Some(json!({ "name": "Acme Corp" })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = response_json(resp).await;
    assert_eq!(created["name"], "Acme Corp");
    assert_eq!(created["role"], "owner");

    // List
    let resp = app
        .oneshot(json_request("GET", "/api/v1/orgs", &token, None))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "Acme Corp");
    assert_eq!(list[0]["role"], "owner");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_org_rejects_empty_name(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let token = test_jwt(user_id, Uuid::now_v7(), "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/v1/orgs",
            &token,
            Some(json!({ "name": "   " })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn user_only_sees_own_orgs(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    let org_a = seed_org(&pool, user_a, "Org A").await;
    let _org_b = seed_org(&pool, user_b, "Org B").await;

    let token_a = test_jwt(user_a, org_a, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request("GET", "/api/v1/orgs", &token_a, None))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "Org A");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_orgs_requires_auth(pool: PgPool) {
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/orgs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── Project tests ─────────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_and_list_projects(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "My Org").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    // Create
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/projects",
            &token,
            Some(json!({
                "name": "My Project",
                "description": "A test project",
                "github_repo_url": "https://github.com/acme/myproject"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = response_json(resp).await;
    assert_eq!(created["name"], "My Project");
    assert_eq!(created["org_id"], org_id.to_string());
    let project_id = created["id"].as_str().unwrap().to_string();

    // List
    let resp = app
        .clone()
        .oneshot(json_request("GET", "/api/v1/projects", &token, None))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], project_id);

    // Get by id
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/projects/{project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let detail = response_json(resp).await;
    assert_eq!(detail["description"], "A test project");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_project_rejects_empty_name(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "My Org").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/v1/projects",
            &token,
            Some(json!({ "name": "" })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn update_project(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "My Org").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool.clone()));

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/projects",
            &token,
            Some(json!({ "name": "Old Name" })),
        ))
        .await
        .unwrap();

    let created = response_json(resp).await;
    let project_id = created["id"].as_str().unwrap().to_string();

    let resp = app
        .oneshot(json_request(
            "PATCH",
            &format!("/api/v1/projects/{project_id}"),
            &token,
            Some(json!({ "name": "New Name", "description": "Updated" })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let updated = response_json(resp).await;
    assert_eq!(updated["name"], "New Name");
    assert_eq!(updated["description"], "Updated");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn soft_delete_project(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "My Org").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/projects",
            &token,
            Some(json!({ "name": "Doomed Project" })),
        ))
        .await
        .unwrap();

    let created = response_json(resp).await;
    let project_id = created["id"].as_str().unwrap().to_string();

    // Delete
    let resp = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/projects/{project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Should not appear in list
    let resp = app
        .clone()
        .oneshot(json_request("GET", "/api/v1/projects", &token, None))
        .await
        .unwrap();

    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert!(list.is_empty());

    // Should 404 on direct get
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/projects/{project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn project_not_visible_to_other_org(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    let org_a = seed_org(&pool, user_a, "Org A").await;
    let org_b = seed_org(&pool, user_b, "Org B").await;

    let token_a = test_jwt(user_a, org_a, "owner");
    let token_b = test_jwt(user_b, org_b, "owner");
    let app = build_app(test_state(pool));

    // user_a creates a project
    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/projects",
            &token_a,
            Some(json!({ "name": "Secret Project" })),
        ))
        .await
        .unwrap();

    let created = response_json(resp).await;
    let project_id = created["id"].as_str().unwrap().to_string();

    // user_b cannot see it (RLS + membership)
    let resp = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/projects/{project_id}"),
            &token_b,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // user_b's project list is empty
    let resp = app
        .oneshot(json_request("GET", "/api/v1/projects", &token_b, None))
        .await
        .unwrap();

    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert!(list.is_empty());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn get_project_returns_404_for_unknown_id(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "My Org").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/projects/{}", Uuid::now_v7()),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
