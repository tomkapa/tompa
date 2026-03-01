//! Integration tests for the tasks domain: CRUD, state machine, and DAG
//! dependency cycle detection.
//!
//! Requires a live Postgres instance. `sqlx::test` creates an isolated database
//! per test and runs all migrations automatically.
//!
//! Run with:
//!   DATABASE_URL=postgres://... cargo test --test tasks

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
    sse::broadcaster::SseBroadcaster,
    state::AppState,
};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

// ── Test helpers ──────────────────────────────────────────────────────────────

const TEST_SECRET: &str = "test-jwt-secret-tasks";

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

async fn seed_story(pool: &PgPool, org_id: Uuid, project_id: Uuid, user_id: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO stories (id, org_id, project_id, title, story_type, owner_id, rank)
         VALUES ($1, $2, $3, $4, 'feature', $5, 'a0')",
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

// ── Task CRUD tests ───────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_task_success(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/tasks",
            &token,
            Some(json!({
                "story_id": story_id,
                "name": "Implement login",
                "description": "Build the login page",
                "task_type": "code",
                "position": 1
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = json_body(resp).await;
    assert_eq!(body["name"], "Implement login");
    assert_eq!(body["state"], "pending");
    assert_eq!(body["task_type"], "code");
    assert_eq!(body["position"], 1);
    assert!(body["dependencies"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_task_empty_name_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/tasks",
            &token,
            Some(json!({
                "story_id": story_id,
                "name": "  ",
                "description": "",
                "task_type": "code",
                "position": 1
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_task_invalid_type_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/tasks",
            &token,
            Some(json!({
                "story_id": story_id,
                "name": "Task",
                "description": "",
                "task_type": "magic",
                "position": 1
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_tasks_ordered_by_position(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    for (name, pos) in [("C", 3), ("A", 1), ("B", 2)] {
        app.clone()
            .oneshot(req(
                "POST",
                "/api/v1/tasks",
                &token,
                Some(json!({
                    "story_id": story_id,
                    "name": name,
                    "description": "",
                    "task_type": "code",
                    "position": pos
                })),
            ))
            .await
            .unwrap();
    }

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/tasks?story_id={story_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    assert_eq!(list.len(), 3);
    let names: Vec<&str> = list.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["A", "B", "C"]);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn get_task_detail(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/tasks",
            &token,
            Some(json!({
                "story_id": story_id,
                "name": "My Task",
                "description": "Details here",
                "task_type": "design",
                "position": 1
            })),
        ))
        .await
        .unwrap();
    let task_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["description"], "Details here");
    assert_eq!(body["task_type"], "design");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn soft_delete_hides_task(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/tasks",
            &token,
            Some(json!({
                "story_id": story_id,
                "name": "Doomed",
                "description": "",
                "task_type": "test",
                "position": 1
            })),
        ))
        .await
        .unwrap();
    let task_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── State machine tests ───────────────────────────────────────────────────────

async fn create_task_via_api(app: axum::Router, story_id: Uuid, token: &str, pos: i32) -> String {
    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/tasks",
            token,
            Some(json!({
                "story_id": story_id,
                "name": "Task",
                "description": "",
                "task_type": "code",
                "position": pos
            })),
        ))
        .await
        .unwrap();
    json_body(resp).await["id"].as_str().unwrap().to_string()
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn valid_state_transitions(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let task_id = create_task_via_api(app.clone(), story_id, &token, 1).await;

    // pending → qa
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            Some(json!({ "state": "qa" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await["state"], "qa");

    // qa → running
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            Some(json!({ "state": "running" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await["state"], "running");

    // running → paused
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            Some(json!({ "state": "paused" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await["state"], "paused");

    // paused → running
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            Some(json!({ "state": "running" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await["state"], "running");

    // running → blocked
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            Some(json!({ "state": "blocked" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await["state"], "blocked");

    // blocked → pending (any → pending reset)
    let resp = app
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            Some(json!({ "state": "pending" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(json_body(resp).await["state"], "pending");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn invalid_state_transition_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let task_id = create_task_via_api(app.clone(), story_id, &token, 1).await;

    // pending → running is not a valid direct jump
    let resp = app
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            Some(json!({ "state": "running" })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn mark_done_from_running_succeeds(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let task_id = create_task_via_api(app.clone(), story_id, &token, 1).await;

    // Advance to running state
    for state in ["qa", "running"] {
        app.clone()
            .oneshot(req(
                "PATCH",
                &format!("/api/v1/tasks/{task_id}"),
                &token,
                Some(json!({ "state": state })),
            ))
            .await
            .unwrap();
    }

    // Set some ai_status_text that should be cleared on done
    app.clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            Some(json!({ "ai_status_text": "Working on it..." })),
        ))
        .await
        .unwrap();

    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/tasks/{task_id}/done"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["state"], "done");
    assert!(body["ai_status_text"].is_null());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn mark_done_not_running_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let task_id = create_task_via_api(app.clone(), story_id, &token, 1).await;

    // Task is in 'pending' state, not 'running'
    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/tasks/{task_id}/done"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn patch_cannot_transition_running_to_done(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let task_id = create_task_via_api(app.clone(), story_id, &token, 1).await;

    for state in ["qa", "running"] {
        app.clone()
            .oneshot(req(
                "PATCH",
                &format!("/api/v1/tasks/{task_id}"),
                &token,
                Some(json!({ "state": state })),
            ))
            .await
            .unwrap();
    }

    // PATCH running → done must be rejected (use /done endpoint)
    let resp = app
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/tasks/{task_id}"),
            &token,
            Some(json!({ "state": "done" })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Dependency DAG tests ──────────────────────────────────────────────────────

async fn make_task(app: axum::Router, story_id: Uuid, token: &str, pos: i32) -> Uuid {
    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/tasks",
            token,
            Some(json!({
                "story_id": story_id,
                "name": format!("Task {pos}"),
                "description": "",
                "task_type": "code",
                "position": pos
            })),
        ))
        .await
        .unwrap();
    json_body(resp).await["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn create_dependency_success(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let a = make_task(app.clone(), story_id, &token, 1).await;
    let b = make_task(app.clone(), story_id, &token, 2).await;

    // B depends on A
    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/task-dependencies",
            &token,
            Some(json!({ "task_id": b, "depends_on_task_id": a })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let dep = json_body(resp).await;
    assert_eq!(dep["task_id"], b.to_string());
    assert_eq!(dep["depends_on_task_id"], a.to_string());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn direct_cycle_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let a = make_task(app.clone(), story_id, &token, 1).await;
    let b = make_task(app.clone(), story_id, &token, 2).await;

    // B depends on A
    app.clone()
        .oneshot(req(
            "POST",
            "/api/v1/task-dependencies",
            &token,
            Some(json!({ "task_id": b, "depends_on_task_id": a })),
        ))
        .await
        .unwrap();

    // A depends on B — would create a cycle
    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/task-dependencies",
            &token,
            Some(json!({ "task_id": a, "depends_on_task_id": b })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = json_body(resp).await;
    assert!(body["error"].as_str().unwrap().contains("yclic"));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn transitive_cycle_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let a = make_task(app.clone(), story_id, &token, 1).await;
    let b = make_task(app.clone(), story_id, &token, 2).await;
    let c = make_task(app.clone(), story_id, &token, 3).await;

    // B depends on A, C depends on B
    for (task, dep) in [(b, a), (c, b)] {
        app.clone()
            .oneshot(req(
                "POST",
                "/api/v1/task-dependencies",
                &token,
                Some(json!({ "task_id": task, "depends_on_task_id": dep })),
            ))
            .await
            .unwrap();
    }

    // A depends on C — would close A → B → C → A cycle
    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/task-dependencies",
            &token,
            Some(json!({ "task_id": a, "depends_on_task_id": c })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn self_loop_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let a = make_task(app.clone(), story_id, &token, 1).await;

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/task-dependencies",
            &token,
            Some(json!({ "task_id": a, "depends_on_task_id": a })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_dependencies_for_story(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let a = make_task(app.clone(), story_id, &token, 1).await;
    let b = make_task(app.clone(), story_id, &token, 2).await;
    let c = make_task(app.clone(), story_id, &token, 3).await;

    // B → A, C → B
    for (task, dep) in [(b, a), (c, b)] {
        app.clone()
            .oneshot(req(
                "POST",
                "/api/v1/task-dependencies",
                &token,
                Some(json!({ "task_id": task, "depends_on_task_id": dep })),
            ))
            .await
            .unwrap();
    }

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/task-dependencies?story_id={story_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let deps: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    assert_eq!(deps.len(), 2);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn delete_dependency_removes_edge(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let a = make_task(app.clone(), story_id, &token, 1).await;
    let b = make_task(app.clone(), story_id, &token, 2).await;

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/task-dependencies",
            &token,
            Some(json!({ "task_id": b, "depends_on_task_id": a })),
        ))
        .await
        .unwrap();
    let dep_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/task-dependencies/{dep_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // After deletion, list should be empty
    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/task-dependencies?story_id={story_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();
    let deps: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    assert!(deps.is_empty());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn task_not_visible_to_other_org(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    let org_a = seed_org(&pool, user_a).await;
    let org_b = seed_org(&pool, user_b).await;
    let project_a = seed_project(&pool, org_a).await;
    let story_a = seed_story(&pool, org_a, project_a, user_a).await;

    let token_a = test_jwt(user_a, org_a);
    let token_b = test_jwt(user_b, org_b);
    let app = build_app(test_state(pool));

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/tasks",
            &token_a,
            Some(json!({
                "story_id": story_a,
                "name": "Secret Task",
                "description": "",
                "task_type": "code",
                "position": 1
            })),
        ))
        .await
        .unwrap();
    let task_id = json_body(resp).await["id"].as_str().unwrap().to_string();

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/tasks/{task_id}"),
            &token_b,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
