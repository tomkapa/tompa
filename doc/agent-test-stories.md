# Agent Test Stories

## Purpose

This document defines 7 test stories for validating the consistency and quality of the AI agent's intelligence pipeline: **Grooming Q&A → Planning Q&A → Decomposition → Implementation**. Each story is run against the test fixture project at `test-fixtures/taskly/` — a minimal Express/TypeScript REST API.

By running the same stories repeatedly, we can measure whether the agent:
- Asks the right clarifying questions (grooming & planning)
- Decomposes work into a reasonable number of implementation tasks
- Produces correct, minimal code changes

## Key Design Principle

**Anchor the WHAT, leave the HOW open.** Each story specifies concrete files, data shapes, and expected outcomes, but intentionally omits implementation details (libraries, patterns, architecture choices). This forces the agent to surface those decisions during Q&A rather than assuming defaults.

## Story Overview

| # | Title | Type | Summary |
|---|-------|------|---------|
| 1 | Add rate limiting to the tasks API | Feature | Sliding-window rate limiter on write endpoints only |
| 2 | Fix incorrect task ID collision on concurrent creates | Bug | Non-atomic ID generator produces duplicates |
| 3 | Add input validation for task creation | Feature | Schema validation on POST /api/tasks request body |
| 4 | Refactor task service to use dependency injection | Refactor | Constructor injection for testability and future DB swap |
| 5 | Add search endpoint for tasks with filtering | Feature | Filtered, paginated GET /api/tasks/search |
| 6 | Fix task status allowing invalid transitions | Bug | Enforce valid state machine transitions on PUT |
| 7 | Add structured JSON request logging | Feature | Replace console.log with structured JSON log entries |

---

## Story 1: Add rate limiting to the tasks API

**Type:** Feature

### Description

Add rate limiting to `POST /api/tasks` and `PUT /api/tasks/:id` to prevent abuse. The rate limiter should track requests per client IP using an in-memory sliding window. Apply it only to write endpoints in `src/routes/tasks.ts`, not read endpoints.

### Deliberate Q&A Gaps

The description intentionally omits:
- Window size (e.g., 1 minute? 15 minutes?)
- Maximum requests per window
- Response format when rate limited (status code, body shape)
- Whether to use a library or hand-roll the implementation
- Middleware placement (per-route vs router-level with path filtering)

### Expected Q&A Themes

Across runs, the agent should consistently surface questions about:
- Rate limit thresholds (window duration and request count)
- HTTP 429 response body format
- Whether the sliding window should be exact or bucketed
- Client identification strategy (IP, API key, etc.)

### Expected Decomposition

~3 tasks:
1. Create rate limiting middleware (new file in `src/middleware/`)
2. Wire middleware into POST and PUT routes in `src/routes/tasks.ts`
3. Write tests covering limit enforcement and window expiry

### Verification Criteria

- A new middleware file exists under `src/middleware/`
- Only `POST /api/tasks` and `PUT /api/tasks/:id` routes use the rate limiter
- `GET` routes are unaffected
- Tests cover: requests under limit succeed, requests over limit return 429, window expiry resets the counter

---

## Story 2: Fix incorrect task ID collision on concurrent creates

**Type:** Bug (skips grooming Q&A)

### Description

When multiple `POST /api/tasks` requests arrive simultaneously, the `idGenerator` in `src/utils/idGenerator.ts` sometimes returns duplicate IDs because it uses a non-atomic read-increment-return pattern. Tasks are silently overwritten in the in-memory store. Fix the ID generator to guarantee unique IDs under concurrent access.

### Deliberate Q&A Gaps

Bug stories skip grooming. During planning, the agent may ask about:
- UUID vs atomic counter approach
- Whether existing IDs should be migrated
- Error handling for duplicate detection

### Expected Q&A Themes

- Root cause analysis of the race condition
- Choice of uniqueness strategy (UUID, timestamp-based, closure-scoped counter)
- Whether to add a defensive duplicate check in the store layer

