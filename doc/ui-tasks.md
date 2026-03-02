# UI Implementation Tasks — Atomic Design Breakdown

> **Replaces:** T10 (Frontend UI Atoms) and T23 (Frontend Feature Modules) from `plan.md`
> **Rule:** Each task maps 1:1 to an atomic design step. A developer picks up a task, references the Pencil design frame, and implements the React component.
> **Design file:** Pencil MCP — `/Users/tomtran/playground/custom-agent/tompa`
> **Design system (Halo):** Frame `vtHps` — reusable shadcn-style components

---

## Dependency Graph

```
U01–U12 (atoms) ─────────────────── no deps, fully parallel
U13 ──────────────────────────────── no deps
U14 ──────────────────────────────── U13
U15 ──────────────────────────────── U01
U16 ──────────────────────────────── U03, U06
U17 ──────────────────────────────── U01, U02, U04
U18 ──────────────────────────────── U01, U02, U05
U19 ──────────────────────────────── no deps
U20 ──────────────────────────────── U03, U07, U13, U14
U21 ──────────────────────────────── U07, U11, U12, U20
U22 ──────────────────────────────── U16
U23 ──────────────────────────────── U01, U18
U24 ──────────────────────────────── U10, U15
U25 ──────────────────────────────── U17
U26 ──────────────────────────────── U05
U27 ──────────────────────────────── no deps
U28 ──────────────────────────────── U08, U09, U19, U21, U22, U23
U29 ──────────────────────────────── U08, U09, U21, U22, U24
U30 ──────────────────────────────── U25, U27, U28, U29
```

---

## Task Index

| ID  | Name                              | Layer    | Atomic Step | Depends On             |
|-----|-----------------------------------|----------|-------------|------------------------|
| U01 | Status Badge                      | Atom     | Step 1      | —                      |
| U02 | Attention Dot                     | Atom     | Step 2      | —                      |
| U03 | Domain Tag                        | Atom     | Step 3      | —                      |
| U04 | Story Type Tag                    | Atom     | Step 4      | —                      |
| U05 | Task Type Icon                    | Atom     | Step 5      | —                      |
| U06 | Superseded Badge                  | Atom     | Step 6      | —                      |
| U07 | Rollback Point Badge              | Atom     | Step 7      | —                      |
| U08 | Breadcrumb                        | Atom     | Step 8      | —                      |
| U09 | Tab Switcher                      | Atom     | Step 9      | —                      |
| U10 | Mark Done Button                  | Atom     | Step 10     | —                      |
| U11 | New Question Indicator            | Atom     | Step 11     | —                      |
| U12 | Course Correction Input           | Atom     | Step 12     | —                      |
| U13 | Answer Option Card                | Molecule | Step 13     | —                      |
| U14 | Other Option with Free-form       | Molecule | Step 14     | U13                    |
| U15 | AI Status Indicator               | Molecule | Step 15     | U01                    |
| U16 | Decision Trail Entry              | Molecule | Step 16     | U03, U06               |
| U17 | Story Table Row                   | Molecule | Step 17     | U01, U02, U04          |
| U18 | Task List Item                    | Molecule | Step 18     | U01, U02, U05          |
| U19 | Confirmation Warning Dialog       | Molecule | Step 19     | —                      |
| U20 | Question Block                    | Organism | Step 20     | U03, U07, U13, U14     |
| U21 | Q&A Thread                        | Organism | Step 21     | U07, U11, U12, U20     |
| U22 | Decision Trail                    | Organism | Step 22     | U16                    |
| U23 | Story Overview Panel              | Organism | Step 23     | U01, U18               |
| U24 | Task Overview Panel               | Organism | Step 24     | U10, U15               |
| U25 | Stories Table                     | Organism | Step 25     | U17                    |
| U26 | Task Decomposition Review         | Organism | Step 26     | U05                    |
| U27 | Story Creation Flow               | Organism | Step 27     | —                      |
| U28 | Story Detail Modal (Story View)   | Template | Step 28     | U08, U09, U19, U21, U22, U23 |
| U29 | Story Detail Modal (Task View)    | Template | Step 29     | U08, U09, U21, U22, U24 |
| U30 | Main Application Layout           | Template | Step 30     | U25, U27, U28, U29     |

---

## Layer 1: Atoms

---

### U01 — Status Badge

