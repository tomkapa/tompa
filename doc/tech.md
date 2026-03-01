# Technical Architecture Decision Record

> AI-Integrated Development Pipeline
> Decided: February 2026 — 13 rounds, 65 decisions

---

## 1. System Architecture

### 1.1 Repository Structure

**Decision:** Monorepo with nested backend directory.

```
project-root/
├── backend/
│   ├── server/          # Web app API server (Rust)
│   ├── agent/           # Container agent (Rust)
│   └── shared/          # Shared types crate (Rust)
├── frontend/            # React SPA
├── helm/                # Kubernetes Helm chart (app only)
├── Cargo.toml           # Workspace root
└── .github/workflows/   # CI/CD
```

The three Rust crates (`server`, `agent`, `shared`) form a Cargo workspace rooted at `backend/`. The `shared` crate contains WebSocket message schemas, domain enums, and serialization types consumed by both `server` and `agent`. Frontend and Helm charts live at the repository root alongside the backend directory.

### 1.2 Deployment Infrastructure

**Decision:** Kubernetes on existing k3s cluster. GitOps with ArgoCD.

The platform deploys onto a pre-existing Kubernetes cluster running:

- **k3s** — Lightweight Kubernetes distribution
- **ArgoCD** — GitOps continuous delivery
- **OpenBao** — Secrets management (Vault fork)
- **Nginx Ingress** — HTTP/WebSocket routing
- **Cloudflare** — DNS management

A single Helm chart defines the application deployment (server + ingress). Postgres is managed externally in a dedicated infrastructure repository, keeping the application chart focused on its own domain. ArgoCD watches the Helm values for image tag changes and reconciles automatically.

### 1.3 Multi-tenancy Model

**Decision:** Shared database with `org_id` column scoping.

All organizations share a single Postgres database. Every table that contains tenant-scoped data includes an `org_id` column. Enforcement is dual-layered:

1. **Application layer:** Axum middleware extracts `org_id` from the authenticated JWT and injects it as a request extension. Repository functions require `org_id` as a mandatory parameter — enforced at compile time via function signatures.
2. **Database layer:** Postgres Row-Level Security (RLS) policies set `current_setting('app.org_id')` per connection and enforce `WHERE org_id = current_setting('app.org_id')` on every query, acting as a safety net if application code ever bypasses the middleware.

---

## 2. Tech Stack

### 2.1 Frontend

| Component | Technology |
|-----------|------------|
| Framework | React 18+ |
| Build tool | Vite |
| Runtime | Bun |
| UI components | shadcn/ui (Radix primitives) |
| State (server) | TanStack Query |
| State (UI) | Zustand |
| Routing | TanStack Router (type-safe) |
| Drag-and-drop | @dnd-kit |
| API client | Generated via orval from OpenAPI spec |
| SSE client | Native EventSource API + custom `useSSE()` hook |
| Story ordering | `fractional-indexing` npm package (lexorank) |
| Testing | Vitest (unit/component) + Playwright (E2E) |

### 2.2 Backend (Web App Server)

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Async runtime | tokio |
| HTTP framework | axum |
| Database driver | sqlx (compile-time checked queries) |
| Error handling | thiserror (domain errors) + anyhow (infrastructure) |
| API documentation | utoipa (OpenAPI generation) |
| Migrations | Embedded sqlx migrations (run at startup) |
| WebSocket | axum built-in WebSocket support |
| SSE | axum SSE response type |
| Connection registry | DashMap (trait-backed for future Redis swap) |

### 2.3 Container Agent

