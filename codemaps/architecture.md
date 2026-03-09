# Architecture Codemap
_Updated: 2026-03-07_

## Overview
Tompa is an AI-assisted agile project management tool. Stories are groomed, planned, decomposed, and implemented by a Claude Code agent that runs inside isolated containers.

## Monorepo Layout
```
tompa/
├── backend/
│   ├── server/          # Axum web API (Rust)
│   ├── agent/           # Container agent (Rust)
│   └── shared/          # Common types/enums/messages (Rust)
├── frontend/            # React SPA (TypeScript/Vite)
├── doc/                 # spec.md, tech.md, atomic_design.md
├── helm/                # Kubernetes deployment charts
└── docker-compose.yml
```

## System Components

```
Browser (React SPA)
    │  HTTP (REST + SSE)
    ▼
Axum Server  ──WS──►  Agent Container (claude_code)
    │                      │
    ▼                      ▼
PostgreSQL (RLS)       git repo / worktree
```

## Data Flow: Story Pipeline
1. User creates story → POST /api/v1/stories
2. Server starts agent session → WS sends `ServerToContainer::Execute`
3. Agent runs Q&A rounds → sends questions back via WS
4. User answers in UI → answers forwarded to agent
5. Agent grooms/plans/decomposes → tasks stored in DB
6. Agent implements tasks → git branch per story, worktree per task
7. SSE broadcaster pushes real-time updates to frontend

## Auth Flow
- OAuth (Google/GitHub) → cookie JWT (`session=` cookie)
- `AuthClaims { sub, org_id, role, exp, iat }`
- `AuthContext { user_id, org_id, role }` injected by `require_auth` middleware
- RLS enforced via `set_org_context(&mut tx, org_id)` (PostgreSQL `SET LOCAL`)

## Key Cross-Cutting Concerns
- **IDs**: All PKs are UUIDv7 via `db::new_id()`
- **RLS**: Every repo query runs inside an org-scoped transaction
- **SSE**: `SseBroadcaster` pushes story/task state changes to subscribed clients
- **OpenAPI**: Generated from utoipa annotations; frontend types auto-generated via Orval