**Atomic Step:** Step 1
**Depends on:** Nothing
**Pencil design frame:** `gSJEb` (inside `ovAq2` Layer 1: Atoms)
**Pencil components:**
- `S4c4N` — Story Badge/To Do
- `AzCUG` — Story Badge/In Progress
- `hqzTp` — Story Badge/Done
- `V2yO5` — Task Badge/Done
- `tL18g` — Task Badge/AI Working
- `MlWtc` — Task Badge/Needs Input
- `8rwFQ` — Task Badge/Blocked

**File:** `frontend/src/components/ui/status-badge.tsx`

**Requirements:**
- Props: `type: 'story' | 'task'`, `value: string`
- Story values: `todo`, `in_progress`, `done`
- Task values: `done`, `running`, `needs_input`, `blocked`
- Each state visually distinct at a glance (match Pencil design colors/fills)
- Compact — never wraps to multiple lines

**Acceptance Criteria:**
- All 7 badge variants render matching the Pencil design
- Component is pure presentational, no API calls
- Uses Tailwind classes only

---

### U02 — Attention Dot

**Atomic Step:** Step 2
**Depends on:** Nothing
**Pencil design frame:** `j0xAI` (inside `ovAq2`)
**Pencil component:** `q4GVq` — Attention Dot

**File:** `frontend/src/components/ui/attention-dot.tsx`

**Requirements:**
- Small pulsing orange circular element with CSS keyframe animation (scale + opacity)
- Subtle urgency — noticeable in peripheral vision without being alarming
- Must not shift layout when appearing/disappearing (use `position: absolute` or reserve space)
- Used inline next to text (story name, task name)

**Acceptance Criteria:**
- Smooth pulse animation matching Pencil design
- No layout shift on show/hide
- Pure CSS animation, no JS timers

---

### U03 — Domain Tag

**Atomic Step:** Step 3
**Depends on:** Nothing
**Pencil design frame:** `NXYQB` (inside `ovAq2`)
**Pencil component:** `oF4EA` — Domain Tag

**File:** `frontend/src/components/ui/domain-tag.tsx`

**Requirements:**
- Props: `domain: string` (e.g., "Security", "Backend", "UX", "Business Logic", "Design", "Marketing")
- Visually lighter than status badges — informational, not actionable
- Compact pill shape

**Acceptance Criteria:**
- Renders with correct styling from Pencil design
- Lighter visual weight than status badges

---

### U04 — Story Type Tag

**Atomic Step:** Step 4
**Depends on:** Nothing
**Pencil design frame:** `SMdc6` (inside `ovAq2`)
**Pencil components:**
- `1a5BI` — Story Type/Bug
- `a1Cqr` — Story Type/Refactor

**File:** `frontend/src/components/ui/story-type-tag.tsx`

**Requirements:**
- Props: `type: 'feature' | 'bug' | 'refactor'`
- Only renders for bug and refactor (feature = no tag / returns null)
- "BUG" tag must be immediately noticeable (prominent, red-ish per Pencil design)
- Compact, sits inline before story name

**Acceptance Criteria:**
- Feature type renders nothing
- Bug and Refactor tags match Pencil design
- Bug tag visually prominent

---

### U05 — Task Type Icon

**Atomic Step:** Step 5
**Depends on:** Nothing
**Pencil design frame:** `WDbRO` (inside `ovAq2`)
**Pencil components:**
- `DBVIf` — Task Type Icon/Design
- `u2JT7` — Task Type Icon/Test
- `C9NdF` — Task Type Icon/Code

**File:** `frontend/src/components/ui/task-type-icon.tsx`

**Requirements:**
- Props: `type: 'design' | 'test' | 'code'`
- Fixed size, used inline in task lists
- Icons match Pencil design (reference the icon_font or emoji used in each component)

**Acceptance Criteria:**
- All 3 icons render at fixed size
- Match Pencil design appearance

---

### U06 — Superseded Badge

**Atomic Step:** Step 6
**Depends on:** Nothing
**Pencil design frame:** `xwr3D` (inside `ovAq2`)
**Pencil component:** `oIKJ1` — Superseded Badge

**File:** `frontend/src/components/ui/superseded-badge.tsx`

**Requirements:**
- Small badge text "Superseded"
- Muted styling (grey) — communicates "this decision was overridden"
- Distinguishable from active decisions without hover or click

**Acceptance Criteria:**
- Renders matching Pencil design
- Visually muted

