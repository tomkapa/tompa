# Frontend Codemap
_Updated: 2026-03-07_

## Stack
- React 18 + TypeScript + Vite
- TanStack Router (file-based routing)
- TanStack Query (server state)
- Zustand (client state stores)
- Tailwind CSS + Radix UI primitives
- Orval (generates typed fetch hooks from OpenAPI spec)
- Playwright (E2E tests in `e2e/`)

## Directory Structure
```
src/
├── main.tsx              # App mount
├── App.tsx               # QueryClient + RouterProvider
├── router.tsx            # Route tree (TanStack Router)
├── index.css             # Global styles
├── test-setup.ts         # Vitest setup
│
├── api/
│   ├── custom-fetch.ts              # credentialsFetch wrapper (credentials: 'include')
│   └── generated/                   # Orval output (bun run generate-api)
│       ├── tompaAPI.schemas.ts      # All shared TS types
│       ├── auth/auth.ts
│       ├── projects/projects.ts
│       ├── stories/stories.ts
│       ├── tasks/tasks.ts
│       ├── qa/qa.ts
│       ├── orgs/orgs.ts
│       ├── knowledge/knowledge.ts
│       ├── container-keys/container-keys.ts
│       ├── decision-patterns/decision-patterns.ts
│       └── project-profiles/project-profiles.ts
│
├── components/ui/        # Atomic design system (atoms + molecules)
│   ├── button.tsx
│   ├── input.tsx / textarea.tsx / select.tsx / checkbox.tsx / switch.tsx
│   ├── card.tsx (Card, CardHeader, CardContent, CardFooter)
│   ├── dialog.tsx / tooltip.tsx / dropdown.tsx / listbox.tsx
│   ├── badge.tsx / alert.tsx / progress.tsx / avatar.tsx
│   ├── table.tsx / pagination.tsx / tabs.tsx / accordion.tsx
│   ├── radio-group.tsx / tab-switcher.tsx
│   ├── sidebar.tsx / breadcrumb.tsx / app-breadcrumb.tsx
│   ├── markdown-editor.tsx / markdown-viewer.tsx
│   ├── toast.tsx / confirmation-dialog.tsx
│   ├── status-badge.tsx        # Story/task status pill
│   ├── story-table-row.tsx     # DnD-capable story row
│   ├── story-type-tag.tsx / domain-tag.tsx
│   ├── task-list-item.tsx / task-type-icon.tsx
│   ├── ai-status-indicator.tsx / attention-dot.tsx / new-question-indicator.tsx
│   ├── answer-option-card.tsx / other-option.tsx / course-correction-input.tsx
│   ├── mark-done-button.tsx / icon-button.tsx
│   ├── rollback-badge.tsx / superseded-badge.tsx
│   ├── decision-trail-entry.tsx / pattern-indicator-badge.tsx
│   └── index.ts              # Barrel export
│
├── features/             # Organisms + pages
│   ├── auth/
│   │   ├── login-page.tsx
│   │   ├── onboarding-page.tsx
│   │   └── onboarding-storage.ts   # localStorage: onboarding complete flag
│   ├── layout/
│   │   └── app-layout.tsx          # Sidebar + main area shell
│   ├── projects/
│   │   ├── create-project-modal.tsx
│   │   └── project-selector.tsx
│   ├── stories/
│   │   ├── stories-table.tsx       # Drag-and-drop story list
│   │   ├── story-creation.tsx
│   │   ├── story-modal.tsx         # Routed modal overlay
│   │   └── story-overview.tsx      # Description + QA + tasks summary
│   ├── tasks/
│   │   ├── task-decomposition.tsx  # Task list with status
│   │   └── task-overview.tsx
│   ├── qa/
│   │   ├── qa-thread.tsx           # Q&A conversation UI
│   │   ├── question-block.tsx      # Single Q&A round
│   │   ├── assignee-avatar.tsx / member-picker-popover.tsx
│   │   ├── use-org-members.ts / use-question-assignment.ts
│   │   └── types.ts
│   ├── decisions/
│   │   └── decision-trail.tsx      # Decision history view
│   ├── patterns/
│   │   ├── patterns-page.tsx
│   │   ├── pattern-detail.tsx / pattern-filters.tsx / confidence-bar.tsx
│   │   ├── use-patterns.ts
│   │   └── types.ts
│   ├── profile/
│   │   ├── profile-page.tsx
│   │   ├── profile-section-editor.tsx
│   │   ├── use-profile.ts
│   │   └── types.ts
│   └── settings/
│       └── project-settings.tsx
│
├── hooks/
│   ├── use-auth.ts             # Current user + org state
│   ├── use-sse.ts              # SSE subscription hook
│   └── use-exit-animation.ts
│
├── stores/
│   ├── sse-store.ts    # Zustand: SSE event queue / real-time state
│   ├── toast-store.ts  # Zustand: toast notifications
│   └── ui-store.ts     # Zustand: misc UI state (sidebar, modal, etc.)
│
└── lib/
    ├── fractional-indexing.ts   # Story rank key generation
    └── utils.ts                 # cn() + misc helpers
```

## Route Tree
```
/                         → redirect to /projects/default
/login                    → LoginPage
/onboarding               → OnboardingPage
/projects/:projectSlug    → AppLayout
  /                       → StoriesTable (index)
  /settings               → ProjectSettings
  /patterns               → PatternsPage
  /profile                → ProfilePage
  /stories/:storyId       → StoryModal
    /tasks/:taskId        → TaskDetail
```

## API Generation Pipeline
```
cargo run --bin generate-openapi  →  openapi.json
orval --config orval.config.json  →  src/api/generated/**
```
Command: `bun run generate-api`

## E2E Tests
- Config: `frontend/playwright.config.ts`
- Requires: `e2e/.auth/user.json` + backend running with `DEV_MODE=true`
- Global setup seeds project + stories via API
