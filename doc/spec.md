# AI-Integrated Development Pipeline — Product Feature Specification

> **Philosophy:** "Developer owns every decision — AI owns the execution"
> **Not vibe-coding. Not manual coding. Structured ownership at AI speed.**

---

## 1. Product Overview

### 1.1 What This Product Is

A custom web application that deeply integrates AI into the software development lifecycle. It replaces traditional project management tools (Jira, Linear) with a native, purpose-built interface where AI-driven workflows are first-class citizens — not bolted-on features.

The core loop: humans make decisions by selecting from AI-generated options. AI executes based on those decisions. AI never guesses — it always asks.

### 1.2 What This Product Is NOT

- Not a Jira/Linear clone with AI features added
- Not a "vibe-coding" tool where AI generates code autonomously
- Not a chatbot interface for development — interactions are structured Q&A with predefined answers
- Not a replacement for code review on GitHub/GitLab — it augments review with decision traceability

### 1.3 Core Value Proposition (Three Equal Pillars)

**Speed:** AI pipeline runs autonomously, pauses only when human input is needed. No context-switching between PM tool and AI assistant.

**Quality:** Every decision point is explicitly answered before code is written. AI never guesses on ambiguous requirements.

**Transparency:** Every line of code traces to a tracked decision. Reviewers see the full decision trail alongside the diff.

### 1.4 Target User

- Startup teams (1–5 people initially)
- CTO / Tech Lead is the buyer
- Each team member has a Claude Code license
- Teams building web apps, mobile apps, APIs, and internal tools

---

## 2. Work Hierarchy

### 2.1 Story (Parent Unit)

- Represents a feature or user story
- Goes through **story-level pipeline**: Backlog → Grooming → Planning → Task Decomposition
- Owns a single **Merge Request** on GitHub/GitLab
- Owns a dedicated **git worktree** (enables parallel story implementation)

### 2.2 Task (Atomic Unit)

- Represents the smallest unit AI can implement in one session
- Target size: 3–5 story points (~10–15 file changes max)
- Created **automatically by AI** after story-level grooming and planning
- Human can **merge, split, or reorder** AI-proposed tasks before execution begins
- Goes through **task-level pipeline**: Deep-dive Technical Q&A → Implementation → Commit
- Each task results in a **single commit** on the story's branch
- Decisions from completed tasks **carry forward** to inform subsequent task Q&A (avoids re-asking same patterns)
- **Tasks never auto-complete** — AI finishes work, notifies human, human can adjust then marks "Done"

### 2.3 Task Types

AI automatically determines which task types are needed based on story requirements. Three types in v1:

- **🎨 Design:** AI generates wireframe/mockup descriptions. Human refines in external tools (Figma, etc.). Q&A focuses on UX decisions, layout, component interactions.
- **✅ Test Planning:** AI asks Q&A about what to test, then generates test case checklists. Actual test code writing is included in implementation tasks.
- **⚡ Code Implementation:** AI writes code via Claude Code in the container agent. This is the only task type that performs code generation, but all types route through the container agent for consistency (agent uses lighter execution for non-code tasks).

**For feature stories:** AI proposes a mix of design, test, and code tasks based on story scope.
**For bug stories:** Only code implementation tasks — no design, no test planning.

AI determines safe parallelism between task types based on dependency analysis (e.g., design tasks and test planning can run in parallel, code tasks may depend on design completion).

### 2.4 Hierarchy Summary

```
Story (feature/user story)
├── Story-level Grooming Q&A (multi-role requirement decisions)
├── Story-level Planning Q&A (technical-only decisions)
├── AI-proposed Task Decomposition (human reviews/adjusts)
├── 🎨 Design Task (wireframe/mockup description)
│   ├── Design Q&A (UX decisions)
│   ├── AI generates wireframe description
│   └── Human refines externally, marks Done
├── ✅ Test Task (test case checklist)
│   ├── Test Q&A (what to test)
│   ├── AI generates test case checklist
│   └── Human validates, marks Done
├── ⚡ Code Task 1 (atomic implementation)
│   ├── Task-level Technical Q&A
│   ├── Implementation (AI codes, pauses on decisions)
│   ├── Human reviews, marks Done → commit to story branch
├── ⚡ Code Task 2 (informed by Task 1 decisions)
│   └── ...
├── Test Suite Run (pass/fail before MR creation)
└── Merge Request (auto-generated, all task commits)
```