---

### U07 — Rollback Point Badge

**Atomic Step:** Step 7
**Depends on:** Nothing
**Pencil design frame:** `B0pi2` (inside `ovAq2`)
**Pencil component:** `0zR2Q` — Rollback Point Badge

**File:** `frontend/src/components/ui/rollback-badge.tsx`

**Requirements:**
- Small badge text "Rollback point"
- Visually distinct from superseded badge (different color — amber/yellow per Pencil design)
- Marks where Q&A thread was rewound to (restoration point, not removed item)

**Acceptance Criteria:**
- Renders matching Pencil design
- Clearly different from Superseded Badge

---

### U08 — Breadcrumb

**Atomic Step:** Step 8
**Depends on:** Nothing
**Pencil design frame:** `n7nsq` (inside `ovAq2`)
**Pencil components:**
- `gqZHe` — Breadcrumb Item/Link
- `OF5sR` — Breadcrumb Separator
- `tftak` — Breadcrumb Item/Current

**File:** `frontend/src/components/ui/breadcrumb.tsx`

**Requirements:**
- Props: `segments: Array<{ label: string, onClick?: () => void }>`
- Displays path: `Project > Story > Task` (task level only when viewing a task)
- All segments except last are clickable (use `gqZHe` style)
- Last segment uses `tftak` style (non-clickable)
- Separator between segments uses `OF5sR`
- Truncates long names with ellipsis rather than wrapping

**Acceptance Criteria:**
- Clickable segments navigate correctly
- Last segment is non-clickable
- Long names truncate with ellipsis
- Matches Pencil design

---

### U09 — Tab Switcher

**Atomic Step:** Step 9
**Depends on:** Nothing
**Pencil design frame:** `pUFc4` (inside `ovAq2`)
**Pencil components:**
- `VR2ae` — Tab Switcher (container)
- `HLOgY` — Tab Switcher Item/Active
- `2xRdo` — Tab Switcher Item/Inactive

**File:** `frontend/src/components/ui/tab-switcher.tsx`

**Requirements:**
- Props: `tabs: Array<{ id: string, label: string }>`, `activeId: string`, `onChange: (id: string) => void`
- One active tab visually indicated (uses `HLOgY` style)
- Inactive tabs use `2xRdo` style
- Horizontal layout
- Used in two contexts: modal right column ("Q&A Thread" | "Decision Trail") and mobile ("Overview" | "Q&A" | "Decisions")

**Acceptance Criteria:**
- Active/inactive states match Pencil design
- Tab switching calls onChange correctly
- Renders horizontally

---

### U10 — Mark Done Button

**Atomic Step:** Step 10
**Depends on:** Nothing
**Pencil design frame:** `JCgRH` (inside `ovAq2`)
**Pencil component:** `TDRZM` — Mark Done Button

**File:** `frontend/src/components/ui/mark-done-button.tsx`

**Requirements:**
- Props: `onClick: () => void`, `loading?: boolean`
- Prominent, clearly actionable — most important action on task view
- Disabled state while loading
- Only appears when AI has completed work (visibility controlled by parent)

**Acceptance Criteria:**
- Renders matching Pencil design
- Disabled/loading state works
- Single action button

---

### U11 — New Question Floating Indicator

**Atomic Step:** Step 11
**Depends on:** Nothing
**Pencil design frame:** `M0NZo` (inside `ovAq2`)
**Pencil component:** `1OoB2` — New Question Indicator

**File:** `frontend/src/components/ui/new-question-indicator.tsx`

**Requirements:**
- Props: `onClick: () => void`, `visible: boolean`
- Floating pill: "New question ↓"
- Anchored to bottom of scrollable Q&A area, floating above content
- Hidden when `visible = false`
- Does NOT force scroll — user controls their scroll position

**Acceptance Criteria:**
- Renders as floating element matching Pencil design
- Click triggers onClick (scroll-to-bottom in parent)
- Properly hidden/shown based on `visible` prop

---

### U12 — Course Correction Chat Input

**Atomic Step:** Step 12
**Depends on:** Nothing
**Pencil design frame:** `68fZi` (inside `ovAq2`)
**Pencil components:**
- `t68pf` — Course Correction Input (empty/default state)
- `KCJ54` — Course Correction Input/Filled

**File:** `frontend/src/components/ui/course-correction-input.tsx`

