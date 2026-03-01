//! Integration tests for the knowledge CRUD endpoints.
//!
//! Requires a live Postgres instance. `sqlx::test` creates an isolated database
//! per test and runs all migrations automatically.
//!
//! Run with:
//!   DATABASE_URL=postgres://... cargo test --test knowledge

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
    auth::{service::make_claims, types::AuthClaims},
    build_app,
    config::Config,
    sse::broadcaster::SseBroadcaster,
    state::AppState,
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

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
    .bind(format!("user-{id}@test.example"))
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
    sqlx::query("INSERT INTO org_members (id, org_id, user_id, role) VALUES ($1, $2, $3, 'owner')")
        .bind(Uuid::now_v7())
        .bind(org_id)
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    org_id
}

async fn seed_project(pool: &PgPool, org_id: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(org_id)
        .bind("Test Project")
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn seed_story(pool: &PgPool, org_id: Uuid, project_id: Uuid, user_id: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        r#"
        INSERT INTO stories (id, org_id, project_id, title, story_type, owner_id, rank)
        VALUES ($1, $2, $3, $4, 'feature', $5, 'a0')
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(project_id)
    .bind("Test Story")
    .bind(user_id)
    .execute(pool)
    .await
    .unwrap();
    id
}

fn json_request(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let body_bytes = body
        .map(|v| serde_json::to_vec(&v).unwrap())
        .unwrap_or_default();
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

// ── Create tests ──────────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_org_level_entry(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token,
            Some(json!({
                "category": "convention",
                "title": "Naming conventions",
                "content": "Use snake_case for variables."
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = response_json(resp).await;
    assert_eq!(body["category"], "convention");
    assert_eq!(body["title"], "Naming conventions");
    assert_eq!(body["org_id"], org_id.to_string());
    assert!(body["project_id"].is_null());
    assert!(body["story_id"].is_null());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_project_level_entry(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token,
            Some(json!({
                "project_id": project_id,
                "category": "adr",
                "title": "Use Postgres",
                "content": "We chose Postgres for ACID guarantees."
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = response_json(resp).await;
    assert_eq!(body["project_id"], project_id.to_string());
    assert!(body["story_id"].is_null());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_story_level_entry(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token,
            Some(json!({
                "project_id": project_id,
                "story_id": story_id,
                "category": "custom",
                "title": "Story context",
                "content": "This story implements the login flow."
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = response_json(resp).await;
    assert_eq!(body["project_id"], project_id.to_string());
    assert_eq!(body["story_id"], story_id.to_string());
}

// ── List / filter tests ───────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_returns_correct_hierarchy(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    // Create one entry at each level
    for (pj, st, title) in [
        (None, None, "org-level"),
        (Some(project_id), None, "project-level"),
        (Some(project_id), Some(story_id), "story-level"),
    ] {
        let mut body = json!({
            "category": "convention",
            "title": title,
            "content": "content"
        });
        if let Some(p) = pj {
            body["project_id"] = json!(p);
        }
        if let Some(s) = st {
            body["story_id"] = json!(s);
        }
        app.clone()
            .oneshot(json_request(
                "POST",
                "/api/v1/knowledge",
                &token,
                Some(body),
            ))
            .await
            .unwrap();
    }

    // No filter → only org-level
    let resp = app
        .clone()
        .oneshot(json_request("GET", "/api/v1/knowledge", &token, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["title"], "org-level");

    // project_id filter → org-level + project-level (not story-level)
    let resp = app
        .clone()
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/knowledge?project_id={project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert_eq!(list.len(), 2);
    let titles: Vec<&str> = list.iter().map(|v| v["title"].as_str().unwrap()).collect();
    assert!(titles.contains(&"org-level"));
    assert!(titles.contains(&"project-level"));

    // project_id + story_id → all three levels
    let resp = app
        .oneshot(json_request(
            "GET",
            &format!("/api/v1/knowledge?project_id={project_id}&story_id={story_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert_eq!(list.len(), 3);
}

// ── Update tests ──────────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn update_entry(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token,
            Some(json!({
                "category": "convention",
                "title": "Old Title",
                "content": "Old content"
            })),
        ))
        .await
        .unwrap();
    let created = response_json(resp).await;
    let id = created["id"].as_str().unwrap().to_string();

    let resp = app
        .oneshot(json_request(
            "PATCH",
            &format!("/api/v1/knowledge/{id}"),
            &token,
            Some(json!({ "title": "New Title", "category": "adr" })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let updated = response_json(resp).await;
    assert_eq!(updated["title"], "New Title");
    assert_eq!(updated["category"], "adr");
    assert_eq!(updated["content"], "Old content");
}

// ── Delete tests ──────────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn soft_delete_entry(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token,
            Some(json!({
                "category": "custom",
                "title": "To delete",
                "content": "This will be deleted."
            })),
        ))
        .await
        .unwrap();
    let created = response_json(resp).await;
    let id = created["id"].as_str().unwrap().to_string();

    // Delete
    let resp = app
        .clone()
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/knowledge/{id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Should not appear in list
    let resp = app
        .oneshot(json_request("GET", "/api/v1/knowledge", &token, None))
        .await
        .unwrap();
    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert!(list.is_empty());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn delete_nonexistent_returns_404(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "DELETE",
            &format!("/api/v1/knowledge/{}", Uuid::now_v7()),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── Validation tests ──────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_rejects_invalid_category(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token,
            Some(json!({
                "category": "invalid_cat",
                "title": "Title",
                "content": "Content"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_rejects_empty_title(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token,
            Some(json!({
                "category": "convention",
                "title": "   ",
                "content": "Content"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_rejects_empty_content(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token,
            Some(json!({
                "category": "convention",
                "title": "Title",
                "content": ""
            })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn update_rejects_invalid_category(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id, "Acme").await;
    let token = test_jwt(user_id, org_id, "owner");
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token,
            Some(json!({
                "category": "custom",
                "title": "Title",
                "content": "Content"
            })),
        ))
        .await
        .unwrap();
    let created = response_json(resp).await;
    let id = created["id"].as_str().unwrap().to_string();

    let resp = app
        .oneshot(json_request(
            "PATCH",
            &format!("/api/v1/knowledge/{id}"),
            &token,
            Some(json!({ "category": "not_a_category" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Auth / isolation tests ────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_requires_auth(pool: PgPool) {
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/knowledge")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn entry_not_visible_to_other_org(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    let org_a = seed_org(&pool, user_a, "Org A").await;
    let org_b = seed_org(&pool, user_b, "Org B").await;

    let token_a = test_jwt(user_a, org_a, "owner");
    let token_b = test_jwt(user_b, org_b, "owner");
    let app = build_app(test_state(pool));

    // Org A creates an entry
    app.clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/knowledge",
            &token_a,
            Some(json!({
                "category": "adr",
                "title": "Secret ADR",
                "content": "Internal decision."
            })),
        ))
        .await
        .unwrap();

    // Org B's list should be empty
    let resp = app
        .oneshot(json_request("GET", "/api/v1/knowledge", &token_b, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(response_json(resp).await).unwrap();
    assert!(list.is_empty());
}
