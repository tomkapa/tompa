# Backend Codemap
_Updated: 2026-03-07_

## Crates

### `shared` — Common types
```
src/
├── lib.rs         # pub mod enums, messages, telemetry, types
├── enums.rs       # StoryType, StoryStatus, PipelineStage, TaskType, TaskState,
│                  #   QaStage, QaRoundStatus, ContainerMode, OrgRole, KnowledgeCategory
├── messages.rs    # ServerToContainer { Execute, Ping }
│                  # ContainerToServer { ExecutionResult, ExecutionFailed, Pong }
├── types.rs       # Shared domain types
└── telemetry.rs   # init_test_tracing()
```

### `server` — Axum web API
```
src/
├── lib.rs              # build_app(AppState) -> Router; merges all module routers
├── main.rs             # thin entry point
├── state.rs            # AppState { pool, config, registry, broadcaster }
├── config.rs           # Config struct loaded from env
├── db.rs               # PgPool init + new_id() (UUIDv7)
├── errors.rs           # ApiError + From<sqlx::Error>
├── openapi.rs          # ApiDoc (utoipa) + openapi_handler
├── bin/
│   └── generate_openapi.rs   # cargo run --bin generate-openapi
│
├── auth/               # OAuth + JWT cookie auth
│   ├── handler.rs      # GET /auth/me, /auth/google, /auth/github, /auth/logout
│   ├── middleware.rs   # require_auth extractor → AuthContext
│   ├── service.rs      # token validation, user upsert (uses sqlx::query_scalar!)
│   └── types.rs        # AuthClaims, AuthContext, MeResponse
│
├── agents/             # WebSocket container management
│   ├── handler.rs      # GET /ws/container (Bearer token auth + WS upgrade)
│   ├── registry.rs     # ConnectionRegistry trait + DashMapRegistry
│   ├── service.rs      # send_start_task(story_id) → WS message dispatch
│   ├── session_repo.rs # agent_sessions DB queries
│   └── prompts/        # Prompt templates (description_refinement, grooming/*, etc.)
│
├── sse/                # Server-Sent Events
│   ├── handler.rs      # GET /api/v1/sse (auth required)
│   ├── broadcaster.rs  # SseBroadcaster: tokio broadcast channel
│   └── mod.rs
│
├── orgs/               # Organizations CRUD
├── project/            # Projects CRUD (scoped to org via RLS)
├── story/              # Stories CRUD + fractional rank
│   └── rank.rs         # Fractional indexing for drag-and-drop ordering
├── task/               # Tasks CRUD + dependencies
├── qa/                 # QA rounds (grooming Q&A)
├── knowledge/          # Knowledge base entries
├── container_keys/     # Container API key management
├── decision_patterns/  # Team decision patterns
└── project_profiles/   # Project QA configuration profiles
```

#### Module Pattern (every domain module)
```
mod.rs → handler.rs (routes + request/response) → service.rs (business logic) → repo.rs (SQL) → types.rs
```

#### Route Protection
```rust
.route_layer(axum::middleware::from_fn_with_state(state, require_auth))
```

### `agent` — Container agent
```
src/
├── main.rs          # Tokio entry; loads config, spawns actors by mode
├── config.rs        # TOML via CONFIG_PATH env; github_access_token, repo path, etc.
├── dispatcher.rs    # DispatchMessage enum; routes msgs between actors
├── ws_client.rs     # WebSocket actor: connects to server, recv/send loop
├── claude_code.rs   # ClaudeCode actor: Q&A gen, decomposition, implementation
├── git_manager.rs   # GitManager: branch per story, worktree per task, commit+push
├── agent_status.rs  # Status tracking
└── setup_ui.rs      # Config UI stub (Axum, T24)
```

#### Actor Communication
```
ws_client  ──► dispatcher ──► claude_code
                    │
                    └──────► git_manager
```

#### Mode → Actors Active
| Mode       | ws_client | claude_code | git_manager | setup_ui |
|------------|-----------|-------------|-------------|----------|
| Project    | yes       | yes         | no          | yes      |
| Dev        | yes       | yes         | yes         | no       |
| Standalone | yes       | yes         | yes         | yes      |

## Migrations (server/migrations/)
| # | Table |
|---|-------|
| 001 | extensions (uuid-ossp) |
| 002 | organizations |
| 003 | users |
| 004 | org_members |
| 005 | projects |
| 006 | stories |
| 007 | tasks |
| 008 | task_dependencies |
| 009 | qa_rounds |
| 010 | container_api_keys |
| 011 | knowledge_entries |
| 012-013 | RLS enable + force |
| 014 | unique project name per org |
| 015 | pending_refined_description on stories |
| 016 | agent_sessions |
| 017 | agent_session → round link |
| 018 | project_grooming_roles |
| 019 | project_qa_config |
| 020 | decision_patterns |
| 021 | project_profiles |

## Key Libraries
- `axum 0.8`, `tower-http`, `sqlx` (no compile-time macros except auth)
- `utoipa 4` (uuid, chrono features) for OpenAPI
- `dashmap 6` for connection registry
- `gix 0.68` (revision feature) for git branch ops in agent
- `tokio/full`, `serde`, `uuid`, `tracing`, `anyhow`