**Requirements:**
- Props: `value: string`, `onChange: (v: string) => void`, `onSubmit: () => void`
- Visually subdued (lighter border, smaller) — signals secondary to predefined selection
- Placeholder: "Course-correct the AI's approach..."
- Submit on Enter (with Shift+Enter for newlines)
- Two visual states: default (`t68pf`) and filled (`KCJ54`)

**Acceptance Criteria:**
- Default and filled states match Pencil design
- Enter submits, Shift+Enter inserts newline
- Visually subdued compared to answer cards

---

## Layer 2: Molecules

---

### U13 — Answer Option Card

**Atomic Step:** Step 13
**Depends on:** Nothing
**Pencil design frame:** `XPRCB` (inside `vbJy2` Layer 2: Molecules)
**Pencil components:**
- `oqaPL` — Answer Option Card/Default
- `oPWk9` — Answer Option Card/Selected

**File:** `frontend/src/components/ui/answer-option-card.tsx`

**Requirements:**
- Props: `text: string`, `selected: boolean`, `disabled: boolean`, `onSelect: () => void`
- Radio-style selection — one answer per question
- Selecting immediately records the answer — no separate "submit"
- Selected state visually distinct (use `oPWk9` style)
- Selection is irreversible in normal flow (parent controls disabled state after selection)

**Acceptance Criteria:**
- Default and selected states match Pencil design
- Click triggers onSelect
- Disabled when already answered

---

### U14 — Other Option with Free-form Input

**Atomic Step:** Step 14
**Depends on:** U13
**Pencil design frame:** `PqIX3` (inside `vbJy2`)
**Pencil components:**
- `NUCyH` — Other Option/Collapsed
- `15Siy` — Other Option/Expanded

**File:** `frontend/src/components/ui/other-option.tsx`

**Requirements:**
- Composes Answer Option Card (U13) labeled "Other"
- Collapsed state (`NUCyH`): looks like a regular option card
- Selecting "Other" expands inline text input (`15Siy`)
- User types custom answer and submits
- Present on every question as the last option — escape hatch from predefined answers

**Acceptance Criteria:**
- Collapsed/expanded states match Pencil design
- Selecting expands text input inline
- Submit sends custom answer text

---

### U15 — AI Status Indicator

**Atomic Step:** Step 15
**Depends on:** U01 (Status Badge)
**Pencil design frame:** `DKuM4` (inside `vbJy2`)
**Pencil components:**
- `MllKk` — AI Status Indicator/Running
- `4TQNQ` — AI Status Indicator/Paused
- `ntcVV` — AI Status Indicator/Blocked
- `ktiAE` — AI Status Indicator/Done

**File:** `frontend/src/components/ui/ai-status-indicator.tsx`

**Requirements:**
- Props: `state: 'running' | 'paused' | 'blocked' | 'done'`, `statusText: string`, `blockedOn?: string`
- Composes task state badge (U01) + status description text
- Running: subtle animation suggesting ongoing activity
- Paused: clearly communicates "waiting for you" — distinct from blocked
- Blocked: shows what it's blocked on (dependency reference)
- Done: static, completed state
- Updates in real-time via SSE (parent handles data, this is presentational)

**Acceptance Criteria:**
- All 4 states match Pencil design components
- Running has animation
- Paused visually distinct from blocked

---

### U16 — Decision Trail Entry

**Atomic Step:** Step 16
**Depends on:** U03 (Domain Tag), U06 (Superseded Badge)
**Pencil design frame:** `V8TiX` (inside `vbJy2`)
**Pencil components:**
- `IQYDs` — Decision Trail Entry/Active
- `eMYx6` — Decision Trail Entry/Superseded

**File:** `frontend/src/components/ui/decision-trail-entry.tsx`

**Requirements:**
- Props: `domain: string`, `questionText: string`, `answerText: string`, `superseded: boolean`, `entryUrl?: string`
- Composes: Domain Tag (U03) + question text + answer text
- Active decisions (`IQYDs`): full prominence
- Superseded decisions (`eMYx6`): muted with strikethrough + Superseded Badge (U06)
- Compact, scannable format — optimized for reading many entries
- Each entry has a stable URL for deep linking from MR descriptions

**Acceptance Criteria:**
- Active and superseded states match Pencil design
- Superseded entries are visually muted with strikethrough
- Compact layout

---

### U17 — Story Table Row