### 2.5 Story Types

- **Feature:** Full pipeline — grooming (multi-role) → planning → task decomposition (design + test + code) → implementation → MR
- **Bug:** Shortened pipeline — skips grooming, goes straight to technical Q&A → implementation tasks only (no design, no test planning) → MR
- **Refactor:** Same pipeline as feature but AI adjusts question focus toward codebase structure and technical debt

---

## 3. Pipeline Stages

### 3.1 Story Creation

**Creation Flow:**
1. Human clicks "+ New" → modal opens with: title, description (brief), owner, story type (feature/bug/refactor)
2. Human writes 1–2 sentence brief as description
3. AI expands brief into a structured description using knowledge base + codebase context
4. Human reviews, edits, and approves the expanded description
5. Story is created in "To Do" status

**Pipeline Trigger:** Manual — human moves story from "To Do" to "In Progress" when ready. AI pipeline starts only after this explicit action.

### 3.2 Backlog Prioritization

| Attribute | Detail |
|-----------|--------|
| **Level** | Story |
| **Human action** | Rerank stories by dragging rows in the table |
| **AI action** | Monitors status changes |
| **Trigger** | Human moves story to "In Progress" |
| **Output** | Story enters AI pipeline |

No AI questions at this stage. Row order = priority. Human drives prioritization entirely.

### 3.3 Requirement Grooming

| Attribute | Detail |
|-----------|--------|
| **Level** | Story (feature and refactor only — bugs skip this stage) |
| **Human action** | Select answers to AI-generated requirement questions across multiple domains |
| **AI action** | Generates contextual questions from multiple role perspectives with predefined answers |
| **Trigger** | Story status → "In Progress" (first stage for features) |
| **Output** | Structured requirement document attached to story |

**Multi-Role Q&A:**
Grooming generates questions from multiple domain perspectives. For solo devs, one person answers all domains. For teams, questions are tagged by domain and anyone with relevant expertise can answer.

Domain perspectives include:
- **Business/BA:** Use case scope, user impact, success criteria
- **Design/UX:** Interaction patterns, accessibility, complexity avoidance
- **Marketing:** Whether feature needs marketing, community impact, positioning
- **Development:** Technical constraints that affect business decisions
- **Security:** Data handling, authentication, compliance implications

Questions are tagged by domain (not assigned to rigid roles). Anyone on the team can answer any question.

**Q&A Behavior:**
- AI generates a batch of questions with predefined answer options
- Human answers all questions in the batch
- AI generates follow-up questions based on answers — continues until AI determines it has enough context
- "Other" option available on every question for free-form input
- AI generates as many predefined options as it considers distinct valid approaches (not a fixed count)
- All answers permanently tracked on the story

**Question Generation Inputs:**
- Story description + knowledge base (org → project → task hierarchy)
- Codebase analysis (existing patterns, APIs, conventions)
- Past decisions from similar stories
- Team member profiles/expertise (for domain tagging)

### 3.4 Implementation Planning

| Attribute | Detail |
|-----------|--------|
| **Level** | Story |
| **Human action** | Select high-level technical approach, drill into details |
| **AI action** | Proposes 2–3 approaches with tradeoffs, then drills into chosen approach |
| **Trigger** | Grooming Q&A completed → auto-transitions |
| **Output** | Technical decision document + implementation plan |

**Stage-specific question logic:** Planning asks technical-only questions (architecture, database, API design, error handling). No business/marketing/design questions — those were handled in grooming.