| Component | Technology |
|-----------|------------|
| Language | Rust (shared monorepo) |
| Architecture | Actor model (tokio tasks + mpsc channels) |
| Claude Code | CLI subprocess with session flags |
| Git operations | gitoxide (Rust-native) |
| Setup UI | Embedded Axum server + bundled React SPA |
| LLM calls | Anthropic API (customer's key, called from container) |

### 2.4 Database

| Component | Technology |
|-----------|------------|
| DBMS | PostgreSQL |
| ID strategy | UUIDv7 (time-ordered) |
| Migrations | sqlx embedded, run programmatically at app startup |
| Query validation | sqlx compile-time checking (`sqlx::query!` macro) |

### 2.5 Infrastructure

| Component | Technology |
|-----------|------------|
| Orchestration | k3s (Kubernetes) |
| CD | ArgoCD (GitOps) |
| Secrets | OpenBao + External Secrets Operator |
| Ingress | Nginx |
| DNS | Cloudflare |
| CI | GitHub Actions |
| Docker build | Multi-stage with cargo-chef |

---

## 3. Authentication & Security

### 3.1 User Authentication

**Decision:** OAuth2 only (Google + GitHub). No password infrastructure.

OAuth2 flow:

1. User clicks "Sign in with Google/GitHub" → redirect to provider
2. Provider callback hits `/api/v1/auth/callback/:provider`
3. Server exchanges code for tokens, extracts user profile
4. Server creates or updates user record, creates JWT
5. JWT set in HTTP-only secure cookie with user ID, org ID, and role claims

Session is stateless via JWT. No server-side session store. Token includes:

```json
{
  "sub": "user-uuid-v7",
  "org_id": "org-uuid-v7",
  "role": "admin",
  "exp": 1234567890,
  "iat": 1234567890
}
```

Revocation requires a blocklist (Redis or DB table) checked at middleware level — deferred until needed.

### 3.2 Container Authentication

**Decision:** API key per container, server-generated, bcrypt-hashed in DB.

Flow:

1. User creates a "Container Connection" in the web app for a project
2. Server generates a cryptographically random API key, displays it once
3. Server stores `bcrypt(api_key)` in `container_api_keys` table with `project_id` and `org_id`
4. User configures the container with the API key (via setup UI or environment variable)
5. Container sends the API key in the WebSocket handshake `Authorization` header
6. Server verifies against stored hash, associates the WebSocket with the project

**Future upgrade:** Mutual TLS (mTLS) via OpenBao-issued client certificates for stronger identity guarantees.

### 3.3 Rate Limiting

**Decision:** Nginx ingress rate limiting only.

Rate limits configured at the Nginx ingress controller level with per-IP and per-path rules. No application-level rate limiting in v1. Sufficient for early-stage protection against abuse and accidental DDoS.

### 3.4 CORS Policy

**Decision:** Same origin in production (API under subpath), separate in development.

- **Production:** `app.yourdomain.com` serves the frontend SPA. `app.yourdomain.com/api/v1/...` serves the API. Nginx routes by path prefix. No CORS headers needed.
- **Development:** Vite dev server proxies `/api` requests to the local Axum server. No permissive CORS configuration required.

---

## 4. Database Design

### 4.1 Core Principles

- **UUIDv7** for all primary keys — time-ordered for better B-tree index performance and natural chronological sorting
- **Soft delete** via `deleted_at` timestamp on all major entities — `WHERE deleted_at IS NULL` in all queries
- **`org_id`** on every tenant-scoped table with RLS enforcement
- **Compile-time checked queries** via `sqlx::query!` macro
- **Embedded migrations** — sqlx migrations run programmatically before the server starts serving traffic

### 4.2 Entity Relationship Overview

```
organizations
├── org_members (join: users ↔ organizations, with role)
├── projects
│   ├── container_api_keys
│   └── stories
│       ├── qa_rounds (story-level grooming + planning)
│       └── tasks
│           ├── task_dependencies (DAG edges)
│           └── qa_rounds (task-level Q&A + implementation pauses)
└── knowledge_entries (org-level defaults)
    └── knowledge_entries (project-level overrides)
```

### 4.3 Key Tables

#### organizations

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| name | text | |
| created_at | timestamptz | |
| updated_at | timestamptz | |
| deleted_at | timestamptz | Nullable, soft delete |

#### users

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| email | text | Unique |
| display_name | text | |
| avatar_url | text | Nullable |
| oauth_provider | text | "google" or "github" |
| oauth_provider_id | text | Provider's user ID |
| created_at | timestamptz | |
| updated_at | timestamptz | |
| deleted_at | timestamptz | Nullable |

#### org_members

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| org_id | uuid | FK → organizations |
| user_id | uuid | FK → users |
| role | text | "owner", "admin", "member" |
| created_at | timestamptz | |

Unique constraint on `(org_id, user_id)`. Users can belong to multiple organizations.

#### projects

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| org_id | uuid | FK → organizations |
| name | text | |
| description | text | Nullable |
| github_repo_url | text | Nullable |
| created_at | timestamptz | |
| updated_at | timestamptz | |
| deleted_at | timestamptz | Nullable |

#### stories

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| org_id | uuid | FK → organizations (denormalized for RLS) |
| project_id | uuid | FK → projects |
| title | text | |
| description | text | Brief + AI-expanded |
| story_type | text | "feature", "bug", "refactor" |
| status | text | "todo", "in_progress", "done" |
| owner_id | uuid | FK → users |
| rank | text | Fractional index (lexorank) for priority ordering |
| pipeline_stage | text | Internal: "grooming", "planning", "decomposition", "implementation", "testing", "review". Nullable for "todo"/"done" |
| created_at | timestamptz | |
| updated_at | timestamptz | |
| deleted_at | timestamptz | Nullable |

The `status` field is user-facing (To Do / In Progress / Done). The `pipeline_stage` field tracks the internal AI pipeline progress within "In Progress" and is not exposed as a user-visible status.

#### tasks

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| org_id | uuid | FK → organizations (denormalized for RLS) |
| story_id | uuid | FK → stories |
| name | text | |
| description | text | |
| task_type | text | "design", "test", "code" |
| state | text | "pending", "qa", "running", "paused", "blocked", "done" |
| position | integer | Display/execution order within story |
| assignee_id | uuid | FK → users. Nullable |
| claude_session_id | text | Nullable. Server-controlled Claude Code session ID |
| ai_status_text | text | Nullable. Brief description: "implementing file X", "paused on question" |
| created_at | timestamptz | |
| updated_at | timestamptz | |
| deleted_at | timestamptz | Nullable |

#### task_dependencies

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| task_id | uuid | FK → tasks (the dependent task) |
| depends_on_task_id | uuid | FK → tasks (the prerequisite) |
| created_at | timestamptz | |

Unique constraint on `(task_id, depends_on_task_id)`. Forms a DAG. Application layer validates no cycles on insert.

#### qa_rounds

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| org_id | uuid | FK → organizations (denormalized for RLS) |
| story_id | uuid | FK → stories |
| task_id | uuid | Nullable FK → tasks (null for story-level rounds) |
| stage | text | "grooming", "planning", "task_qa", "implementation" |
| round_number | integer | Sequential within (story_id, task_id, stage) |
| status | text | "active", "superseded" |
| content | jsonb | Questions, options, and selected answers |
| created_at | timestamptz | |
| updated_at | timestamptz | |

The `content` JSONB structure:

```json
{
  "questions": [
    {
      "id": "q-uuid",
      "text": "Token storage strategy for mobile clients?",
      "domain": "Security",
      "options": [
        "Secure keychain (iOS) / Keystore (Android)",
        "Encrypted SharedPreferences / UserDefaults",
        "In-memory only (re-auth on app restart)"
      ],
      "selected_answer_index": 0,
      "selected_answer_text": "Secure keychain (iOS) / Keystore (Android)",
      "answered_by": "user-uuid",
      "answered_at": "2026-02-28T10:30:00Z"
    }
  ],
  "course_correction": null
}
```

#### container_api_keys

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| org_id | uuid | FK → organizations |
| project_id | uuid | FK → projects |
| key_hash | text | bcrypt hash of the API key |
| label | text | User-assigned name for the key |
| container_mode | text | "project", "dev", "standalone" |
| last_connected_at | timestamptz | Nullable |
| created_at | timestamptz | |
| revoked_at | timestamptz | Nullable |

#### knowledge_entries

| Column | Type | Notes |
|--------|------|-------|
| id | uuid (v7) | PK |
| org_id | uuid | FK → organizations |
| project_id | uuid | Nullable FK → projects (null = org-level) |
| story_id | uuid | Nullable FK → stories (null = not story-scoped) |
| category | text | "convention", "adr", "api_doc", "design_system", "custom" |
| title | text | |
| content | text | Markdown/plain text |
| created_at | timestamptz | |
| updated_at | timestamptz | |
| deleted_at | timestamptz | Nullable |

Knowledge hierarchy resolution: story-level > project-level > org-level. Lower-level entries override higher-level defaults for the same category/topic.

---

## 5. API Design

### 5.1 Style

**Decision:** REST with flat URLs and query filters. Versioned under `/api/v1/`.

All endpoints require authentication (JWT cookie) except OAuth callback routes. Every tenant-scoped endpoint is filtered by the middleware-injected `org_id`.

### 5.2 Endpoint Map

#### Authentication

```
GET  /api/v1/auth/login/:provider          → Redirect to OAuth provider
GET  /api/v1/auth/callback/:provider       → OAuth callback, set JWT cookie
POST /api/v1/auth/logout                   → Clear JWT cookie
GET  /api/v1/auth/me                       → Current user + org info
```

#### Organizations & Projects

```
GET    /api/v1/orgs                        → List user's organizations
POST   /api/v1/orgs                        → Create organization
GET    /api/v1/projects?org_id=X           → List projects in org
POST   /api/v1/projects                    → Create project
GET    /api/v1/projects/:id                → Project detail
PATCH  /api/v1/projects/:id                → Update project
DELETE /api/v1/projects/:id                → Soft delete project
```

#### Stories

```
GET    /api/v1/stories?project_id=X        → List stories (ordered by rank)
POST   /api/v1/stories                     → Create story
GET    /api/v1/stories/:id                 → Story detail (includes tasks)
PATCH  /api/v1/stories/:id                 → Update story (title, description, status, owner)
DELETE /api/v1/stories/:id                 → Soft delete story
PATCH  /api/v1/stories/:id/rank            → Update rank (reorder)
POST   /api/v1/stories/:id/start           → Move to "In Progress", trigger pipeline
```

#### Tasks

```
GET    /api/v1/tasks?story_id=X            → List tasks for story (ordered by position)
POST   /api/v1/tasks                       → Create task (manual or from AI proposal)
GET    /api/v1/tasks/:id                   → Task detail
PATCH  /api/v1/tasks/:id                   → Update task (name, position, assignee, state)
DELETE /api/v1/tasks/:id                   → Soft delete task
POST   /api/v1/tasks/:id/done              → Mark task as done (human sign-off)
GET    /api/v1/task-dependencies?story_id=X → List dependency edges
POST   /api/v1/task-dependencies           → Create dependency edge
DELETE /api/v1/task-dependencies/:id       → Remove dependency edge
```

#### Q&A

```
GET    /api/v1/qa-rounds?story_id=X                → Story-level rounds
GET    /api/v1/qa-rounds?task_id=X                 → Task-level rounds
GET    /api/v1/qa-rounds?story_id=X&stage=grooming → Filtered by stage
POST   /api/v1/qa-rounds/:id/answer                → Submit answer for a question
POST   /api/v1/qa-rounds/:id/rollback              → Checkpoint rollback to this round
POST   /api/v1/qa-rounds/course-correct             → Free-form course correction
```

#### Container Management

```
GET    /api/v1/container-keys?project_id=X → List API keys for project
POST   /api/v1/container-keys              → Generate new API key
DELETE /api/v1/container-keys/:id          → Revoke key
```

#### SSE

```
GET    /api/v1/events/stream               → SSE connection (authenticated via cookie)
```

#### Knowledge Base

```
GET    /api/v1/knowledge?project_id=X      → List knowledge entries
POST   /api/v1/knowledge                   → Create entry
PATCH  /api/v1/knowledge/:id               → Update entry
DELETE /api/v1/knowledge/:id               → Soft delete entry
```

### 5.3 Type Pipeline

The API contract flows through a fully automated type pipeline:

```
Rust structs (#[derive(ToSchema, Serialize)])
    ↓ utoipa
OpenAPI 3.1 spec (generated at build time)
    ↓ orval
TypeScript types + TanStack Query hooks (generated)
    ↓
Frontend consumes: useGetStories(), useCreateStory(), etc.
```

CI enforces sync: a GitHub Actions step regenerates the OpenAPI spec and TypeScript client. If the generated files differ from what's committed, CI fails. This guarantees the frontend SDK always matches the server's actual API shape.

---

## 6. Real-time Communication

### 6.1 Architecture Overview

The system has two real-time channels:

1. **WebSocket** — between server and container agents (bidirectional)
2. **SSE** — between server and browser clients (unidirectional server→browser)

```
Container (Project/Dev)
    ↕ WebSocket (JSON envelope)
Server (Axum)
    → SSE (event stream)
Browser (React)
    → HTTP POST (actions)
Server
```

### 6.2 Server ↔ Container WebSocket Protocol

#### Message Envelope

All messages use a typed JSON envelope via serde and the shared crate:

```rust
// shared/src/messages.rs
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
enum ServerToContainer {
    StartGrooming { story_id: Uuid, context: GroomingContext },
    StartPlanning { story_id: Uuid, context: PlanningContext },
    AnswerReceived { round_id: Uuid, answers: Vec<Answer> },
    StartTask { task_id: Uuid, session_id: String, context: TaskContext },
    ResumeTask { task_id: Uuid, session_id: String, answer: Answer },
    CancelTask { task_id: Uuid },
    Ping,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
enum ContainerToServer {
    QuestionBatch { story_id: Uuid, task_id: Option<Uuid>, round: QaRoundContent },
    TaskDecomposition { story_id: Uuid, proposed_tasks: Vec<ProposedTask> },
    TaskPaused { task_id: Uuid, question: PauseQuestion },
    TaskCompleted { task_id: Uuid, commit_sha: String },
    TaskFailed { task_id: Uuid, error: String },
    StatusUpdate { task_id: Uuid, status_text: String },
    Pong,
}
```

#### Connection Lifecycle

1. Container initiates WebSocket connection to `wss://app.yourdomain.com/ws/container`
2. API key sent in `Authorization` header during handshake
3. Server verifies key hash, registers connection in DashMap
4. Heartbeat: server sends `Ping` every 30 seconds, container responds `Pong`
5. If pong missed for 2 consecutive intervals, server considers connection dead and removes from registry

#### Reconnection (Container-side)

- On disconnect: immediate reconnect attempt with random jitter (0–2 seconds)
- On failure: retry with jitter, no exponential backoff (quick recovery is prioritized)
- On reconnect: container sends its current state; server reconciles
- Claude Code sessions survive disconnects — agent resumes by session ID after reconnecting

### 6.3 Server → Browser SSE

#### Connection

Single SSE connection per authenticated user session:

```
GET /api/v1/events/stream
Cookie: session=<jwt>
Accept: text/event-stream
```

Server sends all events for the user's current org. Frontend filters by relevance.

#### Event Types

```
event: story_updated
data: {"story_id": "...", "fields": ["status", "pipeline_stage"]}

event: task_updated
data: {"task_id": "...", "story_id": "...", "fields": ["state", "ai_status_text"]}

event: new_question
data: {"story_id": "...", "task_id": "...", "round_id": "..."}

event: task_completed
data: {"task_id": "...", "story_id": "..."}
```

#### Frontend Integration

The `useSSE()` hook connects on mount, listens for events, and invalidates TanStack Query caches:

```typescript
// Pseudocode
const eventSource = new EventSource('/api/v1/events/stream');
eventSource.addEventListener('story_updated', (e) => {
  const data = JSON.parse(e.data);
  queryClient.invalidateQueries({ queryKey: ['stories', data.story_id] });
});
eventSource.addEventListener('new_question', (e) => {
  const data = JSON.parse(e.data);
  queryClient.invalidateQueries({ queryKey: ['qa-rounds', { story_id: data.story_id }] });
});
```

EventSource auto-reconnects natively. On reconnect, all active query caches are invalidated (at-most-once delivery — reconnect triggers full re-fetch).

### 6.4 Connection Registry

Server tracks active WebSocket connections using DashMap:

```rust
type ContainerRegistry = DashMap<Uuid, WebSocketSender>; // key = container_api_key.id
```

The registry is wrapped behind a trait to allow future swap to a Redis-backed implementation for horizontal scaling:

```rust
#[async_trait]
trait ConnectionRegistry: Send + Sync {
    async fn register(&self, id: Uuid, sender: WebSocketSender);
    async fn unregister(&self, id: Uuid);
    async fn send_to(&self, id: Uuid, msg: ServerToContainer) -> Result<()>;
    async fn is_connected(&self, id: Uuid) -> bool;
}
```

---

## 7. Backend Application Structure

### 7.1 Domain Modules with Layered Internals

```
backend/server/src/
├── main.rs                    # Startup, migration, router assembly
├── config.rs                  # Environment/config loading
├── auth/
│   ├── mod.rs
│   ├── handler.rs             # OAuth routes, callback, logout
│   ├── middleware.rs           # JWT extraction, org_id injection
│   ├── service.rs             # JWT creation/validation, OAuth exchange
│   └── types.rs               # AuthClaims, OAuthProfile
├── stories/
│   ├── mod.rs
│   ├── handler.rs             # HTTP handlers (request → response)
│   ├── service.rs             # Business logic (validation, state transitions)
│   ├── repo.rs                # Database queries (sqlx::query!)
│   └── types.rs               # CreateStoryRequest, StoryResponse, StoryError
├── tasks/
│   ├── mod.rs
│   ├── handler.rs
│   ├── service.rs
│   ├── repo.rs
│   └── types.rs
├── qa/
│   ├── mod.rs
│   ├── handler.rs
│   ├── service.rs
│   ├── repo.rs
│   └── types.rs
├── agents/
│   ├── mod.rs
│   ├── handler.rs             # WebSocket upgrade + message routing
│   ├── service.rs             # Container orchestration logic
│   ├── registry.rs            # DashMap connection registry
│   └── types.rs
├── knowledge/
│   ├── mod.rs
│   ├── handler.rs
│   ├── service.rs
│   ├── repo.rs
│   └── types.rs
├── projects/
│   ├── mod.rs
│   ├── handler.rs
│   ├── service.rs
│   ├── repo.rs
│   └── types.rs
├── sse/
│   ├── mod.rs
│   ├── handler.rs             # SSE endpoint
│   └── broadcaster.rs         # Event broadcasting to connected clients
├── errors.rs                  # Unified ApiError enum
└── db.rs                      # Pool creation, RLS setup
```

### 7.2 Layer Responsibilities

**Handler layer** (`handler.rs`): Extracts request data (path params, query params, JSON body, auth context). Calls service layer. Maps service results to HTTP responses. No business logic.

**Service layer** (`service.rs`): Contains all business logic — validation, state machine transitions, authorization checks, orchestration. Calls repository layer for data access. Returns domain-specific `Result<T, DomainError>`.

**Repository layer** (`repo.rs`): Pure database operations. Each function takes `&PgPool` and `org_id` as mandatory parameters. Uses `sqlx::query!` for compile-time checked SQL. Returns raw data structs — no business logic.

**Types** (`types.rs`): Request/response structs with `#[derive(Serialize, Deserialize, ToSchema)]`. Domain error enums with `#[derive(thiserror::Error)]`.

### 7.3 Error Handling

Each domain defines its own error enum:

```rust
// stories/types.rs
#[derive(Debug, thiserror::Error)]
pub enum StoryError {
    #[error("Story not found")]
    NotFound,
    #[error("Invalid status transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
    #[error("Story has active tasks, cannot delete")]
    HasActiveTasks,
}
```

A top-level `ApiError` wraps all domain errors and implements `IntoResponse`:

```rust
// errors.rs
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Story(#[from] StoryError),
    #[error(transparent)]
    Task(#[from] TaskError),
    #[error(transparent)]
    Qa(#[from] QaError),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Story(StoryError::NotFound) => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::Story(StoryError::InvalidTransition { .. }) => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into()),
            // ... other mappings
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

`anyhow` is used only in infrastructure code (startup, migration, configuration loading) — never in domain service logic.

---

## 8. Frontend Application Structure

### 8.1 Directory Layout

```
frontend/src/
├── main.tsx                       # Entry point
├── router.tsx                     # TanStack Router configuration
├── App.tsx                        # Root component (SSE provider, query client)
├── api/
│   └── generated/                 # orval-generated client + hooks (DO NOT EDIT)
├── components/
│   └── ui/                        # Shared primitives (shadcn + custom atoms)
│       ├── status-badge.tsx
│       ├── attention-dot.tsx
│       ├── domain-tag.tsx
│       ├── story-type-tag.tsx
│       ├── task-type-icon.tsx
│       ├── superseded-badge.tsx
│       ├── rollback-badge.tsx
│       ├── breadcrumb.tsx
│       ├── tab-switcher.tsx
│       ├── mark-done-button.tsx
│       ├── new-question-indicator.tsx
│       └── course-correction-input.tsx
├── features/
│   ├── auth/
│   │   └── ...                    # OAuth login page, callback handler
│   ├── projects/
│   │   └── ...                    # Project selection/creation
│   ├── stories/
│   │   ├── stories-table.tsx      # Main table view with @dnd-kit
│   │   ├── story-table-row.tsx
│   │   ├── story-modal.tsx        # Custom modal (portal + focus trap)
│   │   ├── story-overview.tsx     # Left column: description, status, task list
│   │   ├── story-creation.tsx     # New story form + AI expansion
│   │   └── hooks/
│   │       └── use-story-reorder.ts
│   ├── tasks/
│   │   ├── task-list-item.tsx
│   │   ├── task-overview.tsx      # Left column: description, AI status, mark done
│   │   ├── task-decomposition.tsx # Review/merge/split AI-proposed tasks
│   │   └── hooks/
│   │       └── ...
│   ├── qa/
│   │   ├── qa-thread.tsx          # Chat-like Q&A thread
│   │   ├── question-block.tsx     # Single question with answer cards
│   │   ├── answer-option-card.tsx
│   │   ├── other-option.tsx       # "Other" with expandable free-form
│   │   └── hooks/
│   │       └── use-qa-rounds.ts
│   ├── decisions/
│   │   ├── decision-trail.tsx     # Chronological decision log
│   │   ├── decision-entry.tsx
│   │   └── hooks/
│   │       └── ...
│   └── knowledge/
│       └── ...
├── stores/
│   ├── ui-store.ts                # Zustand: modal state, active tab, drafts
│   └── sse-store.ts               # Zustand: SSE connection state
├── hooks/
│   ├── use-sse.ts                 # SSE connection + query invalidation
│   └── use-auth.ts                # Auth context
└── lib/
    ├── fractional-indexing.ts     # Lexorank helpers
    └── utils.ts
```

### 8.2 State Management Split

**TanStack Query** (server state — all API data):

- Stories list, story details
- Tasks list, task details
- Q&A rounds, decision trail
- Projects, org info, user profile
- Caching, background re-fetching, loading/error states

**Zustand** (UI state — ephemeral, client-only):

- Which story modal is open
- Active tab (Q&A Thread vs Decision Trail)
- Draft text in course correction input
- Scroll position tracking for "New question ↓" indicator
- Expanded/collapsed state for UI sections

### 8.3 Routing

TanStack Router with type-safe route definitions:

```
/                                  → Redirect to default project
/projects/:projectId               → Stories table
/projects/:projectId/stories/:storyId              → Story modal open
/projects/:projectId/stories/:storyId/tasks/:taskId → Task detail in modal
```

URL reflects full application state. Modal open/close, task drill-in, and navigation back are all URL-driven. Browser back button works naturally. Deep linking to specific stories and tasks is supported.

---

## 9. Container Agent Architecture

### 9.1 Single Image, Multi-mode

One Docker image supports three operating modes via the `MODE` environment variable:

| Mode | `MODE` value | Services active |
|------|-------------|-----------------|
| Project | `project` | Story Q&A generation, task decomposition, setup UI |
| Dev | `dev` | Task Q&A generation, Claude Code execution, git operations |
| Standalone | `standalone` | All services (project + dev merged) |

### 9.2 Actor Model

The agent runs as a set of tokio tasks communicating via `mpsc` channels:

```
┌─────────────────────────────────────────────┐
│  Agent Process                               │
│                                              │
│  ┌──────────────┐    ┌──────────────────┐   │
│  │  WebSocket    │───→│  Dispatcher       │   │
│  │  Handler      │←───│  (central router) │   │
│  └──────────────┘    └──────┬───────────┘   │
│                             │               │
│         ┌───────────────────┼────────┐      │
│         ▼                   ▼        ▼      │
│  ┌──────────────┐ ┌────────────┐ ┌───────┐ │
│  │  LLM Service  │ │ Claude Code│ │  Git  │ │
│  │  (Anthropic   │ │ Supervisor │ │Manager│ │
│  │   API calls)  │ │(subprocess)│ │(oxide)│ │
│  └──────────────┘ └────────────┘ └───────┘ │
│                                              │
│  ┌──────────────┐                           │
│  │  Setup UI     │ (Axum on separate port)  │
│  │  (config)     │                           │
│  └──────────────┘                           │
└─────────────────────────────────────────────┘
```

Each actor owns its state and communicates exclusively through typed channel messages. No shared mutable state.

### 9.3 Claude Code Integration

The agent invokes Claude Code via CLI subprocess with session management:

```bash
# Start new task
claude-code --session-id <server-assigned-id> --task "implement OAuth2 provider integration"

# Resume after pause
claude-code --session-id <server-assigned-id> --resume
```

**Pause/resume flow:**

1. Claude Code writes to stdout during execution
2. Agent monitors stdout for decision-needed markers (structured output patterns)
3. When detected: agent kills Claude Code process gracefully, sends `TaskPaused` to server
4. User answers in web app → server sends `ResumeTask` → agent receives answer
5. Agent restarts Claude Code with `--session-id X --resume` — Claude Code's built-in session persistence restores state

**Future migration path:** If output parsing proves fragile, upgrade to MCP `ask_human` tool where Claude Code explicitly calls a tool to request human input, giving the agent structured interception.

### 9.4 Git Operations (gitoxide)

The agent uses gitoxide for all git operations:

- **Branch creation:** `story/STORY-{id}-{slug}` per story
- **Worktree management:** One git worktree per active story, enabling parallel story execution
- **Commits:** One commit per completed task, message references task ID and key decisions
- **Push:** After each task commit, push to remote

### 9.5 Setup UI

The agent binary embeds a lightweight Axum server on a configurable port (default `:3001`) serving a bundled React SPA for initial configuration:

**v1 configuration options:**

- GitHub repository URL and access token
- Anthropic API key
- Server connection: web app URL + container API key
- Operating mode selection (project / dev / standalone)

Configuration is written to a local TOML file and read by the agent on startup.

### 9.6 System Prompt Management

Prompt templates are hardcoded in the agent crate, organized by domain role:

```
backend/agent/src/prompts/
├── grooming/
│   ├── business.rs        # BA perspective prompt template
│   ├── design.rs          # UX perspective
│   ├── marketing.rs       # Marketing perspective
│   ├── development.rs     # Dev constraints perspective
│   └── security.rs        # Security perspective
├── planning.rs            # Technical planning prompt
├── task_qa.rs             # Task-level Q&A prompt
└── implementation.rs      # Implementation prompt for Claude Code
```

Each template has variable injection points filled from the knowledge base:

- `{{org_conventions}}` — org-level defaults (tech stack, coding standards)
- `{{project_patterns}}` — project-level overrides (APIs, design system)
- `{{story_decisions}}` — grooming and planning answers for this story
- `{{sibling_task_decisions}}` — decisions from completed tasks in the same story
- `{{codebase_context}}` — detected patterns, APIs, and conventions from repo analysis

### 9.7 Convergence Logic

After each answered Q&A round, the agent prompts the LLM with all accumulated context and asks it to self-assess whether it has sufficient information to proceed. The LLM responds with `CONTINUE` (generate more questions) or `SUFFICIENT` (move to next stage).

Users can also override convergence via the course correction chat input — typing "proceed" or similar instructions tells the AI to stop asking and move forward. This gives users explicit control over the Q&A depth.

---

## 10. CI/CD Pipeline

### 10.1 GitHub Actions (CI)

Triggered on every pull request and push to main:

```yaml
# Simplified workflow structure
jobs:
  backend-check:
    - Cargo fmt check
    - Cargo clippy (all crates)
    - sqlx prepare (offline mode validation)
    - Cargo test (unit + integration against Postgres service container)

  frontend-check:
    - Bun install
    - TypeScript type check
    - ESLint
    - Vitest run
    - Playwright (E2E against built frontend + test server)

  api-contract:
    - Generate OpenAPI spec from Rust types
    - Run orval to generate TypeScript client
    - git diff --exit-code (fail if generated files differ)

  docker-build:
    - Multi-stage build with cargo-chef (server)
    - Multi-stage build with cargo-chef (agent)
    - Push images to container registry with git SHA tag
    - Update Helm values with new image tags
```

### 10.2 ArgoCD (CD)

ArgoCD watches the `helm/` directory for changes. When CI updates the image tag in Helm values, ArgoCD detects the drift and automatically syncs the deployment. The flow:

1. Developer merges PR
2. GHA builds images, pushes to registry, commits updated Helm values
3. ArgoCD detects Helm values change
4. ArgoCD applies the Helm template, rolling out new pods
5. New pods run embedded migrations at startup before serving traffic

### 10.3 Docker Build (cargo-chef)

```dockerfile
# Stage 1: Chef prepare
FROM rust:1.78 AS chef
RUN cargo install cargo-chef
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cook dependencies (cached layer)
FROM rust:1.78 AS cook
RUN cargo install cargo-chef
WORKDIR /app
COPY --from=chef /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: Build application
FROM rust:1.78 AS build
WORKDIR /app
COPY --from=cook /app/target target
COPY . .
RUN cargo build --release --bin server

# Stage 4: Runtime
FROM debian:bookworm-slim
COPY --from=build /app/target/release/server /usr/local/bin/
CMD ["server"]
```

Dependencies (stage 2) are cached unless `Cargo.lock` changes. Only application code changes trigger a full rebuild of stage 3.

---

## 11. Testing Strategy

### 11.1 Backend Tests

**Unit tests** — test service layer business logic with mocked repositories:

- Story state machine transitions (todo → in_progress → done)
- Rank recalculation on reorder
- Q&A round status transitions (active → superseded on rollback)
- Dependency DAG cycle detection
- JWT validation and claims extraction

**Integration tests** — test full stack against a real Postgres instance:

- Docker Postgres spun up in CI via GitHub Actions service container
- Each test runs in a transaction that is rolled back after completion
- Tests cover: CRUD operations, RLS enforcement, soft delete filtering, Q&A flow end-to-end

### 11.2 Frontend Tests

**Vitest** — component and hook tests:

- TanStack Query hooks with MSW (Mock Service Worker) for API mocking
- Zustand store behavior
- Component rendering with various states (loading, error, empty, populated)
- Q&A thread interaction (answer selection, rollback, course correction)

**Playwright** — E2E tests for critical flows:

- OAuth login flow (mocked provider)
- Create story → answer grooming Q&A → verify decision trail
- Drag-and-drop story reorder → verify rank persistence
- Task drill-in → answer question → verify AI status change

### 11.3 Agent Tests

**Integration tests with mocked external services:**

- Mock WebSocket server (simulates the real server)
- Mock Anthropic API responses (predefined question batches)
- Mock Claude Code CLI output (predefined stdout patterns)
- Test the agent's state machine: message routing, pause/resume flow, task lifecycle

### 11.4 API Contract Enforcement

CI step regenerates OpenAPI spec from Rust types and TypeScript client from the spec. If generated files differ from committed files, CI fails. This prevents frontend/backend drift without runtime overhead.

---

## 12. Key Design Decisions Log

| # | Area | Decision | Rationale |
|---|------|----------|-----------|
| 1 | Repo | Monorepo | Shared Rust types, atomic commits across boundaries |
| 2 | Hosting | Existing k3s cluster | Infrastructure already in place, GitOps workflow established |
| 3 | Server↔Container | WebSocket (direct) | Simplest bidirectional communication, auto-reconnect on container side |
| 4 | Auth | OAuth2 only | No password infrastructure, developer-focused audience |
| 5 | Multi-tenancy | Shared DB + org_id | Standard SaaS pattern, RLS as defense-in-depth |
| 6 | Migrations | Embedded at startup | No external tooling, atomic with deployment |
| 7 | IDs | UUIDv7 | Time-ordered for index performance, natural chronological sort |
| 8 | Browser push | SSE | Unidirectional, simple, works through proxies |
| 9 | Q&A storage | Relational metadata + JSONB content | Queryable structure with flexible content |
| 10 | Deletion | Soft delete | Audit trail, recoverable, consistent pattern |
| 11 | Container auth | API key (bcrypt hashed) | Simple, revocable, upgrade path to mTLS |
| 12 | Agent language | Rust | Code sharing via shared crate, consistent toolchain |
| 13 | Claude Code | CLI subprocess with session flags | Session persistence built-in, pause = kill + resume |
| 14 | Container image | Single image, MODE flag | Simpler CI, one artifact to manage |
| 15 | Session tracking | Server-controlled session ID on task | Server is source of truth, agent is stateless |
| 16 | Frontend state | TanStack Query + Zustand | Clean server/UI state separation |
| 17 | SSE integration | Global SSE + Query invalidation | SSE as signal, Query re-fetches — single data path |
| 18 | Routing | TanStack Router | Type-safe, URL reflects full state |
| 19 | DnD | @dnd-kit | Modern, accessible, composable |
| 20 | Components | Hybrid (ui/ + features/) | Shared primitives + domain encapsulation |
| 21 | API style | REST flat URLs | Simple, well-tooled, OpenAPI compatible |
| 22 | Backend structure | Domain modules + layered internals | Discoverable by domain, separated by concern |
| 23 | Errors | Domain thiserror + unified ApiError | Type-safe error paths, clean HTTP mapping |
| 24 | SQL | sqlx::query! compile-time | Maximum safety, errors caught at build time |
| 25 | Shared types | Cargo workspace shared crate | Single source of truth for message schemas |
| 26 | API contract | utoipa → OpenAPI → orval → TS hooks | Full type pipeline, CI-enforced sync |
| 27 | WS format | JSON envelope | Debuggable, serde + shared types |
| 28 | Reconnection | Heartbeat + jitter reconnect | Fast recovery, no thundering herd |
| 29 | Connection registry | DashMap (trait-backed) | Fast concurrent access, swappable to Redis |
| 30 | SSE design | Single connection per user | All org events, frontend filters |
| 31 | Delivery | At-most-once + re-fetch | Answers persisted before forwarding, no data loss risk |
| 32 | Org scoping | Middleware + Postgres RLS | Belt and suspenders |
| 33 | Sessions | JWT in HTTP-only cookie | Stateless, no session store |
| 34 | Rate limiting | Nginx ingress only | Sufficient for early stage |
| 35 | CORS | Same origin prod, Vite proxy dev | No CORS complexity in production |
| 36 | Workspace | Nested backend/ directory | Clean Rust/non-Rust separation |
| 37 | CI/CD | GHA → ArgoCD | CI builds + pushes, ArgoCD deploys via GitOps |
| 38 | Docker | cargo-chef multi-stage | Cached dependency layer, fast rebuilds |
| 39 | Helm | App chart only | DB managed in separate infra repo |
| 40 | Secrets | External Secrets Operator | OpenBao → k8s Secrets, no sidecars |
| 41 | User↔Org | Many-to-many with roles | Multi-org support from day one |
| 42 | Projects | Explicit entity | Data model ready for multi-project |
| 43 | Task ordering | Position integer | Simple, works with dependency DAG |
| 44 | Task deps | Separate edges table | DAG enables parallel execution |
| 45 | Story priority | Fractional indexing (lexorank) | O(1) reorder, no bulk updates |
| 46 | Claude sessions | Column on task | Simple, server-controlled |
| 47 | LLM execution | Container-side only | Customer code never leaves their infra |
| 48 | QA JSONB | Relational columns + JSONB content | Metadata queryable, content flexible |
| 49 | Rollback | Status flag (active/superseded) | Simple, preserves full audit history |
| 50 | Prompts | Templates in code + KB injection | Versioned structure, dynamic context |
| 51 | Convergence | LLM self-assessment + user override | AI decides, human can override |
| 52 | Agent arch | Actor model (tokio + mpsc) | Isolated state, clean communication |
| 53 | Claude Code comms | CLI with session flags | Kill to pause, resume by session ID |
| 54 | Decision detection | Output parsing (upgrade path to MCP) | Start simple, migrate if fragile |
| 55 | Git | gitoxide | Rust-native, no subprocess |
| 56 | Setup UI | Embedded Axum + bundled SPA | Single binary, no extra process |
| 57 | OpenAPI client | orval | Direct TanStack Query hook generation |
| 58 | SSE client | Native EventSource | Auto-reconnect built-in, no deps |
| 59 | Modal | Custom-built | Full control over dismiss behavior |
| 60 | Thread rendering | Simple list + scrollIntoView | Sufficient for expected thread sizes |
| 61 | Lexorank | fractional-indexing npm | Battle-tested algorithm, small package |
| 62 | Backend tests | Unit + integration (real Postgres) | Service logic tested + full stack validated |
| 63 | Frontend tests | Vitest + Playwright | Component coverage + critical E2E flows |
| 64 | Contract tests | CI regenerate + diff check | Zero-cost sync enforcement |
| 65 | Agent tests | Integration with mocked services | State machine validated end-to-end |