**Atomic Step:** Step 17
**Depends on:** U01 (Status Badge), U02 (Attention Dot), U04 (Story Type Tag)
**Pencil design frame:** `OhJ79` (inside `vbJy2`)
**Pencil component:** `52lc2` — Story Table Row/Default

**File:** `frontend/src/components/ui/story-table-row.tsx`

**Requirements:**
- Props: `story: { id, title, storyType, status, ownerName, needsAttention }`, `onClick: () => void`, `dragHandleProps?: object`
- Three-column layout: Name (with Story Type Tag + Attention Dot inline), Status (Status Badge), Owner
- Composes: Story Type Tag (U04, only for bug/refactor), Attention Dot (U02, only when attention needed), Status Badge (U01, story variant)
- Clickable — opens story detail modal
- Draggable for reordering (drag handle affordance)
- Done stories at reduced visual emphasis (lower opacity)
- Row order = priority — top is highest

**Acceptance Criteria:**
- Matches Pencil design `52lc2`
- Drag handle visible
- Done stories at reduced opacity
- Click triggers onClick

---

### U18 — Task List Item

**Atomic Step:** Step 18
**Depends on:** U01 (Status Badge), U02 (Attention Dot), U05 (Task Type Icon)
**Pencil design frame:** `hE9Y8` (inside `vbJy2`)
**Pencil component:** `wEZ62` — Task List Item/Default

**File:** `frontend/src/components/ui/task-list-item.tsx`

**Requirements:**
- Props: `task: { id, name, taskType, state, needsAttention }`, `onClick: () => void`
- Composes: Task Type Icon (U05), task name, Attention Dot (U02, when unanswered question), Task State Badge (U01, task variant)
- Clickable — navigates to task detail view within modal
- AI working state: subtle pulsing animation on the badge

**Acceptance Criteria:**
- Matches Pencil design `wEZ62`
- Attention dot shown only when task has unanswered question
- Click navigates to task detail

---

### U19 — Confirmation Warning Dialog

**Atomic Step:** Step 19
**Depends on:** Nothing
**Pencil design frame:** `pyQlB` (inside `vbJy2`)
**Pencil component:** `rFtO0` — Confirmation Warning Dialog/Pending Questions

**File:** `frontend/src/components/ui/confirmation-dialog.tsx`

**Requirements:**
- Props: `open: boolean`, `onStay: () => void`, `onLeave: () => void`, `reason: 'pending_questions' | 'unsent_draft'`
- Appears when user tries to close modal AND either: pending unanswered questions OR unsent draft in chat input
- Two actions: "Stay" (cancel close) and "Leave" (confirm close and discard)
- Blocks the close action until user confirms

**Acceptance Criteria:**
- Matches Pencil design `rFtO0`
- Two distinct actions work correctly
- Blocks modal close until resolved

---

## Layer 3: Organisms

---

### U20 — Question Block

**Atomic Step:** Step 20
**Depends on:** U03 (Domain Tag), U07 (Rollback Badge), U13 (Answer Option Card), U14 (Other Option)
**Pencil design frame:** `4Sr6b` (inside `L3hG8` Layer 3: Organisms)
**Pencil components:**
- `vrZJ8` — Question Block/Default (unanswered)
- `odYRw` — Question Block/Answered
- `XwWLt` — Question Block/Hover (Undo)
- `poVzY` — Question Block/Rollback Point

**File:** `frontend/src/features/qa/question-block.tsx`

**Requirements:**
- Props: `question: QaQuestion`, `onAnswer: (questionId, answerIndex, answerText) => void`, `onRollback?: () => void`, `isRollbackPoint: boolean`, `answered: boolean`
- Composes: Domain Tag (U03), question text, list of Answer Option Cards (U13), Other Option (U14), undo icon
- 4 visual states:
  - Default (`vrZJ8`): unanswered, options selectable
  - Answered (`odYRw`): selected option locked in
  - Hover/Undo (`XwWLt`): hovering over answered round reveals undo icon
  - Rollback Point (`poVzY`): marked with Rollback Badge (U07)
- Questions delivered in batches — multiple blocks appear per round
- Undo click triggers checkpoint rollback

**Acceptance Criteria:**
- All 4 states match Pencil design
- Answer selection locks in visually
- Undo icon appears on hover for answered questions
- Rollback point badge shown when applicable

---

### U21 — Q&A Thread