**Interaction Model:**
1. AI presents high-level options with clear tradeoffs (e.g., "New microservice vs. extend existing service vs. serverless")
2. Human selects approach
3. AI drills into detailed technical decisions for the chosen approach (database strategy, error handling, API design)
4. Each drill-down question has predefined answers + "Other" option
5. Uses hierarchical knowledge: org defaults → project overrides → story-specific choices
6. Batch delivery with follow-up rounds until AI has sufficient context

### 3.5 Task Decomposition

| Attribute | Detail |
|-----------|--------|
| **Level** | Story → Task creation |
| **Human action** | Review AI-proposed task split; merge, split, or reorder tasks |
| **AI action** | Proposes task breakdown based on grooming + planning decisions |
| **Trigger** | Planning Q&A completed → auto-generates task proposals |
| **Output** | Ordered list of tasks with dependency relationships |

**Behavior:**
- AI analyzes the story requirements and selected architecture to propose atomic tasks
- Each proposed task targets 3–5 story points / 10–15 file changes
- AI identifies dependencies between tasks (Task 2 depends on Task 1's API, etc.)
- Human reviews and can: merge small tasks together, split large tasks, reorder execution priority
- Once human confirms task split, deep-dive Q&A begins per task

### 3.6 Task-Level Technical Q&A

| Attribute | Detail |
|-----------|--------|
| **Level** | Task |
| **Human action** | Answer implementation-specific technical decisions |
| **AI action** | Generates task-scoped technical questions based on story decisions + knowledge + previous task decisions |
| **Trigger** | Task is next in execution order (dependencies satisfied) |
| **Output** | Technical decision set for this specific task |

**Stage-specific question logic:** Task Q&A asks implementation-only questions. No requirement or architecture questions — those were handled at story level.

**Key Behaviors:**
- Questions are strictly technical/implementation decisions — requirement questions were handled at story level
- Decisions from completed sibling tasks carry forward (e.g., if Task 1 chose useState for state management, AI won't re-ask for Task 2 unless context differs)
- Predefined answers + "Other" option
- Multi-round conversational follow-ups

**Example Questions:**
- "State management approach for this component?" → [Local state | Context API | Redux/Zustand | URL params]
- "Found overlapping utility. Reuse or create new?" → [Extend existing | Create new | Refactor existing to be generic]

### 3.7 Implementation

| Attribute | Detail |
|-----------|--------|
| **Level** | Task |
| **Human action** | Answer decision-point questions when AI pauses |
| **AI action** | Claude Code generates code, PAUSES at every unanswered decision |
| **Trigger** | Task-level Q&A completed → container agent starts coding |
| **Output** | Single commit on story branch |

**Core Principle: AI NEVER guesses.**

When encountering a decision not covered by any prior Q&A:
1. Container agent **pauses** code generation
2. Sends a new question with predefined options to the web app (real-time)
3. Owner receives push notification
4. Owner answers directly in the task UI
5. AI resumes only after receiving the answer

**Pause Behavior:**
- Paused task blocks all dependent tasks
- Independent tasks in the same story continue execution
- No timeout — AI waits as long as needed

**Error Correction:**
- If human realizes a previous answer was wrong, they can change it
- AI re-implements the task from scratch with corrected decisions

### 3.8 Human Sign-Off

After AI completes any task (design, test, or code):
1. AI finishes work and auto-notifies the human
2. Human reviews the output (wireframe description, test checklist, or code diff)
3. Human explicitly marks the task as **"Done"** — this is the only sign-off action available in the web app
4. **Tasks never auto-complete** — AI cannot advance a task to Done status
5. If the human finds issues with the code, corrections and change requests are handled **on GitHub/GitLab** (manual fixes, MR comments, etc.) — the web app's role ends at "Mark Done"

This keeps the web app focused on decision ownership while leveraging existing code review workflows for code corrections.

### 3.9 Test & MR Creation

| Attribute | Detail |
|-----------|--------|
| **Level** | Story |
| **Human action** | Review auto-generated MR description before submitting |
| **AI action** | Runs existing test suite, auto-generates MR with decision trail |
| **Trigger** | All tasks in story completed → test suite runs |
| **Output** | Merge Request on GitHub/GitLab with full traceability |

**MR Flow:**
1. All task commits land on story's branch (one MR per story)
2. AI runs existing test suite, reports pass/fail
3. If tests pass: AI auto-generates MR description from the full decision trail
4. Human reviews the MR description before it's submitted
5. MR description includes links back to the web app's decision log for each task
6. Code review happens on GitHub/GitLab — reviewers can click through to decision trail

---

## 4. Knowledge System

### 4.1 Hierarchical Override Model

```
Organization Level (lowest priority)
  └── Default tech stack, coding conventions, security requirements, ADRs
Project Level (overrides org)
  └── Project-specific patterns, API guidelines, third-party integration docs, design system specs
Task Level (highest priority)
  └── Grooming answers, planning decisions, implementation Q&A, inline knowledge additions
```

Lower-level decisions override higher-level defaults. Example: org default is "PostgreSQL for all databases" but project-level override says "use Redis for caching layer."

### 4.2 Knowledge Sources

AI ingests all available knowledge to generate better questions and predefined answers:

- **Codebase documentation:** README files, architecture decision records (ADRs)
- **Code patterns:** Detected automatically from existing repository conventions
- **External docs:** API references, library documentation
- **Design system specs:** Component and style specifications
- **Custom docs:** Team-uploaded knowledge files
- **Cross-task decisions:** Decisions from completed tasks within the same story

### 4.3 Knowledge Input in the UI

- **Task-specific knowledge:** Added inline within the task detail view (e.g., "this endpoint must support pagination because...")
- **Org/Project knowledge:** Managed in a separate knowledge base UI, linked to projects

---

## 5. Agent Architecture

### 5.1 Self-Hosted Container Model

Customer code never leaves customer infrastructure. Our server is purely the web app + orchestration layer. All LLM execution happens inside customer-hosted containers.

**Two container roles:**

**Project Container (customer self-hosted, 24/7):**
- One per project — expected to run continuously for team collaboration
- Has codebase access (linked to GitHub repo)
- Runs LLM with codebase context using customer's own Anthropic API key
- Responsible for: story-level Q&A generation (grooming + planning), task decomposition
- Uses different role-specific system prompts per domain (BA, Design, Marketing, Dev, Security)
- Generates question batches, processes checkpoint rollbacks, handles follow-up generation
- Includes a **lightweight setup web UI** for initial configuration (GitHub repo linking, API key, etc.)
- Connects to our server via **bidirectional WebSocket** (container initiates connection, API key authentication)

**Dev Container (per developer, runs when dev is online):**
- One per developer — runs during active development sessions
- Has codebase access via volume mount + git worktrees
- Responsible for: task-level Q&A generation + Claude Code implementation
- Handles pause/resume on unanswered decisions during implementation
- Single-owner — no collaboration needed at task level

### 5.2 Three Operating Modes

The container is a single codebase that supports three modes, configurable via the integrated setup UI or environment variables:

| Mode | When | What runs |
|------|------|-----------|
| **Project mode** | Team setup — dedicated server for collaborative Q&A | Story Q&A service + task decomposition (no Claude Code execution) |
| **Dev mode** | Team setup — each developer's machine | Task Q&A + Claude Code implementation (no story-level Q&A) |
| **Standalone mode** | Solo developer — single container does everything | Both project + dev services merged into one container |

### 5.3 Setup UI

The project container includes a lightweight web UI accessible at `localhost:<port>` for initial configuration:

**v1 Setup:**
- GitHub integration — link repository, configure read access
- Anthropic API key configuration
- Project connection — API key to authenticate with our web app server

**Future:**
- MCP server connections (Context7, Figma, etc.) for richer AI context
- Additional repository providers (GitLab, Bitbucket)

### 5.4 Responsibility Boundary

| Scope | Container | Reason |
|-------|-----------|--------|
| Story grooming Q&A | Project container | Collaborative — multiple team members, needs codebase context |
| Story planning Q&A | Project container | Collaborative — architecture decisions, needs codebase context |
| Task decomposition | Project container | Story-scoped, needs full grooming + planning context |
| Task-level Q&A | Dev container | Single owner, implementation-specific |
| Task implementation | Dev container | Needs Claude Code + local codebase access |
| Implementation pause questions | Dev container | Task-scoped, single owner answers |

### 5.5 Data Flow

```
Customer Infrastructure                          Our Infrastructure
┌─────────────────────────┐                      ┌──────────────────────┐
│  Project Container       │◄────WebSocket──────►│  Web App Server       │
│  (24/7, codebase access) │  (Q&A results,      │  (Orchestration,      │
│  - Story Q&A generation  │   task proposals,    │   UI, notifications,  │
│  - Task decomposition    │   status updates)    │   decision storage)   │
│  - LLM + Anthropic API   │                      │                       │
└─────────────────────────┘                      └──────────────────────┘
                                                          ▲
┌─────────────────────────┐                               │
│  Dev Container (Dev A)   │◄────WebSocket────────────────┘
│  - Task Q&A generation   │  (Task assignments,
│  - Claude Code execution │   Q&A results, commits)
│  - Volume-mounted repo   │
│  - Git worktrees         │
└─────────────────────────┘
         │
         ▼
┌─────────────────────────┐
│  GitHub / GitLab         │
│  (Branches, commits, MRs)│
└─────────────────────────┘
```

**Key principle:** Customer code and LLM calls stay on customer infrastructure. Our server only receives Q&A results (questions + selected answers), never raw code.

### 5.6 Codebase Interaction

- Project container: reads codebase for context during Q&A generation (patterns, APIs, conventions)
- Dev container: codebase is **volume-mounted**, each story gets a dedicated **git worktree**
- Tasks within a story work sequentially on the same worktree
- Multiple stories can execute in parallel across different worktrees

### 5.7 Smart Task Ordering

- AI determines dependency order between tasks within a story
- Independent tasks can execute in parallel (if developer's container supports concurrent Claude Code sessions)
- Dependent tasks are blocked until prerequisites complete
- If a task is paused (waiting for human answer), independent tasks continue

### 5.8 Agent Visibility

- **MVP:** Status updates only — AI status indicator in the task view shows current state (running / paused / blocked / done) with brief descriptions (e.g., "implementing file X", "running tests", "paused on question")
- **Deferred:** Live terminal-like output showing Claude Code's raw activity (file edits, test runs, reasoning). Requires advanced streaming architecture — terminal output contains customer code, so it must stream directly from the dev container to the user's browser without passing through our server. This will be revisited post-MVP.

---

## 6. Web Application Features

### 6.1 Main View: Table

- Primary interface is a **table view** (not kanban)
- Three columns: **Name** (with inline attention indicator), **Status** (To Do / In Progress / Done), **Owner**
- Row order = priority — drag rows to reorder
- Clicking a story row opens the **story detail modal** (see §6.3)
- Attention indicator: subtle pulsing orange dot appears inline right after story name when any question needs the user's input
- Done stories appear with reduced opacity
- Bug stories display a small "BUG" tag before the name

### 6.2 Status Model

Three statuses only — minimal, explicit:
- **To Do** — story created, not yet started
- **In Progress** — AI pipeline active (covers grooming, planning, building, testing internally)
- **Done** — story complete, MR merged

Moving a story from "To Do" to "In Progress" is the manual trigger that starts the AI pipeline. All pipeline substages (grooming, planning, decomposition, implementation, testing) are internal to "In Progress" and not exposed as separate statuses.

### 6.3 Story Detail Modal

Click a story row → a **large centered modal (~80% viewport)** opens, replacing the previous side panel approach. The modal provides richer information density and reduces unnecessary click-through.

**Modal Behavior:**
- Closes only via explicit **X button** or **Escape key** — no backdrop click dismissal to prevent accidental closure mid-Q&A
- If pending unanswered questions exist OR the user has typed an unsent draft in the chat input, a confirmation warning is shown before closing ("You have pending questions / unsent draft")
- If no pending items, closes immediately on X or Escape

**Breadcrumb Navigation (always visible at modal top):**
- Full hierarchy breadcrumb always present: `Project: MyApp > Story: User authentication`
- When drilling into a task: `Project: MyApp > Story: User authentication > Task: OAuth2 provider integration`
- Story name in the breadcrumb is clickable to navigate back from task view

**Two-Column Layout (40% / 60%):**

| Left Column (40%) | Right Column (60%) |
|---|---|
| **Overview** (always visible) | **Two tabs:** Q&A Thread (default) · Decision Trail |

**Left Column — Overview:**
- Story description
- Status and owner
- Task list with type icon (🎨✅⚡), name, state badge, and attention dot for tasks needing input
- Click any task → modal content replaces with task detail view (see §6.4)

**Right Column — Q&A Thread Tab (default):**
- Chat-like thread showing all story-level Q&A (grooming + planning)
- Stage selector at top — click "Grooming" or "Planning" to filter the thread
- AI posts structured question blocks with predefined answer options as selectable cards
- Assigned person selects an answer directly in the thread
- All answers accumulate as shared prompt context for subsequent AI generation
- See §6.5 for full Q&A thread interface details including chat input and checkpoint rollback

**Right Column — Decision Trail Tab:**
- Simple log format: flat list of question + selected answer pairs
- Chronologically ordered
- Grouped by stage (story grooming → story planning → task decomposition → per-task Q&A → per-task implementation decisions)
- Superseded decisions (from checkpoint rollbacks) shown **inline** with a "superseded" badge — greyed out with strikethrough, maintaining chronological position. Active decisions display at full opacity.
- Linkable — MR descriptions reference specific decisions via URL

**Mobile Responsive Layout:**
- On small screens, two-column layout collapses to a **single column with a tab bar** at the top
- Four tabs: **Overview** | **Q&A** | **Decisions**
- One tab visible at a time, tap to switch

### 6.4 Task Detail View (within Modal)

Click a task in the story overview → modal content replaces with task detail. The **two-column layout is maintained** for consistency with story view.

**Breadcrumb updates to:** `Project: MyApp > Story: User authentication > Task: OAuth2 provider integration` — click story name to navigate back.

**When navigating back to story view**, the right column restores whichever tab (Q&A Thread or Decision Trail) the user was last viewing at the story level.

**Left Column — Task Overview:**
- Task description
- Assignee
- AI status indicator: running / paused / blocked / done
- **"Mark Done" button** — appears when AI completes work. This is the only sign-off action. Code corrections and change requests are handled on GitHub/GitLab, outside the app. See §3.8.

**Right Column — Two Tabs (Q&A Thread default · Decision Trail):**

**Q&A Thread Tab:**
- Same chat thread UX as story level (see §6.5), but single-owner (no collaboration)
- Task owner answers all questions
- When agent pauses with a new question and user is scrolled up reviewing history: a **floating "New question ↓" indicator** appears — user clicks it to jump to the new question. Scroll position is never forced.

**Decision Trail Tab:**
- Task-scoped decisions only
- Same format as story-level Decision Trail (chronological, superseded decisions inline with badge)

**Mobile:** Same tab bar pattern as story view — collapses to single column with tabs.

### 6.5 Q&A Chat Thread Interface

The Q&A thread is the core interaction surface. It appears in the right column of both story and task views. It looks like a chat thread optimized for structured Q&A, with an always-available chat input for course correction.

**Predefined Q&A Flow:**
- **AI posts question blocks** with predefined answer options as selectable cards
- Each question block shows a domain tag (e.g., "Security", "Backend", "UX")
- **Assigned person selects an answer** — answer is immediately recorded, AI generates follow-ups
- "Other" option on every question expands to free-form text input
- Multi-round: after answering a batch, follow-up questions appear in the same thread
- All answers permanently tracked — active answers feed into AI prompt context
- **Reassignment:** Assigned person can reassign a question to someone else (story-level only)

**Chat Input for Course Correction:**
- An input field is **always visible** at the bottom of the Q&A thread
- **Visually subdued** — smaller than answer cards, lighter border, placeholder text: *"Course-correct the AI's approach..."*
- This signals it is secondary to predefined answer selection, not the primary interaction path
- Draft text **persists within the session** — switching between tabs preserves the draft. Navigating away from the modal clears all drafts.

**Two correction paths:**

**Path 1 — Redirect current round:**
User types in the chat input instead of selecting a predefined answer during the current Q&A round. AI acknowledges the correction and regenerates unanswered questions in the current batch. Already-answered questions in the batch are preserved.

**Path 2 — Checkpoint rollback to a past round:**
1. User hovers on a previously answered Q&A round → an **undo icon** appears on that round
2. User clicks the undo icon → all subsequent rounds are **removed from the Q&A thread** and moved to Decision Trail only (visible there with "superseded" badge)
3. The rolled-back round receives a **"Rollback point" badge**
4. Chat input auto-focuses for the user to type their correction
5. AI regenerates Q&A from that checkpoint onward with the new context

**Checkpoint Rollback Audit:**
- Superseded rounds are hidden from the Q&A Thread view to keep the active thread clean
- All superseded decisions remain fully visible in the **Decision Trail tab** with "superseded" badges for complete audit history
- The rolled-back round in the Q&A thread shows its "Rollback point" badge as a clear indicator of where the thread was rewound

### 6.6 Decision Trail

- **Simple log format:** flat list of question + selected answer pairs
- Chronologically ordered
- Grouped by stage (story grooming → story planning → task decomposition → per-task Q&A → per-task implementation decisions)
- Superseded decisions shown **inline chronologically** with "superseded" badge — greyed out, strikethrough, but maintaining their original position in the timeline
- Active decisions display at full opacity
- Linkable — MR descriptions reference specific decisions via URL

### 6.7 Notifications

- **Real-time push notifications** when container agent pauses with a new question
- Web app is the single source of truth — all interactions happen within the app
- No Slack/email integrations in initial version

### 6.8 Search

- **Full-text search** across story names, task names, Q&A content, and decision trail
- Enables finding past decisions, patterns, and context across the entire project history

---

## 7. Git Integration

### 7.1 Branch Model

- One branch per story (e.g., `story/STORY-123-user-authentication`)
- Each task produces one commit on the story branch
- Git worktrees enable parallel story branches without conflicts

### 7.2 Commit Model

- Each task commit message references the task ID and key decisions
- Commits are ordered by dependency (Task 1 commit before Task 2 if dependent)

### 7.3 MR Model

- One MR per story on GitHub/GitLab
- MR description auto-generated by AI from the full decision trail
- Human reviews and edits description before submission
- Description includes deep links to the web app for each task's decision log
- Test results included in MR description

---

## 8. AI Question Generation

### 8.1 Generation Logic

AI dynamically determines what to ask and how many questions to generate. There is no fixed question count — AI reads context and generates as many questions as needed for the given story complexity.

### 8.2 Context Inputs

AI uses all available context to generate better, more targeted questions:
- Story description + knowledge base (org → project → task hierarchy)
- Codebase analysis: scan repo for existing patterns, APIs, conventions, dependencies
- Past decisions from similar stories in the project
- Team member profiles and expertise (for domain tagging)
- Cross-task decisions: completed sibling tasks within the same story

### 8.3 Pacing Model

- **Batch delivery:** AI generates a full batch of questions at once
- Human answers all questions in the batch
- AI generates follow-up questions based on answers
- Follow-up rounds continue until AI determines it has sufficient context to proceed
- No fixed round limit — AI decides convergence

### 8.4 Answer Options

- AI generates as many predefined options as it considers distinct valid approaches (no fixed count)
- Binary questions may have 2 options; complex decisions may have 5–6
- Every question includes an "Other" option for free-form input

### 8.5 Stage-Specific Behavior

| Stage | Question Focus | Perspectives |
|-------|---------------|--------------|
| **Grooming** | Requirements, scope, user impact, business value | Multi-role: BA, Design, Marketing, Dev, Security |
| **Planning** | Architecture, database, API design, error handling | Technical only |
| **Task Q&A** | Implementation specifics for this task | Implementation only |
| **Implementation pauses** | Unexpected decisions encountered during coding | Implementation only |

---

## 9. Approval System (Deferred)

Approval gates are **deferred from initial build** to avoid overengineering. The planned design for future implementation:

- Configurable per team — teams decide which stages require reviewer approval
- Small teams (1–2): no approval gates, owner self-reviews
- Medium teams (3–5): optional per-stage approval
- Larger teams (5+): required on planning + code review minimum
- Reviewer can: approve all decisions, override specific answers, or request changes
- Override triggers a structured discussion (predefined resolution options, not free-form chat)

---

## 10. System Architecture Overview

```
Customer Infrastructure                          Our Infrastructure
┌─────────────────────────────────────┐          ┌────────────────────────────┐
│  Project Container (self-hosted)     │          │  Web App + Orchestrator     │
│  (24/7, one per project)             │◄──WSS──►│  (UI, notifications,        │
│  ┌─────────────────────────────┐    │          │   decision storage,         │
│  │  LLM Service                 │    │          │   pipeline orchestration)   │
│  │  (Anthropic API, customer    │    │          │                             │
│  │   key, multi-role prompts)   │    │          │  No customer code.          │
│  └─────────────────────────────┘    │          │  Only Q&A results.          │
│  ┌─────────────────────────────┐    │          └────────────────────────────┘
│  │  Setup UI (lightweight web)  │    │                    ▲
│  │  GitHub link, API key config │    │                    │ WSS
│  └─────────────────────────────┘    │                    │
│  ┌─────────────────────────────┐    │          ┌────────────────────────────┐
│  │  Codebase (read access)      │    │          │  Dev Container (per dev)    │
│  └─────────────────────────────┘    │          │  (runs when dev is online)  │
└─────────────────────────────────────┘          │  - Task Q&A generation      │
                                                  │  - Claude Code execution    │
                                                  │  - Volume-mounted repo      │
                                                  │  - Git worktrees per story  │
                                                  └────────────┬───────────────┘
                                                               │
                                                               ▼
                                                  ┌────────────────────────────┐
                                                  │  GitHub / GitLab            │
                                                  │  (Branches, commits, MRs)   │
                                                  └────────────────────────────┘

Standalone Mode (solo dev): Project + Dev merged into single container
```

### 10.1 Communication Flow

1. Web app → Project container (via server relay): Story status changes, Q&A answers
2. Project container → Web app (via server): Generated questions, task decomposition proposals, follow-up rounds
3. Web app → Dev container (via server relay): Task assignments, decision context
4. Dev container → Web app (via server): Task Q&A questions, pause questions, commit notifications, implementation status
5. Dev container → GitHub/GitLab: Commits, branch management, MR creation
6. **Customer code never reaches our server** — only Q&A results (questions + answers) transit through

---

## 11. Key Design Principles

1. **"Never guess, always ask"** — AI pauses and asks structured questions rather than making assumptions
2. **Predefined answers first** — Humans select from AI-generated options; free-form is the escape hatch, not the default
3. **Decision ownership** — Every line of code traces to a human decision
4. **Human sign-off required** — AI never auto-completes a task; humans review output and explicitly mark Done
5. **Permanent traceability** — All decisions are stored, linked, and reviewable forever
6. **Hierarchical knowledge** — Org → Project → Task knowledge inheritance with override capability
7. **Context-aware sizing** — Tasks are decomposed to fit LLM context windows (3–5 story points, 10–15 files max)
8. **Cross-task learning** — Decisions carry forward within a story to avoid redundant questions
9. **Parallel when safe** — Independent tasks and stories execute concurrently; dependencies block appropriately
10. **Web app is the single source of truth** — All human-AI interaction happens in the custom app
11. **AI-native UX** — The interface is designed around AI workflows, not retrofitted from traditional PM tools
12. **Minimal workflow, maximum clarity** — Three statuses (To Do / In Progress / Done), no pipeline stages exposed; Q&A state IS the status