### Expected Decomposition

~2 tasks:
1. Fix `src/utils/idGenerator.ts` to use an atomic/UUID approach
2. Add a duplicate guard in the task store or service layer

### Verification Criteria

- `idGenerator.ts` no longer uses the non-atomic read-increment-return pattern
- IDs are guaranteed unique (UUID or equivalent)
- 2–3 file changes maximum
- Existing tests still pass

---

## Story 3: Add input validation for task creation

**Type:** Feature

### Description

Add request body validation for `POST /api/tasks` in `src/routes/tasks.ts`. A new task requires `title` (non-empty string, max 200 chars) and `status` (one of 'todo', 'in_progress', 'done'). Optional fields: `description` (string, max 2000 chars) and `assignee_id` (positive integer). Invalid requests should be rejected before reaching the service layer.

### Deliberate Q&A Gaps

The description intentionally omits:
- Validation library vs hand-rolled validation
- Error response format (single error string vs per-field error array)
- Whether to sanitize inputs for XSS
- Where the schema definition should live (inline, separate file, shared types)

### Expected Q&A Themes

- Validation approach: library (zod, joi, express-validator) vs manual checks
- Error response shape: `{ errors: [{ field, message }] }` vs `{ error: string }`
- Input sanitization beyond type/length checks
- Schema file organization

### Expected Decomposition

~4 tasks:
1. Create validation schemas/functions for task creation
2. Add validation middleware or route-level validation to `POST /api/tasks`
3. Update error handler in `src/middleware/errorHandler.ts` to format validation errors
4. Write tests for valid and invalid payloads

### Verification Criteria

- Invalid payloads return 400 with field-level error details
- Valid payloads pass through to the service layer unchanged
- `title` is validated as non-empty, max 200 chars
- `status` is validated against the enum `['todo', 'in_progress', 'done']`
- Optional fields are validated when present
- Validation runs before the service layer is invoked

---

## Story 4: Refactor task service to use dependency injection

**Type:** Refactor

### Description

Refactor `src/services/taskService.ts` to accept its data store dependency via constructor injection instead of importing `src/models/task.ts` directly. This will make the service testable with mock stores and prepare for the planned SQLite migration. The public API of the service (function signatures and return types) must not change.

### Deliberate Q&A Gaps

The description intentionally omits:
- Class-based vs factory function approach
- Interface shape for the data store abstraction
- Whether to use a DI container or manual wiring
- How to handle the transition in existing consuming code

### Expected Q&A Themes

- Class with constructor vs factory function with closure
- Store interface design (which methods, naming conventions)
- DI container (tsyringe, inversify) vs manual wiring in entry point
- Migration strategy for existing code that imports taskService

### Expected Decomposition

~4 tasks:
1. Define a store interface (e.g., `TaskStore`) in a types or interfaces file
2. Refactor `taskService.ts` to accept the store via constructor/factory parameter
3. Update `src/routes/tasks.ts` and `src/index.ts` to wire the concrete store
4. Update tests to use mock store implementations

### Verification Criteria

- `taskService.ts` no longer has a direct import of `src/models/task.ts`
- A store interface/type exists
- The public API (function signatures, return types) is unchanged
- Routes and entry point wire the concrete store into the service
- Tests demonstrate mock store injection

---

## Story 5: Add search endpoint for tasks with filtering

**Type:** Feature

### Description

Add a `GET /api/tasks/search` endpoint in `src/routes/tasks.ts` that supports filtering tasks by `status` and searching task `title` by substring match. The endpoint should accept query parameters and return a paginated response. Integrate with the existing `taskService` in `src/services/taskService.ts`.

### Deliberate Q&A Gaps

The description intentionally omits:
- Pagination strategy (offset-based vs cursor-based)
- Response envelope shape (e.g., `{ data, total, page }` vs `{ items, next_cursor }`)
- Case sensitivity of title search
- Default and maximum page size
- Sort order