**Atomic Step:** Step 21
**Depends on:** U07 (Rollback Badge), U11 (New Question Indicator), U12 (Course Correction Input), U20 (Question Block)
**Pencil design frame:** `8eSRk` (inside `L3hG8`)
**Pencil components:**
- `BPLqT` — Q&A Thread (default state)
- `8043v` — Q&A Thread/New Question (with floating indicator)

**File:** `frontend/src/features/qa/qa-thread.tsx`

**Requirements:**
- Props: `rounds: QaRound[]`, `stage?: string`, `stages?: string[]`, `onAnswer`, `onRollback`, `onCourseCorrect`, `onStageChange?`
- Composes: Stage selector (story-level only), ordered Question Blocks (U20), New Question Indicator (U11), Course Correction Input (U12)
- Scrollable — new questions append to bottom
- Stage selector filters by grooming/planning (story-level only)
- Multi-round: follow-up questions appear in same thread
- Superseded rounds hidden here (shown only in Decision Trail)
- Scroll position never forced — user controls scroll
- Course correction input always visible at bottom
- Two correction paths:
  1. Type in chat during current round → AI regenerates unanswered questions
  2. Click undo on past round → rollback, chat auto-focuses

**Acceptance Criteria:**
- Default and new-question states match Pencil design
- Stage selector works (story-level)
- Scroll position preserved
- Floating indicator appears when new question below viewport
- Course correction input pinned at bottom

---

### U22 — Decision Trail

**Atomic Step:** Step 22
**Depends on:** U16 (Decision Trail Entry)
**Pencil design frame:** `A3gLF` (inside `L3hG8`)
**Pencil component:** `57J4R` — Decision Trail

**File:** `frontend/src/features/decisions/decision-trail.tsx`

**Requirements:**
- Props: `decisions: Decision[]`, `level: 'story' | 'task'`
- Composes: list of Decision Trail Entries (U16) grouped by stage headers
- Flat chronological list with stage group headers:
  - Story level: Grooming → Planning → Task Decomposition → Per-task Q&A → Per-task Implementation
  - Task level: Task Q&A → Implementation Decisions only
- Active decisions at full prominence
- Superseded decisions inline at chronological position with muted styling
- Each entry has linkable URL

**Acceptance Criteria:**
- Matches Pencil design `57J4R`
- Grouped by stage headers
- Active and superseded entries visually distinct
- Scrollable

---

### U23 — Story Overview Panel

**Atomic Step:** Step 23
**Depends on:** U01 (Status Badge), U18 (Task List Item)
**Pencil design frame:** `nFRaj`
**Pencil component:** `Q76nA` — Story Overview Panel

**File:** `frontend/src/features/stories/story-overview.tsx`

**Requirements:**
- Props: `story: Story`, `tasks: Task[]`, `onTaskClick: (taskId) => void`
- Composes: story description, Status Badge (U01), owner, ordered list of Task List Items (U18)
- Always visible in the left 40% of the modal
- Clicking any task navigates modal to task detail view
- Task list order reflects execution/dependency order

**Acceptance Criteria:**
- Matches Pencil design `Q76nA`
- Task list renders all tasks with correct ordering
- Task clicks navigate to task detail

---

### U24 — Task Overview Panel

**Atomic Step:** Step 24
**Depends on:** U10 (Mark Done Button), U15 (AI Status Indicator)
**Pencil design frame:** `Q3iM3`
**Pencil components:**
- `qJ0xw` — Task Overview Panel (active state)
- `sIGXP` — Task Overview Panel/Done State

**File:** `frontend/src/features/tasks/task-overview.tsx`

**Requirements:**
- Props: `task: Task`, `onMarkDone: () => void`
- Composes: task description, assignee, AI Status Indicator (U15), Mark Done Button (U10, conditional)
- Replaces story overview panel when user drills into a task
- Mark Done button appears only when AI has finished (`state = 'running'` with completed status)
- AI status updates in real-time (parent handles SSE data)

**Acceptance Criteria:**
- Active and done states match Pencil design
- Mark Done button conditionally shown
- AI status indicator displays correctly

---

### U25 — Stories Table

**Atomic Step:** Step 25
**Depends on:** U17 (Story Table Row)
**Pencil design frame:** `lEMeF`
**Pencil component:** `ppsfk` — Stories Table

**File:** `frontend/src/features/stories/stories-table.tsx`

