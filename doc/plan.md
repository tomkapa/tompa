# Implementation Tasks — AI-Integrated Development Pipeline

> **Rule:** Each task is self-contained. A developer with no project context can pick up any task and deliver it, given only the task description and listed dependencies.
> **Codebase:** All tasks target the same monorepo. No branching — coordinate via directory boundaries.
> **Completion:** All tasks done = product done.

---

## Dependency Graph (Visual)

```
T01 ─────────────────────────────────────────────────────────────────────────┐
T02 ──────────────────────────────────┐                                      │
T03 ─────────────┐                    │                                      │
                 ├──► T07 ──► T08 ──►├──► T11 ──► T12                       │
T04 ─────────────┤                    │                                      │
T05 ─────────────┘                    │                                      │
                                      ├──► T13 ──► T14 ──► T16 ──► T17      │
T06 ──────────────────────────────────┘                                      │
                                                                             │
T09 ──► T10                                                                  │
                                                                             │
T15 (depends on T11, T14)                                                    │
                                                                             │
T18 (depends on T01) ──► T19 ──► T20                                         │
                    └──► T21                                                 │
                    └──► T24                                                 │
                                                                             │
T22 (depends on T01, T07, T08, T11, T13)                                    │
                                                                             │
T23 (depends on T22, T10)                                                    │
```

---

## Task Index

| ID | Name | Layer | Depends On | Parallelizable With |
|----|------|-------|------------|---------------------|
| T01 | Monorepo Scaffold & Shared Crate | Backend | — | T02–T06, T09 |
| T02 | Database Migrations | Backend | — | T01, T03–T06, T09 |
| T03 | Server Boilerplate & Config | Backend | — | T01, T02, T04–T06, T09 |
| T04 | Unified Error Handling | Backend | — | T01–T03, T05–T06, T09 |
| T05 | DB Pool, RLS & Base Repository | Backend | — | T01–T04, T06, T09 |
| T06 | Auth: OAuth2 + JWT Middleware | Backend | — | T01–T05, T09 |
| T07 | Orgs & Projects Domain (CRUD) | Backend | T03, T04, T05 | T08 |
| T08 | Stories Domain (CRUD + Rank) | Backend | T07 | T11, T13 |
| T09 | Frontend Scaffold & Routing | Frontend | — | T01–T06 |
| T10 | Frontend UI Atoms | Frontend | T09 | T01–T08 |
| T11 | Tasks Domain (CRUD + Dependencies) | Backend | T08 | T12 |
| T12 | Q&A Domain (Rounds, Answers, Rollback) | Backend | T11 | T13, T14 |
| T13 | Container Keys Domain (CRUD) | Backend | T07 | T14 |
| T14 | Knowledge Domain (CRUD) | Backend | T07 | T12, T13 |
| T15 | SSE Broadcaster & Endpoint | Backend | T11, T12 | — |
| T16 | WebSocket Handler & Connection Registry | Backend | T13 | T17 |
| T17 | Container Orchestration Service | Backend | T16 | — |
| T18 | Agent Scaffold & Actor Framework | Agent | T01 | T09, T10 |
| T19 | Agent WebSocket Client & Reconnection | Agent | T18 | — |
| T20 | Claude Code Supervisor & Prompts | Agent | T19 | — |
| T21 | Git Manager | Agent | T18 | T19, T20, T24 |
| T22 | OpenAPI + Orval Type Pipeline | Cross | T01, T07, T08, T11, T13 | — |
| T23 | Frontend Feature Modules (Full UI) | Frontend | T22, T10 | — |
| T24 | Agent Setup UI (Embedded SPA) | Agent | T18 | T19–T21 |
| T25 | CI/CD Pipeline & Docker Builds | Infra | — | All |
| T26 | Helm Chart & ArgoCD Config | Infra | T25 | All |

---

## T01 — Monorepo Scaffold & Shared Crate

**Layer:** Backend (Rust)
**Depends on:** Nothing
**Outputs:** Cargo workspace, shared crate with all message types

### What to Build

Initialize the monorepo directory structure and the `shared` crate that both `server` and `agent` consume.

### Directory Structure

```
project-root/
├── backend/
│   ├── Cargo.toml           # Workspace root: members = ["server", "agent", "shared"]
│   ├── server/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs      # Empty main, just compiles
│   ├── agent/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs      # Empty main, just compiles
│   └── shared/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── messages.rs   # WebSocket protocol enums
│           ├── enums.rs      # Domain enums (StoryType, TaskType, etc.)
│           └── types.rs      # Shared data structures
├── frontend/                 # Empty placeholder
├── helm/                     # Empty placeholder
└── .github/workflows/        # Empty placeholder
```

### Shared Crate Contents

