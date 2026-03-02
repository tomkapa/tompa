//! Integration tests for the Q&A rounds domain: listing, answer submission,
//! checkpoint rollback, and course correction.
//!
//! Requires a live Postgres instance. `sqlx::test` creates an isolated database
//! per test and runs all migrations automatically.
//!
//! Run with:
//!   DATABASE_URL=postgres://... cargo test --test qa_rounds

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

const TEST_SECRET: &str = "test-jwt-secret-qa";

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
            dev_mode: false,
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

async fn seed_task(pool: &PgPool, org_id: Uuid, story_id: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO tasks (id, org_id, story_id, name, task_type, position)
         VALUES ($1, $2, $3, 'Test Task', 'code', 1)",
    )
    .bind(id)
    .bind(org_id)
    .bind(story_id)
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Seed a QA round directly (rounds are created by the container agent, not via API).
async fn seed_round(
    pool: &PgPool,
    org_id: Uuid,
    story_id: Uuid,
    task_id: Option<Uuid>,
    stage: &str,
    round_number: i32,
    content: Value,
) -> Uuid {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO qa_rounds (id, org_id, story_id, task_id, stage, round_number, content)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(id)
    .bind(org_id)
    .bind(story_id)
    .bind(task_id)
    .bind(stage)
    .bind(round_number)
    .bind(content)
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

fn make_question(id: Uuid) -> Value {
    json!({
        "id": id,
        "text": "Which approach to use?",
        "domain": "backend",
        "options": ["Option A", "Option B"],
        "selected_answer_index": null,
        "selected_answer_text": null,
        "answered_by": null,
        "answered_at": null
    })
}

// ── List rounds tests ─────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_rounds_by_story_id(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q1 = Uuid::now_v7();
    let q2 = Uuid::now_v7();
    seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q1)], "course_correction": null }),
    )
    .await;
    seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        2,
        json!({ "questions": [make_question(q2)], "course_correction": null }),
    )
    .await;

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/qa-rounds?story_id={story_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["round_number"], 1);
    assert_eq!(list[1]["round_number"], 2);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_rounds_by_task_id(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let task_id = seed_task(&pool, org_id, story_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q = Uuid::now_v7();
    seed_round(
        &pool,
        org_id,
        story_id,
        Some(task_id),
        "task_qa",
        1,
        json!({ "questions": [make_question(q)], "course_correction": null }),
    )
    .await;
    // Also seed a story-level round (should not be returned)
    let q2 = Uuid::now_v7();
    seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q2)], "course_correction": null }),
    )
    .await;

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/qa-rounds?task_id={task_id}"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["stage"], "task_qa");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_rounds_filtered_by_stage(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q1 = Uuid::now_v7();
    let q2 = Uuid::now_v7();
    seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q1)], "course_correction": null }),
    )
    .await;
    seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "planning",
        1,
        json!({ "questions": [make_question(q2)], "course_correction": null }),
    )
    .await;

    let resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/qa-rounds?story_id={story_id}&stage=grooming"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let list: Vec<Value> = serde_json::from_value(json_body(resp).await).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["stage"], "grooming");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn list_rounds_missing_filter_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(req("GET", "/api/v1/qa-rounds", &token, None))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Answer submission tests ───────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn submit_answer_success(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q_id = Uuid::now_v7();
    let round_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({
            "questions": [make_question(q_id)],
            "course_correction": null
        }),
    )
    .await;

    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/answer"),
            &token,
            Some(json!({
                "question_id": q_id,
                "selected_answer_index": 0,
                "answer_text": "Option A"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    let questions = body["content"]["questions"].as_array().unwrap();
    assert_eq!(questions[0]["selected_answer_index"], 0);
    assert_eq!(questions[0]["selected_answer_text"], "Option A");
    assert!(!questions[0]["answered_by"].is_null());
    assert!(!questions[0]["answered_at"].is_null());
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn submit_answer_other_option(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q_id = Uuid::now_v7();
    let round_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q_id)], "course_correction": null }),
    )
    .await;

    // Submit with null index (free-form "Other" answer)
    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/answer"),
            &token,
            Some(json!({
                "question_id": q_id,
                "selected_answer_index": null,
                "answer_text": "My custom answer"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    let questions = body["content"]["questions"].as_array().unwrap();
    assert!(questions[0]["selected_answer_index"].is_null());
    assert_eq!(questions[0]["selected_answer_text"], "My custom answer");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn double_answer_returns_error(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q_id = Uuid::now_v7();
    let round_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q_id)], "course_correction": null }),
    )
    .await;

    // First answer
    app.clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/answer"),
            &token,
            Some(json!({
                "question_id": q_id,
                "selected_answer_index": 0,
                "answer_text": "Option A"
            })),
        ))
        .await
        .unwrap();

    // Second answer on the same question
    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/answer"),
            &token,
            Some(json!({
                "question_id": q_id,
                "selected_answer_index": 1,
                "answer_text": "Option B"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = json_body(resp).await;
    assert!(body["error"].as_str().unwrap().contains("already"));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn answer_superseded_round_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q_id = Uuid::now_v7();
    let round_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO qa_rounds (id, org_id, story_id, task_id, stage, round_number, status, content)
         VALUES ($1, $2, $3, NULL, 'grooming', 1, 'superseded', $4)",
    )
    .bind(round_id)
    .bind(org_id)
    .bind(story_id)
    .bind(json!({ "questions": [make_question(q_id)], "course_correction": null }))
    .execute(&pool)
    .await
    .unwrap();

    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/answer"),
            &token,
            Some(json!({
                "question_id": q_id,
                "selected_answer_index": 0,
                "answer_text": "Option A"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn answer_unknown_question_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q_id = Uuid::now_v7();
    let round_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q_id)], "course_correction": null }),
    )
    .await;

    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/answer"),
            &token,
            Some(json!({
                "question_id": Uuid::now_v7(),  // unknown id
                "selected_answer_index": 0,
                "answer_text": "Option A"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── Rollback tests ────────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn rollback_supersedes_subsequent_rounds(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q1 = Uuid::now_v7();
    let q2 = Uuid::now_v7();
    let q3 = Uuid::now_v7();

    let round1_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q1)], "course_correction": null }),
    )
    .await;
    let round2_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        2,
        json!({ "questions": [make_question(q2)], "course_correction": null }),
    )
    .await;
    let round3_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        3,
        json!({ "questions": [make_question(q3)], "course_correction": null }),
    )
    .await;

    // Rollback to round 1 — rounds 2 and 3 should be superseded
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round1_id}/rollback"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    assert_eq!(body["id"], round1_id.to_string());
    assert_eq!(body["status"], "active");

    // Verify rounds 2 and 3 are now superseded
    let list_resp = app
        .oneshot(req(
            "GET",
            &format!("/api/v1/qa-rounds?story_id={story_id}&stage=grooming"),
            &token,
            None,
        ))
        .await
        .unwrap();
    let list: Vec<Value> = serde_json::from_value(json_body(list_resp).await).unwrap();

    let round2_status = list
        .iter()
        .find(|r| r["id"] == round2_id.to_string())
        .unwrap()["status"]
        .as_str()
        .unwrap()
        .to_string();
    let round3_status = list
        .iter()
        .find(|r| r["id"] == round3_id.to_string())
        .unwrap()["status"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(round2_status, "superseded");
    assert_eq!(round3_status, "superseded");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn rollback_clears_answers_for_re_answering(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let q_id = Uuid::now_v7();
    let round_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q_id)], "course_correction": null }),
    )
    .await;

    // Answer the question
    app.clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/answer"),
            &token,
            Some(json!({
                "question_id": q_id,
                "selected_answer_index": 0,
                "answer_text": "Option A"
            })),
        ))
        .await
        .unwrap();

    // Rollback — answers should be cleared
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/rollback"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Re-answer the same question — should succeed now
    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/answer"),
            &token,
            Some(json!({
                "question_id": q_id,
                "selected_answer_index": 1,
                "answer_text": "Option B"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    let questions = body["content"]["questions"].as_array().unwrap();
    assert_eq!(questions[0]["selected_answer_text"], "Option B");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn rollback_superseded_round_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    let round_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO qa_rounds (id, org_id, story_id, task_id, stage, round_number, status, content)
         VALUES ($1, $2, $3, NULL, 'grooming', 1, 'superseded', $4)",
    )
    .bind(round_id)
    .bind(org_id)
    .bind(story_id)
    .bind(json!({ "questions": [], "course_correction": null }))
    .execute(&pool)
    .await
    .unwrap();

    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/rollback"),
            &token,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Course correction tests ───────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn course_correct_creates_new_round(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    // Seed an existing round so round_number starts at 1
    let q = Uuid::now_v7();
    seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q)], "course_correction": null }),
    )
    .await;

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/qa-rounds/course-correct",
            &token,
            Some(json!({
                "story_id": story_id,
                "task_id": null,
                "stage": "grooming",
                "text": "Please focus more on security implications"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = json_body(resp).await;
    assert_eq!(body["story_id"], story_id.to_string());
    assert_eq!(body["stage"], "grooming");
    assert_eq!(body["round_number"], 2);
    assert_eq!(body["status"], "active");
    assert!(body["content"]["questions"].as_array().unwrap().is_empty());
    assert_eq!(
        body["content"]["course_correction"],
        "Please focus more on security implications"
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn course_correct_first_round_gets_number_one(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/qa-rounds/course-correct",
            &token,
            Some(json!({
                "story_id": story_id,
                "task_id": null,
                "stage": "planning",
                "text": "Consider a microservices approach"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = json_body(resp).await;
    assert_eq!(body["round_number"], 1);
    assert_eq!(body["stage"], "planning");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn course_correct_invalid_stage_rejected(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool));

    let resp = app
        .oneshot(req(
            "POST",
            "/api/v1/qa-rounds/course-correct",
            &token,
            Some(json!({
                "story_id": story_id,
                "task_id": null,
                "stage": "invalid_stage",
                "text": "Some correction"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── RLS / multi-org isolation ─────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn round_not_visible_to_other_org(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    let org_a = seed_org(&pool, user_a).await;
    let org_b = seed_org(&pool, user_b).await;
    let project_a = seed_project(&pool, org_a).await;
    let story_a = seed_story(&pool, org_a, project_a, user_a).await;

    let token_b = test_jwt(user_b, org_b);
    let app = build_app(test_state(pool.clone()));

    let q = Uuid::now_v7();
    let round_id = seed_round(
        &pool,
        org_a,
        story_a,
        None,
        "grooming",
        1,
        json!({ "questions": [make_question(q)], "course_correction": null }),
    )
    .await;

    // Org B tries to answer a round belonging to Org A
    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round_id}/answer"),
            &token_b,
            Some(json!({
                "question_id": q,
                "selected_answer_index": 0,
                "answer_text": "hacked"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── Full flow test ────────────────────────────────────────────────────────────

#[sqlx::test(migrator = "MIGRATOR")]
async fn full_qa_flow_create_answer_rollback_re_answer(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let org_id = seed_org(&pool, user_id).await;
    let project_id = seed_project(&pool, org_id).await;
    let story_id = seed_story(&pool, org_id, project_id, user_id).await;
    let token = test_jwt(user_id, org_id);
    let app = build_app(test_state(pool.clone()));

    // 1. Create round 1 with two questions
    let q1 = Uuid::now_v7();
    let q2 = Uuid::now_v7();
    let round1_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        1,
        json!({
            "questions": [make_question(q1), make_question(q2)],
            "course_correction": null
        }),
    )
    .await;

    // 2. Answer Q1 in round 1
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round1_id}/answer"),
            &token,
            Some(json!({
                "question_id": q1,
                "selected_answer_index": 0,
                "answer_text": "Option A"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Create round 2 (follow-up)
    let q3 = Uuid::now_v7();
    let round2_id = seed_round(
        &pool,
        org_id,
        story_id,
        None,
        "grooming",
        2,
        json!({ "questions": [make_question(q3)], "course_correction": null }),
    )
    .await;

    // 4. Rollback to round 1 — round 2 should be superseded, answers in round 1 cleared
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round1_id}/rollback"),
            &token,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    // Answers cleared after rollback
    let questions = body["content"]["questions"].as_array().unwrap();
    assert!(questions[0]["selected_answer_text"].is_null());

    // 5. Verify round 2 is superseded
    let list_resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/qa-rounds?story_id={story_id}&stage=grooming"),
            &token,
            None,
        ))
        .await
        .unwrap();
    let list: Vec<Value> = serde_json::from_value(json_body(list_resp).await).unwrap();
    let r2 = list
        .iter()
        .find(|r| r["id"] == round2_id.to_string())
        .unwrap();
    assert_eq!(r2["status"], "superseded");

    // 6. Re-answer Q1 in round 1 with a different answer
    let resp = app
        .oneshot(req(
            "POST",
            &format!("/api/v1/qa-rounds/{round1_id}/answer"),
            &token,
            Some(json!({
                "question_id": q1,
                "selected_answer_index": 1,
                "answer_text": "Option B"
            })),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    let questions = body["content"]["questions"].as_array().unwrap();
    assert_eq!(questions[0]["selected_answer_text"], "Option B");
}