**Requirements:**
- Props: `stories: Story[]`, `onStoryClick: (storyId) => void`, `onNewStory: () => void`, `onReorder: (storyId, beforeId?, afterId?) => void`
- Composes: table header (Name, Status, Owner columns), "+ New" button, ordered list of Story Table Rows (U17)
- Row order = priority — top is highest
- Drag-and-drop reordering via @dnd-kit → calls onReorder with fractional index params
- Done stories at reduced visual emphasis but remain visible
- Search bar in header for full-text search across story names

**Acceptance Criteria:**
- Matches Pencil design `ppsfk`
- Drag-and-drop reordering works
- "+ New" button triggers onNewStory
- Row clicks trigger onStoryClick
- Done stories at reduced opacity

---

### U26 — Task Decomposition Review

**Atomic Step:** Step 26
**Depends on:** U05 (Task Type Icon)
**Pencil design frame:** `gh2dP`
**Pencil components:**
- `L4zqb` — Task Decomposition Review (container)
- `RMBUR` — Decomposition Task Item/Default
- `tPpnm` — Task Split Editor

**File:** `frontend/src/features/tasks/task-decomposition.tsx`

**Requirements:**
- Props: `proposedTasks: ProposedTask[]`, `onConfirm: (tasks) => void`, `onReorder`, `onMerge`, `onSplit`, `onEditTask`
- Shows AI-proposed task list with: task name, Task Type Icon (U05), dependencies
- User can: reorder (drag), merge tasks, split tasks (inline editor `tPpnm`), edit names/descriptions
- "Confirm" button locks in decomposition and begins task-level Q&A
- Dependencies visually indicated

**Acceptance Criteria:**
- Matches Pencil design `L4zqb`
- Drag reordering works
- Merge/split/edit actions work
- Confirm button triggers onConfirm

---

### U27 — Story Creation Flow

**Atomic Step:** Step 27
**Depends on:** Nothing
**Pencil design frame:** `JsJ91`
**Pencil components:**
- `Wuc5B` — Story Creation Modal/Input (initial form)
- `ux0sV` — Story Creation Modal/Review (AI-expanded description)

**File:** `frontend/src/features/stories/story-creation.tsx`

**Requirements:**
- Two-step modal flow:
  1. Input form (`Wuc5B`): Title, Description (1-2 sentences), Owner (dropdown), Story Type (Feature/Bug/Refactor)
  2. Review (`ux0sV`): AI-expanded description — user reviews, edits, approves
- After approval: story created in "To Do" status
- Story type selection determines pipeline stages

**Acceptance Criteria:**
- Both modal states match Pencil design
- Form validation (title required)
- Two-step flow: input → AI expansion → review → create
- Story type selector works

---

## Layer 4: Templates

---

### U28 — Story Detail Modal (Story View)

**Atomic Step:** Step 28
**Depends on:** U08 (Breadcrumb), U09 (Tab Switcher), U19 (Confirmation Dialog), U21 (Q&A Thread), U22 (Decision Trail), U23 (Story Overview Panel)
**Pencil design frame:** `qUuCm`
**Pencil components:**
- `AkYbm` — Modal Backdrop (story view)
- `P2Tg9` — Modal with Confirmation Dialog overlay

**File:** `frontend/src/features/stories/story-modal.tsx`

**Requirements:**
- Composes: Breadcrumb (U08, `Project > Story`), close button (X), two-column layout (40%/60%)
  - Left: Story Overview Panel (U23)
  - Right: Tab Switcher (U09) with Q&A Thread (U21) and Decision Trail (U22)
- Confirmation Dialog (U19) triggered on close when pending questions or unsent draft
- Modal covers ~80% viewport, centered
- No backdrop click dismissal — only X button or Escape key
- URL-driven: `/projects/:projectId/stories/:storyId`
- Mobile: collapses to single column with tab bar "Overview" | "Q&A" | "Decisions"

**Acceptance Criteria:**
- Matches Pencil design `AkYbm` and `P2Tg9`
- Two-column layout with correct proportions
- Confirmation dialog blocks close when needed
- URL navigation works (deep links)
- Mobile responsive collapse

---

### U29 — Story Detail Modal (Task View)

**Atomic Step:** Step 29
**Depends on:** U08 (Breadcrumb), U09 (Tab Switcher), U21 (Q&A Thread), U22 (Decision Trail), U24 (Task Overview Panel)
**Pencil design frame:** `IJSO2`
**Pencil component:** `BGlR7` — Modal Backdrop (task view)