**`messages.rs`** — WebSocket protocol envelope. All messages are tagged JSON via serde:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ServerToContainer {
    StartGrooming { story_id: Uuid, context: GroomingContext },
    StartPlanning { story_id: Uuid, context: PlanningContext },
    AnswerReceived { round_id: Uuid, answers: Vec<Answer> },
    StartTask { task_id: Uuid, session_id: String, context: TaskContext },
    ResumeTask { task_id: Uuid, session_id: String, answer: Answer },
    CancelTask { task_id: Uuid },
    Ping,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ContainerToServer {
    QuestionBatch { story_id: Uuid, task_id: Option<Uuid>, round: QaRoundContent },
    TaskDecomposition { story_id: Uuid, proposed_tasks: Vec<ProposedTask> },
    TaskPaused { task_id: Uuid, question: PauseQuestion },
    TaskCompleted { task_id: Uuid, commit_sha: String },
    TaskFailed { task_id: Uuid, error: String },
    StatusUpdate { task_id: Uuid, status_text: String },
    Pong,
}
```

**`enums.rs`** — Shared domain enums used by both server and agent:

```rust
pub enum StoryType { Feature, Bug, Refactor }
pub enum StoryStatus { Todo, InProgress, Done }
pub enum PipelineStage { Grooming, Planning, Decomposition, Implementation, Testing, Review }
pub enum TaskType { Design, Test, Code }
pub enum TaskState { Pending, Qa, Running, Paused, Blocked, Done }
pub enum QaStage { Grooming, Planning, TaskQa, Implementation }
pub enum QaRoundStatus { Active, Superseded }
pub enum ContainerMode { Project, Dev, Standalone }
pub enum OrgRole { Owner, Admin, Member }
pub enum KnowledgeCategory { Convention, Adr, ApiDoc, DesignSystem, Custom }
```

All enums: `#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]` with `#[serde(rename_all = "snake_case")]`.

**`types.rs`** — Shared data structures for message payloads:

```rust
pub struct GroomingContext { pub story_description: String, pub knowledge: Vec<KnowledgeEntry>, pub codebase_context: String }
pub struct PlanningContext { pub story_description: String, pub grooming_decisions: Vec<QaDecision>, pub knowledge: Vec<KnowledgeEntry>, pub codebase_context: String }
pub struct TaskContext { pub task_description: String, pub story_decisions: Vec<QaDecision>, pub sibling_decisions: Vec<QaDecision>, pub knowledge: Vec<KnowledgeEntry> }
pub struct QaRoundContent { pub questions: Vec<Question> }
pub struct Question { pub id: Uuid, pub text: String, pub domain: String, pub options: Vec<String> }
pub struct Answer { pub question_id: Uuid, pub selected_answer_index: Option<i32>, pub selected_answer_text: String, pub answered_by: Uuid, pub answered_at: DateTime<Utc> }
pub struct QaDecision { pub question_text: String, pub answer_text: String, pub domain: String }
pub struct ProposedTask { pub name: String, pub description: String, pub task_type: TaskType, pub position: i32, pub depends_on: Vec<i32> }
pub struct PauseQuestion { pub text: String, pub domain: String, pub options: Vec<String> }
pub struct KnowledgeEntry { pub title: String, pub content: String, pub category: KnowledgeCategory }
```

### Dependencies (Cargo.toml for shared)

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Acceptance Criteria

- `cargo build` succeeds for all three crates
- `cargo test` in shared passes (add round-trip serde tests for every enum variant and every message type)
- Server and agent crates can `use shared::messages::*`

---

## T02 — Database Migrations

**Layer:** Backend (SQL)
**Depends on:** Nothing
**Outputs:** Complete sqlx migration files in `backend/server/migrations/`

### What to Build

All SQL migration files that create the full database schema. Migrations are embedded and run at server startup via sqlx.

### Migration Files (ordered)

**`001_create_extensions.sql`**
```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";  -- for gen_random_uuid fallback
```

**`002_create_organizations.sql`**
- `organizations` table: `id` (uuid PK), `name` (text NOT NULL), `created_at` (timestamptz DEFAULT now()), `updated_at` (timestamptz DEFAULT now()), `deleted_at` (timestamptz NULL)

**`003_create_users.sql`**
- `users` table: `id` (uuid PK), `email` (text UNIQUE NOT NULL), `display_name` (text NOT NULL), `avatar_url` (text NULL), `oauth_provider` (text NOT NULL), `oauth_provider_id` (text NOT NULL), `created_at`, `updated_at`, `deleted_at`
- Unique constraint on `(oauth_provider, oauth_provider_id)`

**`004_create_org_members.sql`**
- `org_members` table: `id` (uuid PK), `org_id` (uuid FK → organizations NOT NULL), `user_id` (uuid FK → users NOT NULL), `role` (text NOT NULL CHECK IN ('owner', 'admin', 'member')), `created_at`
- Unique constraint on `(org_id, user_id)`

**`005_create_projects.sql`**
- `projects` table: `id`, `org_id` (FK → organizations NOT NULL), `name` (text NOT NULL), `description` (text NULL), `github_repo_url` (text NULL), `created_at`, `updated_at`, `deleted_at`

**`006_create_stories.sql`**
- `stories` table: `id`, `org_id` (FK → organizations NOT NULL), `project_id` (FK → projects NOT NULL), `title` (text NOT NULL), `description` (text NOT NULL DEFAULT ''), `story_type` (text NOT NULL CHECK IN ('feature', 'bug', 'refactor')), `status` (text NOT NULL DEFAULT 'todo' CHECK IN ('todo', 'in_progress', 'done')), `owner_id` (uuid FK → users NOT NULL), `rank` (text NOT NULL), `pipeline_stage` (text NULL CHECK IN ('grooming', 'planning', 'decomposition', 'implementation', 'testing', 'review')), `created_at`, `updated_at`, `deleted_at`
- Index on `(project_id, rank)` WHERE `deleted_at IS NULL`

**`007_create_tasks.sql`**
- `tasks` table: `id`, `org_id` (FK → organizations NOT NULL), `story_id` (FK → stories NOT NULL), `name` (text NOT NULL), `description` (text NOT NULL DEFAULT ''), `task_type` (text NOT NULL CHECK IN ('design', 'test', 'code')), `state` (text NOT NULL DEFAULT 'pending' CHECK IN ('pending', 'qa', 'running', 'paused', 'blocked', 'done')), `position` (integer NOT NULL), `assignee_id` (uuid FK → users NULL), `claude_session_id` (text NULL), `ai_status_text` (text NULL), `created_at`, `updated_at`, `deleted_at`
- Index on `(story_id, position)` WHERE `deleted_at IS NULL`

**`008_create_task_dependencies.sql`**
- `task_dependencies` table: `id` (uuid PK), `task_id` (FK → tasks NOT NULL), `depends_on_task_id` (FK → tasks NOT NULL), `created_at`
- Unique constraint on `(task_id, depends_on_task_id)`
- CHECK constraint: `task_id != depends_on_task_id`

**`009_create_qa_rounds.sql`**
- `qa_rounds` table: `id`, `org_id` (FK → organizations NOT NULL), `story_id` (FK → stories NOT NULL), `task_id` (uuid FK → tasks NULL), `stage` (text NOT NULL CHECK IN ('grooming', 'planning', 'task_qa', 'implementation')), `round_number` (integer NOT NULL), `status` (text NOT NULL DEFAULT 'active' CHECK IN ('active', 'superseded')), `content` (jsonb NOT NULL DEFAULT '{}'), `created_at`, `updated_at`
- Index on `(story_id, task_id, stage, round_number)`

**`010_create_container_api_keys.sql`**
- `container_api_keys` table: `id`, `org_id` (FK → organizations NOT NULL), `project_id` (FK → projects NOT NULL), `key_hash` (text NOT NULL), `label` (text NOT NULL), `container_mode` (text NOT NULL CHECK IN ('project', 'dev', 'standalone')), `last_connected_at` (timestamptz NULL), `created_at`, `revoked_at` (timestamptz NULL)

**`011_create_knowledge_entries.sql`**
- `knowledge_entries` table: `id`, `org_id` (FK → organizations NOT NULL), `project_id` (uuid FK → projects NULL), `story_id` (uuid FK → stories NULL), `category` (text NOT NULL CHECK IN ('convention', 'adr', 'api_doc', 'design_system', 'custom')), `title` (text NOT NULL), `content` (text NOT NULL), `created_at`, `updated_at`, `deleted_at`
- Index on `(org_id, project_id)` WHERE `deleted_at IS NULL`

**`012_enable_rls.sql`**
- Enable RLS on all tenant-scoped tables: `organizations`, `projects`, `stories`, `tasks`, `qa_rounds`, `container_api_keys`, `knowledge_entries`, `org_members`
- Create RLS policies using `current_setting('app.org_id', true)`:
  ```sql
  ALTER TABLE stories ENABLE ROW LEVEL SECURITY;
  CREATE POLICY org_isolation ON stories USING (org_id::text = current_setting('app.org_id', true));
  ```
- Repeat for all tenant-scoped tables

### Acceptance Criteria

- All migration files parse as valid SQL
- Running migrations against a fresh Postgres creates all tables with correct constraints
- RLS policies are active on all tenant-scoped tables
- UUIDv7 IDs are generated by the application layer (not DB defaults) — migration just defines `uuid` PK columns

---

## T03 — Server Boilerplate & Config

**Layer:** Backend (Rust — `server` crate)
**Depends on:** Nothing
**Outputs:** Runnable Axum server with config loading, router assembly, migration execution

### What to Build

The server entry point that loads config, connects to Postgres, runs migrations, assembles the Axum router, and starts listening.

### Files to Create

**`backend/server/src/config.rs`**
- Struct `Config` with fields: `database_url: String`, `port: u16` (default 3000), `jwt_secret: String`, `google_client_id: String`, `google_client_secret: String`, `github_client_id: String`, `github_client_secret: String`, `oauth_redirect_base_url: String`
- Load from environment variables using `std::env::var`
- Panic with clear error messages on missing required vars

**`backend/server/src/main.rs`**
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load config
    // 2. Create PgPool
    // 3. Run embedded migrations: sqlx::migrate!("./migrations").run(&pool).await?
    // 4. Build app state (pool, config, registries)
    // 5. Assemble router (just a health check endpoint for now: GET /health → 200)
    // 6. Start server on 0.0.0.0:{port}
}
```

**`backend/server/src/db.rs`**
- Function `create_pool(database_url: &str) -> PgPool` — creates sqlx PgPool with reasonable defaults (max_connections: 10)
- Function `set_org_id(pool: &PgPool, org_id: Uuid)` — executes `SET LOCAL app.org_id = '{org_id}'` for RLS

**App State struct** (in `main.rs` or separate `state.rs`):
```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    // Future: container_registry, sse_broadcaster
}
```

### Dependencies (server Cargo.toml)

```toml
[dependencies]
shared = { path = "../shared" }
tokio = { version = "1", features = ["full"] }
axum = { version = "0.8", features = ["ws"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
```

### Acceptance Criteria

- `cargo run` starts the server, runs migrations, and serves `GET /health` → 200 OK
- Server panics with clear message if DATABASE_URL is missing
- Migrations run idempotently (running twice doesn't error)

---

## T04 — Unified Error Handling

**Layer:** Backend (Rust — `server` crate)
**Depends on:** Nothing
**Outputs:** `errors.rs` with `ApiError` enum that maps domain errors to HTTP responses

### What to Build

**`backend/server/src/errors.rs`**

A top-level `ApiError` enum that wraps all domain errors and implements `axum::response::IntoResponse`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Story(#[from] StoryError),
    #[error(transparent)]
    Task(#[from] TaskError),
    #[error(transparent)]
    Qa(#[from] QaError),
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error(transparent)]
    Knowledge(#[from] KnowledgeError),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not found")]
    NotFound,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into()),
            // Domain errors: map each variant to appropriate HTTP status
            // NotFound variants → 404
            // InvalidTransition / validation errors → 400
            // Everything else → 500
            _ => map_domain_error(&self),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

Also create placeholder domain error enums (one per domain module) that will be filled in by later tasks. Each domain error goes in its own `types.rs`:

- `StoryError` — `NotFound`, `InvalidTransition { from, to }`, `HasActiveTasks`
- `TaskError` — `NotFound`, `InvalidState { from, to }`, `CyclicDependency`, `StoryNotFound`
- `QaError` — `NotFound`, `RoundNotActive`, `AlreadyAnswered`, `InvalidRollback`
- `ProjectError` — `NotFound`, `NameRequired`
- `KnowledgeError` — `NotFound`

### Acceptance Criteria

- All error enums compile with `thiserror`
- `ApiError` correctly maps every variant to an HTTP status code
- Response body is always `{ "error": "message" }`
- `anyhow::Error` maps to 500 with generic "Internal server error" (no leak)

---

## T05 — DB Pool, RLS & Base Repository Pattern

**Layer:** Backend (Rust — `server` crate)
**Depends on:** Nothing
**Outputs:** RLS-aware database access pattern, reusable across all domain repos

### What to Build

Establish the repository pattern: every repo function takes `&PgPool` and `org_id: Uuid` as mandatory parameters. Before each query, set the RLS context.

**`backend/server/src/db.rs`** (extend from T03):

```rust
use sqlx::PgPool;
use uuid::Uuid;

/// Sets the org_id for RLS on the current connection.
/// Must be called within a transaction or before queries that need RLS.
pub async fn set_rls_context(pool: &PgPool, org_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(&format!("SET LOCAL app.org_id = '{}'", org_id))
        .execute(pool)
        .await?;
    Ok(())
}

/// Creates a new UUIDv7
pub fn new_id() -> Uuid {
    uuid::Uuid::now_v7()
}
```

**Example base pattern** (document this in a `CONTRIBUTING.md` or inline comments for other developers):

```rust
// Every repo function follows this pattern:
pub async fn find_by_id(pool: &PgPool, org_id: Uuid, id: Uuid) -> Result<Option<Story>, sqlx::Error> {
    sqlx::query_as!(
        StoryRow,
        r#"SELECT id, org_id, project_id, title, description,
                  story_type, status, owner_id, rank, pipeline_stage,
                  created_at, updated_at, deleted_at
           FROM stories
           WHERE id = $1 AND org_id = $2 AND deleted_at IS NULL"#,
        id,
        org_id
    )
    .fetch_optional(pool)
    .await
}
```

Key rules:
- Every query includes `AND org_id = $N` explicitly (belt)
- RLS is the suspenders (set via middleware before handler runs)
- `deleted_at IS NULL` in every SELECT query
- All IDs generated application-side via `Uuid::now_v7()`

### Acceptance Criteria

- `set_rls_context` function works against real Postgres
- Pattern is documented so other task developers can follow it
- `new_id()` returns valid UUIDv7 values

---

## T06 — Auth: OAuth2 + JWT Middleware

**Layer:** Backend (Rust — `server` crate)
**Depends on:** Nothing (uses `users` and `org_members` tables from T02 migrations, but can define its own row types)
**Outputs:** Complete auth module with OAuth2 flow, JWT middleware, and org_id injection

### What to Build

**`backend/server/src/auth/`** module with 4 files:

**`types.rs`**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: Uuid,          // user_id
    pub org_id: Uuid,
    pub role: String,       // "owner", "admin", "member"
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthProfile {
    pub provider: String,
    pub provider_id: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}
```

**`service.rs`**
- `create_jwt(claims: &AuthClaims, secret: &str) -> String` — sign JWT using HMAC-SHA256
- `validate_jwt(token: &str, secret: &str) -> Result<AuthClaims, AuthError>` — decode and validate
- `exchange_google_code(code: &str, config: &Config) -> Result<OAuthProfile, anyhow::Error>` — call Google OAuth2 token endpoint, then userinfo
- `exchange_github_code(code: &str, config: &Config) -> Result<OAuthProfile, anyhow::Error>` — call GitHub OAuth2 token endpoint, then user API
- `find_or_create_user(pool: &PgPool, profile: &OAuthProfile) -> Result<(User, OrgMember), anyhow::Error>` — upsert user, auto-create personal org if first login

**`middleware.rs`**
- Axum middleware extractor that:
  1. Reads JWT from `session` HTTP-only cookie
  2. Validates JWT signature and expiry
  3. Extracts `AuthContext { user_id, org_id, role }`
  4. Sets RLS context via `SET LOCAL app.org_id`
  5. Injects `AuthContext` as request extension

**`handler.rs`**
- `GET /api/v1/auth/login/:provider` — redirect to Google/GitHub OAuth consent screen
- `GET /api/v1/auth/callback/:provider` — exchange code, create/find user, set JWT cookie, redirect to app
- `POST /api/v1/auth/logout` — clear JWT cookie
- `GET /api/v1/auth/me` — return current user + org info (requires auth)

### JWT Cookie Settings

- Name: `session`
- `HttpOnly: true`, `Secure: true` (production), `SameSite: Lax`
- Expiry: 7 days
- Path: `/`

### Additional Dependencies

```toml
jsonwebtoken = "9"
reqwest = { version = "0.12", features = ["json"] }
```

### Acceptance Criteria

- OAuth2 login flow works for both Google and GitHub
- JWT middleware rejects expired/invalid tokens with 401
- `AuthContext` is available in all downstream handlers
- Unauthenticated requests to protected routes return 401
- `GET /api/v1/auth/me` returns user profile with org info

---

## T07 — Orgs & Projects Domain (CRUD)

**Layer:** Backend (Rust — `server` crate)
**Depends on:** T03 (server boilerplate), T04 (error handling), T05 (DB pattern)
**Outputs:** Full CRUD for organizations and projects

### What to Build

Two domain modules following the handler → service → repo → types pattern.

**`backend/server/src/projects/`** — `handler.rs`, `service.rs`, `repo.rs`, `types.rs`

### Endpoints

```
GET    /api/v1/orgs                        → List user's organizations
POST   /api/v1/orgs                        → Create organization
GET    /api/v1/projects?org_id=X           → List projects in org
POST   /api/v1/projects                    → Create project
GET    /api/v1/projects/:id                → Project detail
PATCH  /api/v1/projects/:id                → Update project (name, description, github_repo_url)
DELETE /api/v1/projects/:id                → Soft delete (set deleted_at)
```

### Request/Response Types

```rust
// Projects
struct CreateProjectRequest { name: String, description: Option<String>, github_repo_url: Option<String> }
struct UpdateProjectRequest { name: Option<String>, description: Option<String>, github_repo_url: Option<String> }
struct ProjectResponse { id: Uuid, org_id: Uuid, name: String, description: Option<String>, github_repo_url: Option<String>, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }

// Orgs
struct CreateOrgRequest { name: String }
struct OrgResponse { id: Uuid, name: String, role: String, created_at: DateTime<Utc> }
```

### Business Rules

- Org list returns only orgs the authenticated user belongs to (join `org_members`)
- Creating an org auto-adds the creator as "owner"
- Project CRUD is scoped to `org_id` from auth context
- Soft delete sets `deleted_at = now()`, all queries filter `deleted_at IS NULL`
- All types derive `ToSchema` for OpenAPI generation via utoipa

### Acceptance Criteria

- All endpoints return correct HTTP status codes
- Soft delete works (deleted projects don't appear in list)
- Org membership is enforced (users can't access other orgs' projects)
- Integration tests against real Postgres

---

## T08 — Stories Domain (CRUD + Rank)

**Layer:** Backend (Rust — `server` crate)
**Depends on:** T07 (projects must exist)
**Outputs:** Full stories CRUD with fractional indexing for priority ordering

### What to Build

**`backend/server/src/stories/`** — handler, service, repo, types

### Endpoints

```
GET    /api/v1/stories?project_id=X        → List stories ordered by rank
POST   /api/v1/stories                     → Create story
GET    /api/v1/stories/:id                 → Story detail (include task list)
PATCH  /api/v1/stories/:id                 → Update story (title, description, status, owner, pipeline_stage)
DELETE /api/v1/stories/:id                 → Soft delete
PATCH  /api/v1/stories/:id/rank            → Update rank (reorder via fractional index)
POST   /api/v1/stories/:id/start           → Move to "In Progress", set pipeline_stage = "grooming" (or "implementation" for bugs)
```

### Request/Response Types

```rust
struct CreateStoryRequest { project_id: Uuid, title: String, description: String, story_type: String, owner_id: Uuid }
struct UpdateStoryRequest { title: Option<String>, description: Option<String>, status: Option<String>, owner_id: Option<Uuid>, pipeline_stage: Option<String> }
struct RankUpdateRequest { before_id: Option<Uuid>, after_id: Option<Uuid> }
// before_id/after_id: the story to insert between. Generate rank between them using fractional indexing.

struct StoryResponse { id: Uuid, org_id: Uuid, project_id: Uuid, title: String, description: String, story_type: String, status: String, owner_id: Uuid, rank: String, pipeline_stage: Option<String>, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, tasks: Vec<TaskSummary> }
struct TaskSummary { id: Uuid, name: String, task_type: String, state: String, position: i32 }
```

### Business Rules — Status State Machine

Valid transitions:
- `todo` → `in_progress` (only via `/start` endpoint)
- `in_progress` → `done`
- `in_progress` → `todo` (cancel/revert)

Invalid transitions return `StoryError::InvalidTransition`.

### Fractional Indexing (Rank)

- Use the `fractional-indexing` algorithm (implement in Rust or port the logic)
- New story: generate rank after the last story in the project
- Reorder: given `before_id` and `after_id`, generate a rank between their ranks
- If only `before_id`: generate rank before that story
- If only `after_id`: generate rank after that story
- Rank is a string that sorts lexicographically

### Pipeline Stage Logic

- When `/start` is called:
  - Feature/refactor → `pipeline_stage = "grooming"`
  - Bug → `pipeline_stage = "implementation"` (skips grooming/planning)
- Pipeline stage transitions are managed by the orchestration layer (T17), not by this CRUD module. This module just validates the value is in the allowed set.

### Acceptance Criteria

- Stories ordered by rank in list endpoint
- Rank reorder produces correct intermediate values
- Status transitions enforced (invalid returns 400)
- Story detail includes task summary list
- Soft delete works
- Integration tests for rank ordering and state transitions

---

## T09 — Frontend Scaffold & Routing

**Layer:** Frontend
**Depends on:** Nothing
**Outputs:** Runnable React app with TanStack Router, TanStack Query, Zustand, and shadcn/ui configured

### What to Build

Initialize the frontend project with all core dependencies configured.

### Setup Commands

```bash
cd frontend/
bun create vite . --template react-ts
bun add @tanstack/react-query @tanstack/react-router zustand
bun add @dnd-kit/core @dnd-kit/sortable @dnd-kit/utilities
bun add fractional-indexing
bun add -d @tanstack/router-vite-plugin orval vitest @playwright/test
```

Install shadcn/ui following their Vite guide.

### Directory Structure

```
frontend/src/
├── main.tsx                       # Entry point
├── router.tsx                     # TanStack Router configuration
├── App.tsx                        # Root: QueryClientProvider + SSE provider
├── api/
│   └── generated/                 # Placeholder for orval output (T22)
├── components/
│   └── ui/                        # shadcn + custom atoms (T10)
├── features/                      # Feature modules (T23)
├── stores/
│   ├── ui-store.ts                # Zustand: modal state, active tab, drafts
│   └── sse-store.ts               # Zustand: SSE connection state
├── hooks/
│   ├── use-sse.ts                 # SSE connection (placeholder)
│   └── use-auth.ts                # Auth context
└── lib/
    ├── fractional-indexing.ts     # Lexorank helpers (wrapper around npm package)
    └── utils.ts
```

### Router Configuration

```typescript
// Routes:
/                                           → Redirect to default project
/login                                      → Login page
/projects/:projectId                        → Stories table
/projects/:projectId/stories/:storyId       → Story modal open (rendered over table)
/projects/:projectId/stories/:storyId/tasks/:taskId → Task detail in modal
```

### Vite Proxy Config

```typescript
// vite.config.ts
export default defineConfig({
  server: {
    proxy: {
      '/api': 'http://localhost:3000',
    }
  }
})
```

### Zustand Stores

**`ui-store.ts`**
```typescript
interface UIStore {
  activeStoryTab: 'qa' | 'decisions';
  courseCorrectionDraft: Record<string, string>; // keyed by story/task id
  setActiveStoryTab: (tab: 'qa' | 'decisions') => void;
  setDraft: (id: string, text: string) => void;
  clearDraft: (id: string) => void;
  clearAllDrafts: () => void;
}
```

**`sse-store.ts`**
```typescript
interface SSEStore {
  connected: boolean;
  setConnected: (v: boolean) => void;
}
```

### Acceptance Criteria

- `bun dev` starts the app on localhost:5173
- All routes resolve (show placeholder pages)
- TanStack Query DevTools visible in development
- Vite proxy forwards `/api` to `localhost:3000`
- Zustand stores work (test with React DevTools)

---

## T10 — Frontend UI Atoms

**Layer:** Frontend
**Depends on:** T09 (frontend scaffold)
**Outputs:** All 12 atom components from the UI/UX atomic design spec

### What to Build

Every atom component from the UI/UX spec (Steps 1–12). These are pure presentational components with no API calls.

### Components to Build

**`components/ui/status-badge.tsx`** (Step 1)
- Props: `type: 'story' | 'task'`, `value: string`
- Story values: `todo`, `in_progress`, `done`
- Task values: `done`, `running`, `needs_input`, `blocked`
- Each state visually distinct (different bg color + text color)
- Compact, never wraps

**`components/ui/attention-dot.tsx`** (Step 2)
- Pulsing orange dot, CSS animation (keyframes scale + opacity)
- Must not shift layout when appearing/disappearing (`position: absolute` or reserve space)

**`components/ui/domain-tag.tsx`** (Step 3)
- Props: `domain: string` (e.g., "Security", "Backend", "UX")
- Lighter visual weight than status badges
- Compact pill shape

**`components/ui/story-type-tag.tsx`** (Step 4)
- Props: `type: 'feature' | 'bug' | 'refactor'`
- Only renders for bug and refactor (feature = no tag)
- "BUG" tag should be prominent (red-ish)

**`components/ui/task-type-icon.tsx`** (Step 5)
- Props: `type: 'design' | 'test' | 'code'`
- Icons: 🎨 (design), ✅ (test), ⚡ (code)
- Fixed size

**`components/ui/superseded-badge.tsx`** (Step 6)
- Small badge text "Superseded"
- Muted styling (grey)

**`components/ui/rollback-badge.tsx`** (Step 7)
- Small badge text "Rollback point"
- Visually distinct from superseded (different color — e.g., amber/yellow)

**`components/ui/breadcrumb.tsx`** (Step 8)
- Props: `segments: Array<{ label: string, onClick?: () => void }>`
- All segments except last are clickable
- Truncates long names with ellipsis
- Separator: `>` or `/`

**`components/ui/tab-switcher.tsx`** (Step 9)
- Props: `tabs: Array<{ id: string, label: string }>`, `activeId: string`, `onChange: (id) => void`
- One active tab visually indicated
- Horizontal layout

**`components/ui/mark-done-button.tsx`** (Step 10)
- Props: `onClick: () => void`, `loading?: boolean`
- Prominent green/primary button
- Disabled state while loading

**`components/ui/new-question-indicator.tsx`** (Step 11)
- Props: `onClick: () => void`, `visible: boolean`
- Floating pill: "New question ↓"
- Anchored to bottom of scroll container
- Hidden when `visible = false`

**`components/ui/course-correction-input.tsx`** (Step 12)
- Props: `value: string`, `onChange: (v: string) => void`, `onSubmit: () => void`
- Visually subdued (lighter border, smaller)
- Placeholder: "Course-correct the AI's approach..."
- Submit on Enter (with shift+enter for newlines)

### Acceptance Criteria

- Each component renders in isolation (use Vitest component tests or a simple test page)
- All variants of status badges are visually distinct
- Attention dot animation is smooth and subtle
- No component depends on API data or global state
- All components use Tailwind classes only (shadcn/ui conventions)

---

## T11 — Tasks Domain (CRUD + Dependencies)

**Layer:** Backend (Rust — `server` crate)
**Depends on:** T08 (stories must exist)
**Outputs:** Full task CRUD with dependency DAG management

### What to Build

**`backend/server/src/tasks/`** — handler, service, repo, types

### Endpoints

```
GET    /api/v1/tasks?story_id=X            → List tasks ordered by position
POST   /api/v1/tasks                       → Create task
GET    /api/v1/tasks/:id                   → Task detail
PATCH  /api/v1/tasks/:id                   → Update (name, description, position, assignee_id, state, claude_session_id, ai_status_text)
DELETE /api/v1/tasks/:id                   → Soft delete
POST   /api/v1/tasks/:id/done              → Mark done (human sign-off)
GET    /api/v1/task-dependencies?story_id=X → List dependency edges for story
POST   /api/v1/task-dependencies           → Create dependency edge
DELETE /api/v1/task-dependencies/:id       → Remove dependency edge
```

### Request/Response Types

```rust
struct CreateTaskRequest { story_id: Uuid, name: String, description: String, task_type: String, position: i32, assignee_id: Option<Uuid> }
struct UpdateTaskRequest { name: Option<String>, description: Option<String>, position: Option<i32>, assignee_id: Option<Uuid>, state: Option<String>, claude_session_id: Option<String>, ai_status_text: Option<String> }
struct TaskResponse { id: Uuid, org_id: Uuid, story_id: Uuid, name: String, description: String, task_type: String, state: String, position: i32, assignee_id: Option<Uuid>, claude_session_id: Option<String>, ai_status_text: Option<String>, created_at: DateTime<Utc>, updated_at: DateTime<Utc>, dependencies: Vec<DependencyResponse> }
struct CreateDependencyRequest { task_id: Uuid, depends_on_task_id: Uuid }
struct DependencyResponse { id: Uuid, task_id: Uuid, depends_on_task_id: Uuid }
```

### Business Rules — Task State Machine

Valid transitions:
- `pending` → `qa` (task Q&A starts)
- `qa` → `running` (Q&A complete, implementation starts)
- `running` → `paused` (AI encounters decision)
- `paused` → `running` (human answers, AI resumes)
- `running` → `blocked` (dependency not met)
- `blocked` → `running` (dependency resolved)
- `running` → `done` (only via `/done` endpoint — human sign-off)
- Any state → `pending` (reset/cancel)

Invalid transitions return `TaskError::InvalidState`.

### Dependency DAG Validation

When creating a dependency edge:
1. Both tasks must belong to the same story
2. `task_id != depends_on_task_id`
3. Adding this edge must not create a cycle — perform DFS/BFS cycle detection
4. If cycle detected, return `TaskError::CyclicDependency`

### Mark Done Logic

`POST /tasks/:id/done`:
- Task must be in `running` state (AI has completed work)
- Sets `state = "done"`, `ai_status_text = NULL`
- Returns 400 if task is not in `running` state

### Acceptance Criteria

- Task CRUD with position ordering works
- State machine enforced (invalid transitions rejected)
- Cycle detection prevents circular dependencies
- Mark done only works from `running` state
- Integration tests for DAG cycle detection
- All types derive `ToSchema`

---

## T12 — Q&A Domain (Rounds, Answers, Rollback)

**Layer:** Backend (Rust — `server` crate)
**Depends on:** T11 (tasks must exist for task-level Q&A)
**Outputs:** Q&A round management with JSONB content, answer submission, and checkpoint rollback

### What to Build

**`backend/server/src/qa/`** — handler, service, repo, types

### Endpoints

```
GET    /api/v1/qa-rounds?story_id=X                → List story-level rounds (ordered by round_number)
GET    /api/v1/qa-rounds?task_id=X                 → List task-level rounds
GET    /api/v1/qa-rounds?story_id=X&stage=grooming → Filtered by stage
POST   /api/v1/qa-rounds/:id/answer                → Submit answer for a question
POST   /api/v1/qa-rounds/:id/rollback              → Checkpoint rollback to this round
POST   /api/v1/qa-rounds/course-correct             → Free-form course correction
```

### Data Model

The `qa_rounds` table stores relational metadata + JSONB content:

```rust
struct QaRoundRow {
    id: Uuid,
    org_id: Uuid,
    story_id: Uuid,
    task_id: Option<Uuid>,
    stage: String,
    round_number: i32,
    status: String,             // "active" or "superseded"
    content: serde_json::Value, // JSONB
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### JSONB Content Structure

```rust
#[derive(Serialize, Deserialize)]
struct QaContent {
    questions: Vec<QaQuestion>,
    course_correction: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct QaQuestion {
    id: Uuid,
    text: String,
    domain: String,
    options: Vec<String>,
    selected_answer_index: Option<i32>,
    selected_answer_text: Option<String>,
    answered_by: Option<Uuid>,
    answered_at: Option<DateTime<Utc>>,
}
```

### Answer Submission

`POST /qa-rounds/:id/answer` body:
```rust
struct SubmitAnswerRequest {
    question_id: Uuid,
    selected_answer_index: Option<i32>,  // null if "Other"
    answer_text: String,
}
```

Logic:
1. Round must have `status = "active"` (else `QaError::RoundNotActive`)
2. Question must exist in the round's content
3. Question must not already be answered (else `QaError::AlreadyAnswered`)
4. Update the JSONB content: set `selected_answer_index`, `selected_answer_text`, `answered_by`, `answered_at`
5. Use `jsonb_set` in SQL or read-modify-write pattern

### Checkpoint Rollback

`POST /qa-rounds/:id/rollback`:

1. Find the target round (must be `status = "active"`)
2. All rounds AFTER this round (higher `round_number`, same story_id+task_id+stage) are set to `status = "superseded"`
3. Unanswered questions in the target round are cleared (reset `selected_answer_*` fields)
4. Return the updated round

### Course Correction

`POST /qa-rounds/course-correct` body:
```rust
struct CourseCorrectionRequest {
    story_id: Uuid,
    task_id: Option<Uuid>,
    stage: String,
    text: String,
}
```

Creates a new round with `course_correction` field set, and `questions` as empty. The container agent picks this up and generates new questions.

### Response Types

```rust
struct QaRoundResponse {
    id: Uuid, story_id: Uuid, task_id: Option<Uuid>, stage: String,
    round_number: i32, status: String, content: QaContent,
    created_at: DateTime<Utc>, updated_at: DateTime<Utc>,
}
```

### Acceptance Criteria

- List rounds filtered by story_id, task_id, and stage
- Answer submission updates JSONB correctly
- Double-answer returns error
- Rollback marks all subsequent rounds as superseded
- Course correction creates new round
- Integration tests for full Q&A flow: create round → answer → rollback → re-answer

---

## T13 — Container Keys Domain (CRUD)

**Layer:** Backend (Rust — `server` crate)
**Depends on:** T07 (projects must exist)
**Outputs:** API key generation, listing, and revocation for container authentication

### What to Build

**`backend/server/src/agents/`** — add `keys_handler.rs`, `keys_service.rs`, `keys_repo.rs` (or place in a `container_keys/` module)

### Endpoints

```
GET    /api/v1/container-keys?project_id=X → List API keys for project (never return the raw key)
POST   /api/v1/container-keys              → Generate new API key (return raw key ONCE)
DELETE /api/v1/container-keys/:id          → Revoke key (set revoked_at)
```

### Key Generation Logic

1. Generate 32-byte cryptographically random string (use `rand::rngs::OsRng` + base64 encode)
2. Hash with bcrypt (cost factor 12)
3. Store `bcrypt(key)` in DB, return raw key to user ONCE
4. Display format: `cpk_<base64_encoded_random_bytes>` (prefix for easy identification)

### Request/Response Types

```rust
struct CreateKeyRequest { project_id: Uuid, label: String, container_mode: String }
struct CreateKeyResponse { id: Uuid, api_key: String, label: String, container_mode: String, created_at: DateTime<Utc> }
// api_key is returned ONLY on creation

struct KeyListResponse { id: Uuid, label: String, container_mode: String, last_connected_at: Option<DateTime<Utc>>, created_at: DateTime<Utc>, revoked_at: Option<DateTime<Utc>> }
```

### Key Verification Function

Provide a function used by the WebSocket handler (T16) to verify incoming container connections:

```rust
pub async fn verify_api_key(pool: &PgPool, raw_key: &str) -> Result<ContainerKeyInfo, AuthError> {
    // 1. List all non-revoked keys (WHERE revoked_at IS NULL)
    // 2. bcrypt::verify against each hash (or optimize with key prefix lookup)
    // 3. Return ContainerKeyInfo { key_id, org_id, project_id, container_mode }
}
```

### Dependencies

```toml
bcrypt = "0.15"
rand = "0.8"
base64 = "0.22"
```

### Acceptance Criteria

- Key generation returns a raw key only once
- List endpoint never exposes raw key or hash
- Revoked keys are excluded from list (or shown with `revoked_at`)
- `verify_api_key` correctly matches against bcrypt hashes
- Integration tests

---

## T14 — Knowledge Domain (CRUD)

**Layer:** Backend (Rust — `server` crate)
**Depends on:** T07 (projects must exist)
**Outputs:** Knowledge base CRUD with hierarchical scoping

### What to Build

**`backend/server/src/knowledge/`** — handler, service, repo, types

### Endpoints

```
GET    /api/v1/knowledge?project_id=X                → List project + org-level entries
GET    /api/v1/knowledge?project_id=X&story_id=Y     → Include story-scoped entries
POST   /api/v1/knowledge                              → Create entry
PATCH  /api/v1/knowledge/:id                           → Update entry
DELETE /api/v1/knowledge/:id                           → Soft delete
```

### Request/Response Types

```rust
struct CreateKnowledgeRequest { project_id: Option<Uuid>, story_id: Option<Uuid>, category: String, title: String, content: String }
struct UpdateKnowledgeRequest { title: Option<String>, content: Option<String>, category: Option<String> }
struct KnowledgeResponse { id: Uuid, org_id: Uuid, project_id: Option<Uuid>, story_id: Option<Uuid>, category: String, title: String, content: String, created_at: DateTime<Utc>, updated_at: DateTime<Utc> }
```

### Hierarchy Resolution

- `project_id = NULL` → org-level entry
- `project_id = X, story_id = NULL` → project-level entry
- `project_id = X, story_id = Y` → story-level entry

The list endpoint returns all applicable entries. The consumer (container agent) performs override resolution: story > project > org.

### Validation

- `category` must be one of: `convention`, `adr`, `api_doc`, `design_system`, `custom`
- `title` must not be empty
- `content` must not be empty

### Acceptance Criteria

- CRUD works at all three hierarchy levels
- Filtering by project_id and story_id works correctly
- Soft delete works
- Integration tests

---

## T15 — SSE Broadcaster & Endpoint

**Layer:** Backend (Rust — `server` crate)
**Depends on:** T11 (tasks for event types), T12 (Q&A for event types)
**Outputs:** SSE endpoint and in-memory broadcaster that pushes events to connected browsers

### What to Build

**`backend/server/src/sse/`** — `handler.rs`, `broadcaster.rs`

### SSE Endpoint

```
GET /api/v1/events/stream
```

- Requires JWT authentication (cookie)
- Returns `Content-Type: text/event-stream`
- One connection per user session
- Server sends all events for the user's `org_id`

### Broadcaster

```rust
pub struct SseBroadcaster {
    // Map of org_id → Vec<Sender>
    clients: DashMap<Uuid, Vec<tokio::sync::mpsc::UnboundedSender<SseEvent>>>,
}

impl SseBroadcaster {
    pub fn new() -> Self { ... }
    pub fn subscribe(&self, org_id: Uuid) -> UnboundedReceiver<SseEvent> { ... }
    pub fn unsubscribe(&self, org_id: Uuid, sender_id: Uuid) { ... }
    pub fn broadcast(&self, org_id: Uuid, event: SseEvent) { ... }
}
```

### Event Types

```rust
#[derive(Serialize)]
#[serde(tag = "event", content = "data")]
pub enum SseEvent {
    StoryUpdated { story_id: Uuid, fields: Vec<String> },
    TaskUpdated { task_id: Uuid, story_id: Uuid, fields: Vec<String> },
    NewQuestion { story_id: Uuid, task_id: Option<Uuid>, round_id: Uuid },
    TaskCompleted { task_id: Uuid, story_id: Uuid },
}
```

### Integration Point

The broadcaster is added to `AppState`. Other services call `broadcaster.broadcast(org_id, event)` after state changes:
- Story status change → `StoryUpdated`
- Task state change → `TaskUpdated`
- New Q&A round created → `NewQuestion`
- Task marked done → `TaskCompleted`

These broadcast calls will be added to the service layers of stories, tasks, and Q&A modules.

### Axum SSE Response

Use `axum::response::sse::Sse` with `tokio_stream::wrappers::UnboundedReceiverStream`.

### Acceptance Criteria

- SSE endpoint establishes persistent connection
- Events are received by connected clients within milliseconds
- Disconnected clients are cleaned up (no memory leak)
- Multiple clients per org all receive the same events
- Events are formatted as proper SSE (`event: type\ndata: json\n\n`)

---

## T16 — WebSocket Handler & Connection Registry

**Layer:** Backend (Rust — `server` crate)
**Depends on:** T13 (container key verification)
**Outputs:** WebSocket endpoint for container agents, DashMap-based connection registry

### What to Build

**`backend/server/src/agents/handler.rs`** and **`backend/server/src/agents/registry.rs`**

### WebSocket Endpoint

```
GET /ws/container   (WebSocket upgrade)
```

Handshake:
1. Container sends `Authorization: Bearer cpk_<key>` in upgrade request headers
2. Server calls `verify_api_key()` from T13
3. If valid → accept upgrade, register connection
4. If invalid → reject with 401

### Connection Registry

```rust
#[async_trait]
pub trait ConnectionRegistry: Send + Sync {
    async fn register(&self, key_id: Uuid, sender: WebSocketSender);
    async fn unregister(&self, key_id: Uuid);
    async fn send_to(&self, key_id: Uuid, msg: ServerToContainer) -> Result<()>;
    async fn is_connected(&self, key_id: Uuid) -> bool;
}

pub struct DashMapRegistry {
    connections: DashMap<Uuid, WebSocketSender>,
}
```

### Message Routing

After connection is established:
1. Spawn two tasks: one for reading, one for writing
2. **Reading:** Deserialize incoming `ContainerToServer` messages, dispatch to service layer
3. **Writing:** Receive `ServerToContainer` messages from registry's send channel, serialize and send
4. **Heartbeat:** Server sends `Ping` every 30 seconds. If 2 consecutive `Pong` missed → disconnect and unregister

### Reconnection Handling

- On container reconnect: container is re-authenticated, re-registered
- `last_connected_at` updated in `container_api_keys` table
- Old connection (if any) is cleaned up

### Dependencies

```toml
dashmap = "6"
```

### Acceptance Criteria

- WebSocket upgrade succeeds with valid API key
- WebSocket upgrade rejected with invalid key (401)
- Messages are correctly deserialized into `ContainerToServer` enum
- Registry tracks connections (register, unregister, send_to)
- Heartbeat detects dead connections
- Integration test with mock WebSocket client

---

## T17 — Container Orchestration Service

**Layer:** Backend (Rust — `server` crate)
**Depends on:** T16 (WebSocket handler)
**Outputs:** Logic that routes container messages to appropriate services and drives the pipeline

### What to Build

**`backend/server/src/agents/service.rs`**

This is the "brain" that connects container messages to the rest of the system.

### Incoming Message Handlers

When a `ContainerToServer` message arrives:

**`QuestionBatch`**: Container generated new Q&A questions.
1. Create a new `qa_rounds` row with the questions in JSONB content
2. Broadcast `NewQuestion` SSE event to the org
3. Update story's `pipeline_stage` if needed

**`TaskDecomposition`**: Container proposes task breakdown.
1. Store proposed tasks as actual `tasks` rows with `state = "pending"`
2. Create `task_dependencies` edges from the proposal
3. Broadcast `StoryUpdated` event
4. Update story `pipeline_stage = "decomposition"`

**`TaskPaused`**: Container agent paused on a decision.
1. Update task `state = "paused"`, `ai_status_text = question summary`
2. Create a new `qa_round` with the pause question
3. Broadcast `TaskUpdated` + `NewQuestion` SSE events

**`TaskCompleted`**: Container finished a task.
1. Update task `state = "running"` (awaiting human sign-off), `ai_status_text = "Completed — awaiting review"`
2. Store `commit_sha` (add column or use `ai_status_text`)
3. Broadcast `TaskCompleted` SSE event
4. Check if all tasks done → advance story `pipeline_stage`

**`TaskFailed`**: Container encountered an error.
1. Update task `state = "blocked"`, `ai_status_text = error message`
2. Broadcast `TaskUpdated` SSE event

**`StatusUpdate`**: Informational status from container.
1. Update task `ai_status_text`
2. Broadcast `TaskUpdated` SSE event

### Outgoing Message Triggers

When user actions occur in the web app:

**Story started** (from stories service):
→ Send `StartGrooming` to project container via registry

**Grooming Q&A completed** (all questions answered, AI says SUFFICIENT):
→ Send `StartPlanning` to project container

**Answer submitted** (from Q&A service):
→ Send `AnswerReceived` to appropriate container

**Task ready for execution** (dependencies met, Q&A done):
→ Send `StartTask` to dev container, generate `claude_session_id`

**Human answers pause question**:
→ Send `ResumeTask` to dev container

### Acceptance Criteria

- Each `ContainerToServer` variant correctly updates DB and broadcasts SSE
- Pipeline stage transitions happen automatically based on container messages
- `StartGrooming` is sent when story moves to "In Progress"
- Answer submissions are forwarded to the correct container
- Unit tests with mocked registry and mocked repos

---

## T18 — Agent Scaffold & Actor Framework

**Layer:** Agent (Rust — `agent` crate)
**Depends on:** T01 (shared crate)
**Outputs:** Runnable agent binary with actor model, mode selection, and config loading

### What to Build

The container agent entry point with tokio actor architecture.

### Directory Structure

```
backend/agent/src/
├── main.rs                # Entry point: load config, spawn actors by mode
├── config.rs              # TOML config loading
├── dispatcher.rs          # Central message router (actor)
├── ws_client.rs           # WebSocket client handler (T19)
├── claude_code.rs         # Claude Code subprocess supervisor — ALL LLM interaction (T20)
├── git_manager.rs         # gitoxide operations (T21)
├── setup_ui.rs            # Embedded Axum server for config UI (T24)
└── prompts/               # System prompt templates (T20)
```

### Config

**`config.toml`** (loaded from working directory or `CONFIG_PATH` env var):
```toml
mode = "standalone"           # "project", "dev", "standalone"
server_url = "wss://app.yourdomain.com/ws/container"
api_key = "cpk_..."
github_repo_url = "https://github.com/org/repo"
github_access_token = "ghp_..."
setup_ui_port = 3001
```

> **Note:** No Anthropic API key here. All LLM interaction goes through Claude Code CLI subprocess, which manages its own API key configuration via its own config/environment. The agent never calls the Anthropic API directly.

### Actor Framework

The dispatcher is the central router:

```rust
enum DispatchMessage {
    // From WebSocket
    FromServer(ServerToContainer),
    // From Claude Code (Q&A generation mode)
    QuestionsGenerated { story_id: Uuid, task_id: Option<Uuid>, round: QaRoundContent },
    TaskDecompositionReady { story_id: Uuid, tasks: Vec<ProposedTask> },
    ConvergenceResult { story_id: Uuid, task_id: Option<Uuid>, sufficient: bool },
    // From Claude Code (implementation mode)
    TaskPaused { task_id: Uuid, question: PauseQuestion },
    TaskCompleted { task_id: Uuid, commit_sha: String },
    TaskFailed { task_id: Uuid, error: String },
    // Status
    StatusUpdate { task_id: Uuid, text: String },
}
```

Each actor (ws_client, claude_code, git_manager) gets a `mpsc::Sender<DispatchMessage>` to send to the dispatcher, and receives its own typed messages via a dedicated channel.

### Mode Selection

```rust
match config.mode {
    Mode::Project => {
        spawn(ws_client);
        spawn(claude_code);   // Q&A generation mode only (no implementation)
        spawn(setup_ui);
        // No git_manager (project container doesn't commit code)
    }
    Mode::Dev => {
        spawn(ws_client);
        spawn(claude_code);   // Both Q&A generation and implementation modes
        spawn(git_manager);
        // No setup_ui (or optional)
    }
    Mode::Standalone => {
        spawn(ws_client);
        spawn(claude_code);   // All modes: Q&A generation + implementation
        spawn(git_manager);
        spawn(setup_ui);
    }
}
```

> **Key design:** There is no separate "LLM Service" actor. Claude Code subprocess handles ALL LLM interaction — both Q&A question generation (run with a structured-output prompt and `--print` flag) and code implementation (run in its normal interactive mode). The agent invokes Claude Code with different prompts and flags depending on whether it needs structured Q&A output or code implementation. See T20 for details.

### Dependencies (agent Cargo.toml)

```toml
[dependencies]
shared = { path = "../shared" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Acceptance Criteria

- `cargo run --bin agent` starts with config loaded
- Mode selection spawns correct set of actors
- Dispatcher routes messages between actors
- Config file missing → clear error message
- Unit tests for dispatcher message routing

---

## T19 — Agent WebSocket Client & Reconnection

**Layer:** Agent (Rust — `agent` crate)
**Depends on:** T18 (agent scaffold)
**Outputs:** WebSocket client that connects to the server, handles auth, heartbeat, and reconnection

### What to Build

**`backend/agent/src/ws_client.rs`**

### Connection

1. Connect to `wss://{server_url}` with `Authorization: Bearer {api_key}` header
2. On success: send current agent state to dispatcher
3. On failure: retry with random jitter (0–2 seconds), no exponential backoff

### Message Handling

- **Incoming:** Deserialize `ServerToContainer` messages, send to dispatcher
- **Outgoing:** Receive `ContainerToServer` messages from dispatcher, serialize and send
- **Ping/Pong:** Respond to server `Ping` with `Pong` immediately

### Reconnection Logic

```rust
loop {
    match connect(&config).await {
        Ok(ws) => {
            handle_connection(ws, &dispatcher_tx).await;
            // Connection dropped
            tracing::warn!("WebSocket disconnected, reconnecting...");
        }
        Err(e) => {
            tracing::error!("WebSocket connection failed: {}", e);
        }
    }
    // Jitter: 0-2 seconds
    let jitter = rand::random::<u64>() % 2000;
    tokio::time::sleep(Duration::from_millis(jitter)).await;
}
```

### State Reconciliation

On reconnect, the agent sends its current state (which tasks are running, which are paused) so the server can reconcile.

### Dependencies

```toml
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }
rand = "0.8"
```

### Acceptance Criteria

- Successfully connects and authenticates with API key
- Handles Ping/Pong heartbeat
- Reconnects automatically on disconnect
- Messages are correctly serialized/deserialized
- Integration test against mock WebSocket server

---

## T20 — Agent Claude Code Supervisor & Prompt Templates

**Layer:** Agent (Rust — `agent` crate)
**Depends on:** T19 (needs WebSocket to send results back)
**Outputs:** Unified Claude Code subprocess manager for ALL LLM interaction (Q&A generation + implementation), prompt template system, convergence logic

### What to Build

**`backend/agent/src/claude_code.rs`** and **`backend/agent/src/prompts/`**

### Architecture: One Actor, Two Modes

There is **no separate Anthropic API client**. All LLM interaction goes through Claude Code CLI subprocess. The agent invokes Claude Code in two distinct modes:

1. **Q&A Generation Mode** — Used by project container for grooming/planning Q&A and task decomposition, and by dev container for task-level Q&A. Runs Claude Code with `--print` flag and a structured-output prompt. Claude Code handles its own Anthropic API calls internally.

2. **Implementation Mode** — Used by dev container for code implementation. Runs Claude Code with session flags for pause/resume. This is the "normal" Claude Code usage with codebase access.

### Claude Code Supervisor Actor

```rust
pub struct ClaudeCodeSupervisor {
    dispatcher_tx: mpsc::Sender<DispatchMessage>,
    active_sessions: HashMap<Uuid, tokio::process::Child>, // task_id → child process
}

enum ClaudeCodeRequest {
    // Q&A Generation Mode (project + dev container)
    GenerateGroomingQuestions { story_id: Uuid, context: GroomingContext },
    GeneratePlanningQuestions { story_id: Uuid, context: PlanningContext },
    GenerateTaskDecomposition { story_id: Uuid, context: PlanningContext, grooming_decisions: Vec<QaDecision> },
    GenerateTaskQuestions { task_id: Uuid, context: TaskContext },
    AssessConvergence { story_id: Uuid, task_id: Option<Uuid>, accumulated_decisions: Vec<QaDecision> },
    // Implementation Mode (dev container only)
    StartImplementation { task_id: Uuid, session_id: String, context: TaskContext },
    ResumeImplementation { task_id: Uuid, session_id: String, answer: Answer },
    CancelImplementation { task_id: Uuid },
}
```

### Q&A Generation Mode

For structured Q&A output, invoke Claude Code as a one-shot subprocess:

```rust
async fn generate_questions(&self, prompt: &str) -> Result<QaRoundContent> {
    let output = Command::new("claude")
        .arg("--print")           // Non-interactive, output only
        .arg("--output-format").arg("json")  // Structured JSON output
        .arg("--prompt").arg(prompt)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Claude Code failed: {}", stderr));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let content: QaRoundContent = serde_json::from_str(&stdout)?;
    Ok(content)
}
```

The prompt templates (see below) instruct Claude Code to respond with structured JSON containing questions and predefined answer options.

**Convergence assessment** also uses Q&A generation mode:

```rust
async fn assess_convergence(&self, prompt: &str) -> Result<bool> {
    // One-shot Claude Code call with a prompt that asks:
    // "Do you have sufficient information to proceed? Respond only CONTINUE or SUFFICIENT."
    let output = Command::new("claude")
        .arg("--print")
        .arg("--prompt").arg(prompt)
        .output().await?;
    let response = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(response.contains("SUFFICIENT"))
}
```

### Implementation Mode

For code implementation, invoke Claude Code with session management (pause/resume):

```rust
async fn start_implementation(&self, task_id: Uuid, session_id: &str, context: &TaskContext) -> Result<()> {
    let child = Command::new("claude")
        .arg("--session-id").arg(session_id)
        .arg("--print")
        .arg("--task").arg(&build_implementation_prompt(context))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    self.active_sessions.insert(task_id, child);
    // Spawn monitoring task for stdout
    self.monitor_implementation(task_id).await;
    Ok(())
}
```

**Pause/Resume flow:**
1. Monitor stdout for decision-needed markers (structured output patterns)
2. When detected: send SIGTERM to subprocess, send `TaskPaused` to dispatcher
3. On resume: restart with `--session-id {id} --resume`

**Output parsing:**
- Parse stdout line-by-line
- Look for markers like `[DECISION_NEEDED]` or structured JSON blocks
- Extract question text and options from structured output
- Send `StatusUpdate` for progress messages

**Error handling:**
- Non-zero exit code → `TaskFailed`
- Timeout (configurable) → `TaskFailed`
- Crash → attempt restart once, then `TaskFailed`

### Prompt Templates

**`prompts/grooming/`** — One file per domain role:
- `business.rs` — BA perspective: scope, user impact, success criteria
- `design.rs` — UX: interaction patterns, accessibility
- `marketing.rs` — Community impact, positioning
- `development.rs` — Technical constraints affecting business
- `security.rs` — Data handling, auth, compliance

**`prompts/planning.rs`** — Architecture, database, API design, error handling

**`prompts/task_qa.rs`** — Implementation-specific decisions

**`prompts/task_decomposition.rs`** — Generating task breakdown proposals

**`prompts/implementation.rs`** — Claude Code execution instructions for coding

**`prompts/convergence.rs`** — Convergence assessment prompt

Each template builds a prompt string with context injection:
```rust
pub fn build_grooming_prompt(role: &str, context: &GroomingContext) -> String {
    format!(r#"
You are a {role} analyzing a software story.

## Organization Conventions
{org_conventions}

## Project Patterns
{project_patterns}

## Story Description
{story_description}

## Previous Decisions
{previous_decisions}

Generate questions with predefined answer options. Respond ONLY with valid JSON, no other text:
{{
  "questions": [
    {{ "text": "...", "domain": "...", "options": ["...", "...", "..."] }}
  ]
}}
"#,
        role = role,
        org_conventions = context.knowledge.iter()
            .filter(|k| k.category == KnowledgeCategory::Convention)
            .map(|k| k.content.as_str()).collect::<Vec<_>>().join("\n"),
        project_patterns = context.codebase_context,
        story_description = context.story_description,
        previous_decisions = "", // Empty for first round
    )
}
```

### Convergence Logic

After each round of answers:
1. Build convergence prompt with all accumulated decisions
2. Run one-shot Claude Code call: "Do you have sufficient information to proceed? Respond CONTINUE or SUFFICIENT."
3. If CONTINUE → generate more questions (another one-shot call)
4. If SUFFICIENT → notify dispatcher to advance pipeline stage

Users can also override convergence via the course correction chat input — typing "proceed" or similar instructions tells the AI to stop asking and move forward.

### Mode-Specific Behavior

| Container Mode | Q&A Generation | Implementation | Active |
|---|---|---|---|
| **Project** | ✅ Grooming, Planning, Task Decomposition | ❌ | Story-level Q&A only |
| **Dev** | ✅ Task-level Q&A | ✅ Code implementation | Task Q&A + coding |
| **Standalone** | ✅ All Q&A types | ✅ Code implementation | Everything |

### Dependencies

(No `reqwest` needed — no direct HTTP calls to Anthropic API)

### Acceptance Criteria

- Q&A generation mode: Claude Code one-shot calls produce valid JSON question batches
- Implementation mode: Claude Code subprocess starts, monitors, pauses, resumes correctly
- Prompt templates inject context variables correctly
- Each domain role (grooming) produces domain-appropriate questions
- Convergence assessment correctly determines when to stop
- Error handling: subprocess crash, timeout, non-zero exit all produce `TaskFailed`
- Unit tests with mock subprocess (mock Claude Code binary that outputs predefined responses)
- Integration test: full cycle of generate questions → parse → send to dispatcher

---

## T21 — Agent Git Manager

**Layer:** Agent (Rust — `agent` crate)
**Depends on:** T18 (agent scaffold)
**Outputs:** gitoxide-based git operations: branch creation, worktree management, commits, push

### What to Build

**`backend/agent/src/git_manager.rs`**

### Git Manager Actor

Receives git operation requests from the dispatcher and executes them using gitoxide (Rust-native git).

```rust
pub struct GitManager {
    repo_path: PathBuf,
    dispatcher_tx: mpsc::Sender<DispatchMessage>,
}

enum GitRequest {
    CreateStoryBranch { story_id: Uuid, slug: String },
    CreateWorktree { story_id: Uuid, branch: String },
    CommitTask { worktree: PathBuf, task_id: Uuid, message: String },
    Push { branch: String },
}
```

### Operations

**Branch creation:**
```rust
pub fn create_story_branch(&self, story_id: Uuid, slug: &str) -> Result<String> {
    // Create branch: story/STORY-{id}-{slug}
    // Branch from main/master
    // Return branch name
}
```

**Worktree management:**
```rust
pub fn create_worktree(&self, story_id: Uuid, branch: &str) -> Result<PathBuf> {
    // Create git worktree at a deterministic path: {repo}/.worktrees/story-{id}/
    // Link to the story branch
    // Return worktree path (this is where Claude Code will execute)
}
```

**Commits:**
```rust
pub fn commit_task(&self, worktree: &Path, task_id: Uuid, message: &str) -> Result<String> {
    // Stage all changes in the worktree
    // Create commit with message format: "[T-{task_id}] {message}"
    // Return commit SHA
}
```

**Push:**
```rust
pub fn push(&self, branch: &str) -> Result<()> {
    // Push branch to configured remote (origin)
    // Uses github_access_token from config for authentication
}
```

### Integration with T20 (Claude Code Supervisor)

The dispatcher coordinates between T20 and T21:
1. T20 receives `StartImplementation` → dispatcher first calls T21 to ensure worktree exists
2. T20's Claude Code subprocess executes within the worktree directory
3. When T20 reports `TaskCompleted` → dispatcher calls T21 to commit and push

### Dependencies

```toml
gix = "0.68"  # gitoxide
```

### Acceptance Criteria

- Branch creation follows naming convention: `story/STORY-{id}-{slug}`
- Worktrees are created per story in a deterministic location
- Multiple worktrees can coexist for parallel story execution
- Commits include task ID in message
- Push works to configured remote with token auth
- Worktree cleanup on story completion
- Integration tests with a local bare git repo (no network)

---

## T22 — OpenAPI + Orval Type Pipeline

**Layer:** Cross-cutting (Backend + Frontend)
**Depends on:** T01 (shared crate), T07, T08, T11, T13 (all CRUD modules with `ToSchema` types)
**Outputs:** OpenAPI spec generation, orval config, generated TypeScript client with TanStack Query hooks

### What to Build

### Backend: utoipa OpenAPI Generation

Add utoipa annotations to all handler functions and types across all domain modules.

**`backend/server/src/main.rs`** — Add OpenAPI doc generation:

```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::handler::login, auth::handler::callback, auth::handler::logout, auth::handler::me,
        projects::handler::list, projects::handler::create, projects::handler::get, projects::handler::update, projects::handler::delete,
        stories::handler::list, stories::handler::create, stories::handler::get, stories::handler::update, stories::handler::delete, stories::handler::update_rank, stories::handler::start,
        tasks::handler::list, tasks::handler::create, tasks::handler::get, tasks::handler::update, tasks::handler::delete, tasks::handler::mark_done,
        qa::handler::list_rounds, qa::handler::submit_answer, qa::handler::rollback, qa::handler::course_correct,
        knowledge::handler::list, knowledge::handler::create, knowledge::handler::update, knowledge::handler::delete,
    ),
    components(schemas(
        // All request/response types from all modules
    ))
)]
struct ApiDoc;
```

Add an endpoint or build script that outputs the spec:
```
GET /api/v1/openapi.json → OpenAPI 3.1 spec
```
Or a binary that writes to `frontend/src/api/openapi.json`.

### Frontend: Orval Config

**`frontend/orval.config.ts`**
```typescript
export default {
  pipeline: {
    input: './src/api/openapi.json',
    output: {
      target: './src/api/generated/client.ts',
      client: 'react-query',
      mode: 'tags-split',
      override: {
        mutator: {
          path: './src/api/custom-fetch.ts',
          name: 'customFetch',
        },
      },
    },
  },
};
```

**`frontend/src/api/custom-fetch.ts`** — fetch wrapper that includes credentials:
```typescript
export const customFetch = async <T>(config: RequestConfig): Promise<T> => {
  const response = await fetch(config.url, {
    ...config,
    credentials: 'include', // Send JWT cookie
  });
  if (!response.ok) throw new Error(await response.text());
  return response.json();
};
```

### CI Enforcement Script

**`scripts/check-api-contract.sh`**
```bash
#!/bin/bash
# 1. Build backend, extract openapi.json
# 2. Run orval to generate TypeScript client
# 3. git diff --exit-code frontend/src/api/generated/
# 4. Exit 1 if diff found
```

### Acceptance Criteria

- OpenAPI spec is generated from Rust types (no manual YAML)
- `bun run generate-api` produces TypeScript hooks matching all backend endpoints
- Generated hooks: `useGetStories()`, `useCreateStory()`, `useGetTasks()`, etc.
- CI script detects drift between backend and frontend types
- Generated code is committed to repo

---

## T23 — Frontend Feature Modules (Full UI)

**Layer:** Frontend
**Depends on:** T22 (generated API client), T10 (UI atoms)
**Outputs:** All feature modules from the UI/UX spec — the complete user-facing application

### What to Build

This is the largest frontend task. It builds all molecules, organisms, templates, and pages.

### Module Breakdown

#### 1. Auth Feature (`features/auth/`)
- Login page with "Sign in with Google" and "Sign in with GitHub" buttons
- OAuth callback handler page
- `useAuth()` hook using `GET /api/v1/auth/me`

#### 2. Projects Feature (`features/projects/`)
- Project selector (dropdown or list)
- Create project modal

#### 3. Stories Feature (`features/stories/`)

**`stories-table.tsx`** (Organism: Step 25)
- Table with 3 columns: Name, Status, Owner
- Rows are `story-table-row.tsx` (Molecule: Step 17) composing: story-type-tag, story name, attention-dot, status-badge, owner
- Drag-and-drop reordering via @dnd-kit → calls `PATCH /stories/:id/rank`
- "+ New" button → opens story creation flow
- Done stories at reduced opacity
- Full-text search bar in header

**`story-table-row.tsx`** (Molecule: Step 17)
- Composing atoms: story-type-tag (for bugs/refactors), attention-dot, status-badge
- Click → navigates to `/projects/:projectId/stories/:storyId` (opens modal)
- Drag handle affordance

**`story-creation.tsx`** (Organism: Step 27)
- Modal form: title, description (1-2 sentences), owner (dropdown), story type (feature/bug/refactor)
- After submit: `POST /stories` → display AI-expanded description → user approves → story created

**`story-modal.tsx`** (Template: Step 28)
- Custom modal (~80% viewport, centered)
- No backdrop click dismissal — only X button or Escape
- Confirmation dialog if pending questions or unsent draft (Molecule: Step 19)
- Contains breadcrumb (atom) at top
- Two-column layout (40%/60%)
- URL-driven: `/projects/:projectId/stories/:storyId`

**`story-overview.tsx`** (Organism: Step 23 — left column)
- Story description, status badge, owner
- Task list: ordered list of `task-list-item.tsx` (Molecule: Step 18)
- Click task → navigates to task view within modal

#### 4. Tasks Feature (`features/tasks/`)

**`task-list-item.tsx`** (Molecule: Step 18)
- Composing: task-type-icon, task name, attention-dot, status-badge (task variant)
- Click → navigate to task detail within modal

**`task-overview.tsx`** (Organism: Step 24 — left column in task view)
- Task description, assignee
- AI status indicator (Molecule: Step 15): state badge + status description text
- "Mark Done" button (conditionally shown when AI is done)

**`task-decomposition.tsx`** (Organism: Step 26)
- Shows AI-proposed tasks after planning Q&A
- Drag to reorder
- Merge/split actions
- "Confirm" button to lock in decomposition

#### 5. Q&A Feature (`features/qa/`)

**`qa-thread.tsx`** (Organism: Step 21)
- Stage selector at top for story-level ("Grooming" / "Planning")
- Scrollable list of question blocks
- "New question ↓" floating indicator
- Course correction chat input pinned at bottom
- Handles both correction paths: inline redirect and checkpoint rollback

**`question-block.tsx`** (Organism: Step 20)
- Domain tag, question text
- List of answer-option-cards
- "Other" option with free-form input
- Undo icon on hover for answered rounds → triggers rollback

**`answer-option-card.tsx`** (Molecule: Step 13)
- Selectable card, radio-style
- On select → immediately call `POST /qa-rounds/:id/answer`
- Selected state visually distinct

**`other-option.tsx`** (Molecule: Step 14)
- "Other" label, expands text input on select
- Submit sends custom answer text

#### 6. Decisions Feature (`features/decisions/`)

**`decision-trail.tsx`** (Organism: Step 22)
- Flat chronological list grouped by stage headers
- Active decisions at full prominence
- Superseded decisions inline with superseded-badge + muted styling

**`decision-entry.tsx`** (Molecule: Step 16)
- Domain tag, question text, answer text
- Superseded badge when applicable
- Linkable URL (for MR descriptions)

#### 7. SSE Integration (`hooks/use-sse.ts`)

```typescript
export function useSSE() {
  const queryClient = useQueryClient();
  useEffect(() => {
    const es = new EventSource('/api/v1/events/stream');
    es.addEventListener('story_updated', (e) => {
      const d = JSON.parse(e.data);
      queryClient.invalidateQueries({ queryKey: ['stories', d.story_id] });
    });
    es.addEventListener('task_updated', (e) => { ... });
    es.addEventListener('new_question', (e) => { ... });
    es.addEventListener('task_completed', (e) => { ... });
    return () => es.close();
  }, []);
}
```

#### 8. Mobile Responsive

- Modal two-column layout collapses to single column with tab bar on small screens
- Tab bar: "Overview" | "Q&A" | "Decisions"

### Acceptance Criteria

- All UI/UX spec components (Steps 1–30) are implemented
- Story table with drag-and-drop reordering works
- Story modal opens/closes correctly (no backdrop dismiss, confirmation dialog)
- Q&A thread: answer selection, "Other" free-form, checkpoint rollback
- Decision trail shows active and superseded decisions
- Task drill-in with breadcrumb navigation back to story
- SSE events trigger real-time UI updates
- URL-driven navigation (browser back works, deep links work)
- Mobile responsive layout
- Vitest component tests for key interactions

---

## T24 — Agent Setup UI (Embedded SPA)

**Layer:** Agent (Rust + React)
**Depends on:** T18 (agent scaffold)
**Outputs:** Lightweight React SPA bundled into agent binary, served on `:3001`

### What to Build

A simple single-page configuration UI.

### Setup UI Pages

**Configuration form:**
- GitHub repository URL + access token
- Server connection URL + container API key
- Operating mode selector (project / dev / standalone)
- "Save & Connect" button

> **Note:** No Anthropic API key field. Claude Code manages its own API key via its own configuration. The setup UI only configures the agent's connection to the web app server and GitHub.

**Status page (after config):**
- Connection status to server (connected/disconnected)
- Last heartbeat time
- Current mode

### Technical Approach

1. Build React SPA (tiny, can use Vite with `bun build`)
2. Embed built assets into the agent binary using `include_dir!` or similar
3. Serve via embedded Axum on port 3001 (configurable)
4. POST `/api/config` → writes to `config.toml`
5. GET `/api/status` → returns current connection status

### Acceptance Criteria

- Accessing `http://localhost:3001` shows the config UI
- Saving config writes a valid TOML file
- Agent reads config on startup
- Status page shows live connection state

---

## T25 — CI/CD Pipeline & Docker Builds

**Layer:** Infrastructure
**Depends on:** Nothing (can be built in parallel, uses project structure)
**Outputs:** GitHub Actions workflows, Dockerfiles

### What to Build

**`.github/workflows/ci.yml`**

```yaml
jobs:
  backend-check:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env: { POSTGRES_DB: test, POSTGRES_USER: test, POSTGRES_PASSWORD: test }
    steps:
      - cargo fmt --check
      - cargo clippy --all-targets -- -D warnings
      - SQLX_OFFLINE=true cargo build  # offline mode
      - cargo test

  frontend-check:
    runs-on: ubuntu-latest
    steps:
      - bun install
      - bun run typecheck
      - bun run lint
      - bun run test

  api-contract:
    runs-on: ubuntu-latest
    needs: [backend-check]
    steps:
      - Generate OpenAPI spec
      - Run orval
      - git diff --exit-code

  docker-build:
    runs-on: ubuntu-latest
    needs: [backend-check, frontend-check, api-contract]
    steps:
      - Build server image (multi-stage with cargo-chef)
      - Build agent image (multi-stage with cargo-chef)
      - Push to container registry with SHA tag
      - Update Helm values with new tag
```

### Dockerfiles

**`backend/server/Dockerfile`** — Multi-stage with cargo-chef:
1. Chef prepare (generate recipe)
2. Cook dependencies (cached layer)
3. Build application
4. Runtime: `debian:bookworm-slim` with the binary

**`backend/agent/Dockerfile`** — Same pattern, includes bundled setup UI assets.

### Acceptance Criteria

- CI runs on every PR
- Backend tests run against real Postgres
- API contract check fails if generated types are stale
- Docker images build successfully
- Images use cargo-chef caching (deps cached separately from app code)

---

## T26 — Helm Chart & ArgoCD Config

**Layer:** Infrastructure
**Depends on:** T25 (Docker images must be buildable)
**Outputs:** Helm chart for the server deployment, ArgoCD application manifest

### What to Build

**`helm/`** directory:

```
helm/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml        # Server deployment
│   ├── service.yaml           # ClusterIP service
│   ├── ingress.yaml           # Nginx ingress (HTTP + WebSocket)
│   ├── configmap.yaml         # Non-secret config
│   ├── external-secret.yaml   # OpenBao → k8s secret via ESO
│   └── _helpers.tpl
```

### Key Configuration

**`values.yaml`**:
```yaml
image:
  repository: registry.example.com/pipeline-server
  tag: "latest"  # Updated by CI
replicaCount: 1
ingress:
  host: app.yourdomain.com
  annotations:
    nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"      # SSE + WebSocket
    nginx.ingress.kubernetes.io/proxy-send-timeout: "3600"
    nginx.ingress.kubernetes.io/rate-limit-connections: "5"
    nginx.ingress.kubernetes.io/rate-limit-rps: "20"
```

**Ingress rules:**
- `/` → frontend SPA (served by server)
- `/api/v1/` → API routes
- `/ws/` → WebSocket upgrade

**External Secret:**
- Pulls `DATABASE_URL`, `JWT_SECRET`, OAuth credentials from OpenBao

### ArgoCD Application

```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: pipeline-app
spec:
  source:
    repoURL: https://github.com/org/repo
    path: helm
  destination:
    server: https://kubernetes.default.svc
    namespace: pipeline
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

### Acceptance Criteria

- `helm template` renders valid Kubernetes manifests
- Ingress routes HTTP, API, and WebSocket correctly
- External secrets are referenced (not hardcoded)
- ArgoCD auto-syncs on Helm values change
- Rate limiting configured on ingress

---

## Parallel Execution Guide

### Phase 1 — Full Parallel (no dependencies)
Start all of these simultaneously:
- **T01** Monorepo Scaffold x
- **T02** Database Migrations x
- **T03** Server Boilerplate x
- **T04** Error Handling x
- **T05** DB Pool & RLS x
- **T06** Auth Module x
- **T09** Frontend Scaffold v
- **T25** CI/CD Pipeline x

### Phase 2 — After Phase 1 foundations land
- **T07** Orgs & Projects (needs T03, T04, T05) x
- **T10** Frontend Atoms (needs T09)
- **T18** Agent Scaffold (needs T01) x
- **T26** Helm Chart (needs T25) x

### Phase 3 — Domain modules
- **T08** Stories (needs T07) x
- **T13** Container Keys (needs T07) x
- **T14** Knowledge (needs T07) x
- **T19** Agent WebSocket Client (needs T18) x

### Phase 4 — Dependent domains
- **T11** Tasks (needs T08) x
- **T12** Q&A (needs T11) x
- **T15** SSE (needs T11, T12) x
- **T16** WebSocket Handler (needs T13) x
- **T20** Claude Code Supervisor & Prompts (needs T19) x
- **T21** Git Manager (needs T18 — parallel with T19, T20, T24) x
- **T24** Agent Setup UI (needs T18 — parallel with T19, T20, T21) x

### Phase 5 — Integration
- **T17** Orchestration Service (needs T16) x
- **T22** OpenAPI + Orval Pipeline (needs T01, T07, T08, T11, T13) x

### Phase 6 — Final assembly
- **T23** Frontend Feature Modules (needs T22, T10)

---

## Notes

- **Same codebase, no branching:** Tasks are isolated by directory. Backend domain modules (`stories/`, `tasks/`, `qa/`, etc.) don't touch each other's files. Frontend atoms vs. features are separate directories.
- **Integration points:** The SSE broadcaster (T15) and orchestration service (T17) are the main integration points. They call into other services but those services don't need to know about them.
- **Type pipeline (T22)** is a bottleneck — the frontend feature modules (T23) can't start until the generated API client exists. However, frontend atoms (T10) can be built in parallel with all backend work.
- **Agent tasks (T18–T21, T24)** are almost entirely independent from server/frontend work — they only share the `shared` crate (T01).