### Expected Q&A Themes

- Pagination approach and response metadata
- Case-insensitive vs case-sensitive title matching
- Default page size and maximum allowed page size
- Sort order (creation time, alphabetical, relevance)
- Whether to combine status filter and title search or keep them independent

### Expected Decomposition

~4 tasks:
1. Define the paginated response shape (type/interface)
2. Add a search/filter method to `taskService.ts`
3. Add the `GET /api/tasks/search` route in `src/routes/tasks.ts`
4. Write tests covering filtering, search, pagination, and edge cases

### Verification Criteria

- `GET /api/tasks/search` endpoint exists and responds
- `status` query parameter filters by task status
- `title` or `q` query parameter searches by substring match
- Response includes pagination metadata (total count, page info)
- Empty results return an empty array, not an error

---

## Story 6: Fix task status allowing invalid transitions

**Type:** Bug (skips grooming Q&A)

### Description

The `PUT /api/tasks/:id` endpoint in `src/routes/tasks.ts` allows any status value to be set, including transitions that should be invalid (e.g., 'done' directly to 'todo'). Add status transition validation to `src/services/taskService.ts` that enforces: todo->in_progress, in_progress->done, in_progress->todo. Reject invalid transitions with a 400 error.

### Deliberate Q&A Gaps

Bug stories skip grooming. During planning, the agent may ask about:
- Whether the transition map is exhaustive or if more transitions should be allowed
- Error message format for invalid transitions
- Whether to log invalid transition attempts

### Expected Q&A Themes

- Confirmation of the valid transition set (is `done->todo` really invalid?)
- Whether `done->in_progress` should be allowed (reopening)
- Error response body shape for invalid transitions
- Where the transition map should be defined (service vs separate config)

### Expected Decomposition

~3 tasks:
1. Add a status transition validation function to `src/services/taskService.ts`
2. Return a structured 400 error for invalid transitions from the update endpoint
3. Write tests covering each valid and invalid transition

### Verification Criteria

- `todo -> in_progress` succeeds
- `in_progress -> done` succeeds
- `in_progress -> todo` succeeds
- `done -> todo` returns 400
- `todo -> done` returns 400
- `done -> in_progress` returns 400 (unless agent asks and is told to allow it)
- 2–4 file changes maximum

---

## Story 7: Add structured JSON request logging

**Type:** Feature

### Description

Replace the existing `console.log` calls in `src/middleware/logger.ts` with structured JSON logging. Each log entry should include: timestamp, HTTP method, URL path, status code, response time in milliseconds, and request ID (generated per request via UUID). The request ID should be set as an `X-Request-Id` response header for traceability.

### Deliberate Q&A Gaps

The description intentionally omits:
- Logging library choice (winston, pino, bunyan, or raw JSON.stringify)
- UUID generation method (crypto.randomUUID, uuid package, nanoid)
- How to propagate request ID to downstream services or other middleware
- Log levels (should all requests be "info"? errors "error"?)
- Whether to log request/response bodies and how to redact sensitive fields

### Expected Q&A Themes

- Logging library vs manual JSON construction
- UUID generation approach
- Log levels and when to use each
- Request/response body logging and PII/sensitive field redaction
- Request ID propagation to other middleware or services
- Log output destination (stdout, file, external service)

### Expected Decomposition

~3 tasks:
1. Add request ID generation (UUID) and set `X-Request-Id` response header
2. Rewrite `src/middleware/logger.ts` to output structured JSON log entries
3. Write tests verifying JSON format, required fields, and header presence

### Verification Criteria

- `src/middleware/logger.ts` outputs valid JSON (one object per line)
- Each log entry contains: `timestamp`, `method`, `path`, `statusCode`, `responseTimeMs`, `requestId`
- `X-Request-Id` response header is present on all responses
- Request ID is a valid UUID
- No more raw `console.log` string output in the logger middleware