**File:** `frontend/src/features/stories/story-modal.tsx` (same file as U28, different state)

**Requirements:**
- Same modal shell as U28, interior content changes
- Breadcrumb (U08): `Project > Story (clickable) > Task`
- Two-column layout (40%/60%):
  - Left: Task Overview Panel (U24) — replaces Story Overview Panel
  - Right: Tab Switcher (U09) with task-scoped Q&A Thread (U21) and Decision Trail (U22)
- Clicking story name in breadcrumb returns to story view
- When returning to story view, right column restores last active tab at story level
- URL-driven: `/projects/:projectId/stories/:storyId/tasks/:taskId`

**Acceptance Criteria:**
- Matches Pencil design `BGlR7`
- Breadcrumb navigation between story ↔ task views works
- Tab state preserved when navigating back to story view
- URL navigation works

---

### U30 — Main Application Layout

**Atomic Step:** Step 30
**Depends on:** U25 (Stories Table), U27 (Story Creation Flow), U28 (Story Detail Modal), U29 (Task Detail Modal)
**Pencil design frame:** `uqCBE`
**Pencil components:**
- `NPY7y` — Default State (table view)
- `DtUDE` — Modal State (story open)
- `G7fg1` — Notification State (AI question)

**File:** `frontend/src/features/layout/app-layout.tsx`

**Requirements:**
- Composes: application header (project name, search, user menu), Stories Table (U25), Story Detail Modal (U28/U29)
- Three states:
  - Default (`NPY7y`): table is primary view
  - Modal open (`DtUDE`): story/task modal overlays table
  - Notification (`G7fg1`): AI question notification
- Search in header searches across story names, task names, Q&A content, decision trail
- Real-time notifications when AI pauses with new question

**Acceptance Criteria:**
- All 3 states match Pencil design
- Modal overlays table correctly
- Search works across content types
- Notifications for new AI questions

---

## Parallel Execution Guide

### Phase A — All Atoms (fully parallel, no deps)
Start all simultaneously:
- **U01** Status Badge
- **U02** Attention Dot
- **U03** Domain Tag
- **U04** Story Type Tag
- **U05** Task Type Icon
- **U06** Superseded Badge
- **U07** Rollback Point Badge
- **U08** Breadcrumb
- **U09** Tab Switcher
- **U10** Mark Done Button
- **U11** New Question Indicator
- **U12** Course Correction Input
- **U13** Answer Option Card
- **U19** Confirmation Warning Dialog
- **U27** Story Creation Flow

### Phase B — Molecules with atom deps
- **U14** Other Option (needs U13)
- **U15** AI Status Indicator (needs U01)
- **U16** Decision Trail Entry (needs U03, U06)
- **U17** Story Table Row (needs U01, U02, U04)
- **U18** Task List Item (needs U01, U02, U05)

### Phase C — Organisms
- **U20** Question Block (needs U03, U07, U13, U14)
- **U22** Decision Trail (needs U16)
- **U23** Story Overview Panel (needs U01, U18)
- **U24** Task Overview Panel (needs U10, U15)
- **U25** Stories Table (needs U17)
- **U26** Task Decomposition Review (needs U05)

### Phase D — Organisms with organism deps
- **U21** Q&A Thread (needs U07, U11, U12, U20)

### Phase E — Templates
- **U28** Story Detail Modal — Story View (needs U08, U09, U19, U21, U22, U23)
- **U29** Story Detail Modal — Task View (needs U08, U09, U21, U22, U24)

### Phase F — Final assembly
- **U30** Main Application Layout (needs U25, U27, U28, U29)

---

## Notes

- **API integration:** Templates (U28–U30) wire up TanStack Query hooks from the generated API client (T22). Atoms and molecules are pure presentational.
- **SSE integration:** U30 sets up the SSE connection (`useSSE` hook). Real-time updates invalidate query caches, which re-render organisms automatically.
- **State management:** Zustand stores (`ui-store`, `sse-store`) are used by U21 (tab state, drafts) and U30 (SSE connection state).
- **Drag-and-drop:** U25 (Stories Table) and U26 (Task Decomposition) use @dnd-kit. Import from existing frontend deps (T09).
- **Mobile responsive:** Handled in U28/U29 — single column collapse with tab bar.
