//! Integration tests for the stories CRUD + rank endpoints.
//!
//! Requires a live Postgres instance. `sqlx::test` creates an isolated database
//! per test and runs all migrations automatically.
//!
//! Run with:
//!   DATABASE_URL=postgres://... cargo test --test stories

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

// ── Helpers ───────────────────────────────────────────────────────────────────

const TEST_SECRET: &str = "test-jwt-secret-stories";

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
    .bind(format!("user-{id}@test.example"))
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
        .bind("Test Org")
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

fn req(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("cookie", format!("session={token}"))
        .header("content-type", "application/json");
    match body {
        Some(b) => builder.body(Body::from(b.to_string())).unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn json_body(resp: Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_story_success(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "My first story",
                "description": "As a user...",
                "story_type": "feature",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = json_body(resp).await;
    assert_eq!(body["title"], "My first story");
    assert_eq!(body["status"], "todo");
    assert_eq!(body["story_type"], "feature");
    assert!(!body["rank"].as_str().unwrap().is_empty());
    assert!(body["tasks"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_story_empty_title_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "   ",
                "description": "",
                "story_type": "feature",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_story_invalid_type_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "Story",
                "description": "",
                "story_type": "epic",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_stories_ordered_by_rank(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    // Create three stories (each appended at the end)
    for title in ["Alpha", "Beta", "Gamma"] {
        app.clone()
            .oneshot(req(
                "POST",
                "/api/v1/stories",
                &token,
                Some(json!({
                    "project_id": project_id,
                    "title": title,
                    "description": "",
                    "story_type": "feature",
                    "owner_id": user_id
                })),
            ))
            .await
            .unwrap();
    }

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/stories?project_id={project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    assert_eq!(list.len(), 3);
    let titles: Vec<&str> = list.iter().map(|s| s["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["Alpha", "Beta", "Gamma"]);

    // Verify ranks are in ascending order
    let ranks: Vec<&str> = list.iter().map(|s| s["rank"].as_str().unwrap()).collect();
    for i in 0..ranks.len() - 1 {
        assert!(ranks[i] < ranks[i + 1], "rank order violated: {ranks:?}");
    }
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn get_story_detail_includes_tasks(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "Story with tasks",
                "description": "",
                "story_type": "feature",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();
    let story = json_body(resp).await;
    let story_id: Uuid = story["id"].as_str().unwrap().parse().unwrap();

    // Seed a task directly in the DB
    let task_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO tasks (id, org_id, story_id, name, task_type, state, position)
         VALUES ($1, $2, $3, $4, 'code', 'pending', 1)",
    )
    .bind(task_id)
    .bind(org_id)
    .bind(story_id)
    .bind("Implement login")
    .execute(&pool)
    .await
    .unwrap();

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/stories/{story_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let detail = json_body(resp).await;
    let tasks = detail["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["name"], "Implement login");
    assert_eq!(tasks[0]["task_type"], "code");
    assert_eq!(tasks[0]["state"], "pending");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn start_story_feature_sets_grooming(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "Feature story",
                "description": "",
                "story_type": "feature",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();
    let story = json_body(resp).await;
    let story_id = story["id"].as_str().unwrap();

    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/stories/{story_id}/start"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let started = json_body(resp).await;
    assert_eq!(started["status"], "in_progress");
    assert_eq!(started["pipeline_stage"], "grooming");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn start_story_bug_sets_implementation(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "Bug fix",
                "description": "",
                "story_type": "bug",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();
    let story_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/stories/{story_id}/start"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let started = json_body(resp).await;
    assert_eq!(started["status"], "in_progress");
    assert_eq!(started["pipeline_stage"], "implementation");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn start_already_in_progress_is_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "Story",
                "description": "",
                "story_type": "feature",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();
    let story_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    // First start — should succeed
    app.clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/stories/{story_id}/start"),
            &token,
            None,
        ))
        .await
        .unwrap();

    // Second start — should fail (already in_progress)
    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/stories/{story_id}/start"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn status_transition_in_progress_to_done(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "Story",
                "description": "",
                "story_type": "feature",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();
    let story_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    app.clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/stories/{story_id}/start"),
            &token,
            None,
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/stories/{story_id}"),
            &token,
            Some(json!({ "status": "done" })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await["status"], "done");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn invalid_status_transition_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "Story",
                "description": "",
                "story_type": "feature",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();
    let story_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    // Attempt todo → done without going through in_progress (invalid)
    let resp = app
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/stories/{story_id}"),
            &token,
            Some(json!({ "status": "done" })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn rank_reorder_inserts_between(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    // Create three stories: A, B, C
    let mut ids = Vec::new();
    for title in ["A", "B", "C"] {
        let resp = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/stories",
                &token,
                Some(json!({
                    "project_id": project_id,
                    "title": title,
                    "description": "",
                    "story_type": "feature",
                    "owner_id": user_id
                })),
            ))
            .await
            .unwrap();
        let id = json_body(resp).await["id"].as_str().unwrap().to_string();
        ids.push(id);
    }
    let (id_a, id_b, id_c) = (&ids[0], &ids[1], &ids[2]);

    // Move C between A and B: after=A, before=B
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/stories/{id_c}/rank"),
            &token,
            Some(json!({ "after_id": id_a, "before_id": id_b })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // List and verify order is now A, C, B
    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/stories?project_id={project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    let list: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    let titles: Vec<&str> = list.iter().map(|s| s["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["A", "C", "B"], "expected A,C,B but got {titles:?}");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn rank_reorder_to_first_position(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let mut ids = Vec::new();
    for title in ["A", "B", "C"] {
        let resp = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/stories",
                &token,
                Some(json!({
                    "project_id": project_id,
                    "title": title,
                    "description": "",
                    "story_type": "feature",
                    "owner_id": user_id
                })),
            ))
            .await
            .unwrap();
        ids.push(json_body(resp).await["id"].as_str().unwrap().to_string());
    }
    let (id_a, _id_b, id_c) = (&ids[0], &ids[1], &ids[2]);

    // Move C to first position (before A, no after)
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/stories/{id_c}/rank"),
            &token,
            Some(json!({ "before_id": id_a })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/stories?project_id={project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    let list: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    let titles: Vec<&str> = list.iter().map(|s| s["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["C", "A", "B"], "expected C,A,B but got {titles:?}");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn soft_delete_hides_story(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "Doomed",
                "description": "",
                "story_type": "bug",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();
    let story_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/stories/{story_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Must not appear in list
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/stories?project_id={project_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    let list: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    assert!(list.is_empty());

    // Must 404 on direct GET
    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/stories/{story_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn story_not_visible_to_other_org(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    let org_a = seed_org(&pool, user_a).await;
    let org_b = seed_org(&pool, user_b).await;
    let project_a = seed_project(&pool, org_a).await;

    let token_a = test_jwt(user_a, org_a);
    let token_b = test_jwt(user_b, org_b);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token_a,
            Some(json!({
                "project_id": project_a,
                "title": "Secret",
                "description": "",
                "story_type": "feature",
                "owner_id": user_a
            })),
        ))
        .await
        .unwrap();
    let story_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    // org_b cannot see the story
    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/stories/{story_id}"),
            &token_b,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn update_pipeline_stage(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/stories",
            &token,
            Some(json!({
                "project_id": project_id,
                "title": "Story",
                "description": "",
                "story_type": "feature",
                "owner_id": user_id
            })),
        ))
        .await
        .unwrap();
    let story_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    // Start to enter in_progress
    app.clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/stories/{story_id}/start"),
            &token,
            None,
        ))
        .await
        .unwrap();

    // Update pipeline_stage to planning
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/stories/{story_id}"),
            &token,
            Some(json!({ "pipeline_stage": "planning" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await["pipeline_stage"], "planning");

    // Invalid pipeline_stage
    let resp = app
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/stories/{story_id}"),
            &token,
            Some(json!({ "pipeline_stage": "invalid_stage" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
