# UI/UX Design Requirements — Atomic Design Breakdown

> Serialized steps from smallest atoms → molecules → organisms → templates → pages.
> Each step builds on completed previous steps. No visual detail (colors, radius, spacing) — requirements and behavior only.

---

## Layer 1: Atoms

These are the smallest, indivisible UI elements. They have no dependencies on other custom components.

---

### Step 1: Status Badge

**What it is:** A small label that communicates state.

**Requirements:**
- Renders a text label with a visual treatment that distinguishes different states
- Must support these story statuses: `To Do`, `In Progress`, `Done`
- Must support these task states: `Done`, `AI working`, `Needs input`, `Blocked`
- Each state must be visually distinct from every other state at a glance
- Should be compact — never wraps to multiple lines

**Variants:**
- Story status badge (3 states)
- Task state badge (4 states)

---

### Step 2: Attention Dot

**What it is:** A small pulsing indicator that signals "this needs your attention."

**Requirements:**
- Small circular element with a repeating pulse animation
- Communicates urgency without being disruptive — subtle, not alarming
- Must be noticeable in peripheral vision while user focuses elsewhere in the UI
- Used inline next to text (story name, task name) — must not shift layout when appearing/disappearing

---

### Step 3: Domain Tag

**What it is:** A label indicating which domain/perspective a question belongs to.

**Requirements:**
- Displays a domain name (e.g., "Security", "Backend", "UX", "Business Logic", "Design", "Marketing")
- Visually lighter than status badges — informational, not actionable
- Compact enough to sit above or beside a question without dominating it

---

### Step 4: Story Type Tag

**What it is:** A tag distinguishing story types at a glance.

**Requirements:**
- Displays the story type identifier (e.g., "BUG", "FEATURE", "REFACTOR")
- Bug tag is the most critical — must be immediately noticeable in the story list
- Compact, sits inline before the story name
- Only shown when type is not the default (feature). Bugs and refactors always show the tag.

---

### Step 5: Task Type Icon

**What it is:** An icon representing what kind of task this is.

**Requirements:**
- Three types: Design (🎨), Test Planning (✅), Code Implementation (⚡)
- Immediately communicates task category without reading the name
- Fixed size, used inline in task lists

---

### Step 6: Superseded Badge

**What it is:** A badge marking a decision as superseded by a checkpoint rollback.

**Requirements:**
- Communicates "this decision was overridden and is no longer active"
- Applied to decision trail entries that have been rolled back
- The text of the superseded entry should appear visually muted (greyed out, strikethrough)
- Must be distinguishable from active decisions without requiring hover or click

---

### Step 7: Rollback Point Badge

**What it is:** A badge marking where a Q&A thread was rewound to.

**Requirements:**
- Communicates "the thread was rolled back to this point"
- Appears on the Q&A round that was rolled back to
- Visually distinct from the superseded badge — this marks a *restoration point*, not a *removed item*

---

### Step 8: Breadcrumb

**What it is:** A navigation path showing the current location within the hierarchy.

**Requirements:**
- Displays the full path: `Project > Story > Task` (task level only when viewing a task)
- Each segment except the last is clickable for navigation
- Story name in breadcrumb navigates back from task view to story view
- Always visible at the top of the modal
- Truncates long names with ellipsis rather than wrapping

---

### Step 9: Tab Switcher

**What it is:** A horizontal tab bar for switching between content views.

**Requirements:**
- Displays 2-4 tab labels
- One tab is active at a time — visually indicated
- Clicking a tab switches the content below it
- Used in two contexts:
  - Right column of modal: "Q&A Thread" | "Decision Trail"
  - Mobile collapsed view: "Overview" | "Q&A" | "Decisions"
- Remembers which tab was active when navigating back (e.g., returning from task to story view restores the last-viewed story-level tab)

---

### Step 10: "Mark Done" Button

**What it is:** The single sign-off action for completing a task.

**Requirements:**
- Only appears when AI has completed its work on a task and the task is not yet marked done
- Prominent, clearly actionable — this is the most important action on the task view
- Single action: marks the task as done. No other sign-off options in the web app.
- Should not appear when the task is still running, blocked, or waiting for input

---

### Step 11: "New Question ↓" Floating Indicator

**What it is:** A floating element that appears when a new question arrives while the user is scrolled up.

**Requirements:**
- Only appears when: (a) a new question has been posted at the bottom of the Q&A thread, AND (b) the user's scroll position is above the new question
- Clicking it scrolls the user to the new question
- Disappears when the new question is scrolled into view
- Does NOT force scroll position — user's current position is always preserved until they choose to jump
- Anchored to the bottom of the scrollable Q&A area, floating above content

---

### Step 12: Course Correction Chat Input

**What it is:** A text input field for free-form course correction of the AI.

**Requirements:**
- Always visible at the bottom of the Q&A thread (both story and task level)
- Visually subdued compared to answer option cards — signals it's secondary to predefined selection
- Placeholder text: "Course-correct the AI's approach..."
- Draft text persists within the session — switching tabs preserves the draft
- Navigating away from the modal clears all drafts
- Submit action sends the text as a course correction to the AI

---

## Layer 2: Molecules

Molecules combine atoms into functional units with specific behaviors.

---

### Step 13: Answer Option Card

**What it is:** A selectable card representing one predefined answer to an AI question.

**Composed of:**
- Text label describing the answer option
- Selection indicator (radio-style — one answer per question)

**Requirements:**
- Displayed as a list of options within a question block
- Exactly one option can be selected per question
- Selecting an option immediately records the answer — no separate "submit" needed
- The selected option should be visually distinct from unselected options
- Selection is irreversible within the normal flow (reversible only via checkpoint rollback)

---

### Step 14: "Other" Option with Free-form Input

**What it is:** A special answer option that expands into a text input when selected.

**Composed of:**
- An answer option card labeled "Other"
- A text input that appears when "Other" is selected

**Requirements:**
- Present on every question as the last option
- Selecting "Other" expands an inline text input below it
- User types their custom answer and submits
- The submitted text becomes the recorded answer for this question
- This is the escape hatch from predefined answers — not the primary interaction

---

### Step 15: AI Status Indicator

**What it is:** A compound element showing the current state of the AI agent for a task.

**Composed of:**
- Task state badge (atom: Step 2 variant)
- Status description text (e.g., "implementing file X", "paused on question", "blocked — waiting on T-142-3")

**Requirements:**
- Four states: `running`, `paused`, `blocked`, `done`
- Running state should have a subtle animation suggesting ongoing activity
- Paused state must clearly communicate "waiting for you" — visually distinct from blocked
- Blocked state shows what it's blocked on (dependency reference)
- Description text provides brief context about what the AI is currently doing
- Updates in real-time via WebSocket

---

### Step 16: Decision Trail Entry

**What it is:** A single question + answer pair in the decision log.

**Composed of:**
- Domain tag (atom: Step 3)
- Question text
- Selected answer text
- Superseded badge (atom: Step 6) — only when this decision has been rolled back

**Requirements:**
- Compact, scannable format — optimized for reading many entries quickly
- Active decisions display at full prominence
- Superseded decisions appear muted with strikethrough and superseded badge
- Maintains chronological position even when superseded (not moved or hidden)
- Each entry should have a stable URL for deep linking from MR descriptions

---

### Step 17: Story Table Row

**What it is:** A single row in the main stories table.

**Composed of:**
- Story type tag (atom: Step 4) — only for bugs/refactors
- Story name text
- Attention dot (atom: Step 2) — only when input is needed
- Status badge (atom: Step 1, story variant)
- Owner text

**Requirements:**
- Three column layout: Name (with type tag + attention dot inline), Status, Owner
- Draggable for reordering (drag = reprioritize)
- Clickable — opens story detail modal
- Done stories appear with reduced visual emphasis (lower opacity or muted)
- Row order represents priority — top = highest priority
- Drag and drop must provide visual feedback: drag handle affordance, drop target indicator

---

### Step 18: Task List Item

**What it is:** A single task entry within a story's task list.

**Composed of:**
- Task type icon (atom: Step 5)
- Task name text
- Attention dot (atom: Step 2) — only when task needs input
- Task state badge (atom: Step 1, task variant)

**Requirements:**
- Displayed in the story's overview panel as a list
- Clickable — navigates to task detail view within the modal
- Attention dot appears only when the task has an unanswered pending question
- State badge reflects current AI state: done, AI working, needs input, blocked
- AI working state should have a subtle pulsing animation on the badge

---

### Step 19: Confirmation Warning Dialog

**What it is:** A dialog that prevents accidental modal closure when there's unsaved state.

**Requirements:**
- Appears when user tries to close the modal (X or Escape) AND either:
  - There are pending unanswered questions, OR
  - The user has typed an unsent draft in the chat input
- Two actions: "Stay" (cancel close) and "Leave" (confirm close and discard)
- If no pending items or drafts, modal closes immediately without this dialog
- Blocks the close action until user confirms

---

## Layer 3: Organisms

Organisms combine molecules into distinct, self-contained sections of the interface.

---

### Step 20: Question Block

**What it is:** A complete AI-generated question with all its answer options, displayed in the Q&A thread.

**Composed of:**
- Domain tag (atom: Step 3)
- Question text
- List of answer option cards (molecule: Step 13)
- "Other" option with free-form input (molecule: Step 14)
- Undo icon (appears on hover over a previously answered round)

**Requirements:**
- Displayed as a distinct block in the chat-like Q&A thread
- Questions are delivered in batches — multiple question blocks appear together per round
- When a question is answered, the selected option is visually locked in
- Hovering over a previously answered round reveals an undo icon for checkpoint rollback
- Clicking undo triggers checkpoint rollback: all subsequent rounds are removed from the Q&A thread, moved to Decision Trail with "superseded" badges, and the chat input auto-focuses for correction
- The rolled-back round receives a "Rollback point" badge (atom: Step 7)

---

### Step 21: Q&A Thread

**What it is:** The full scrollable chat-like thread of AI questions and human answers.

**Composed of:**
- Stage selector at top ("Grooming" | "Planning" for story-level, or no selector for task-level)
- Ordered list of question blocks (organism: Step 20)
- Rollback point badges (atom: Step 7) where applicable
- "New question ↓" floating indicator (atom: Step 11) when needed
- Course correction chat input (atom: Step 12) pinned at bottom

**Requirements:**
- Scrollable — new questions append to the bottom
- Stage selector filters the thread to show only questions from the selected stage (story-level only)
- Multi-round: after answering a batch, follow-up questions appear in the same thread
- Superseded rounds (from rollbacks) are hidden from this view — they only appear in the Decision Trail
- When a new question arrives while user is scrolled up, the floating "New question ↓" indicator appears
- Scroll position is never forced — user always controls their scroll
- Course correction input is always visible at the bottom, even when scrolling

**Two correction paths supported:**
1. Type in chat input during current round → AI regenerates unanswered questions, preserves already-answered ones
2. Click undo on past round → rollback to checkpoint, chat input auto-focuses

---

### Step 22: Decision Trail

**What it is:** A chronological log of all decisions made for a story or task.

**Composed of:**
- List of decision trail entries (molecule: Step 16), grouped by stage

**Requirements:**
- Flat chronological list, grouped under stage headers:
  - Story level: Grooming → Planning → Task Decomposition → Per-task Q&A → Per-task Implementation
  - Task level: Task Q&A → Implementation Decisions only
- Active decisions at full prominence
- Superseded decisions shown inline at their chronological position with superseded badge + muted styling
- Stage groups are collapsible (optional, nice-to-have)
- Each decision entry has a linkable URL for referencing from MR descriptions

---

### Step 23: Story Overview Panel (Left Column)

**What it is:** The left column of the story detail modal showing story info and task list.

**Composed of:**
- Story description text
- Status badge (atom: Step 1) and owner
- Task list — ordered list of task list items (molecule: Step 18)

**Requirements:**
- Always visible in the left 40% of the modal
- Task list shows all tasks with their type, name, state, and attention indicators
- Clicking any task navigates the modal to task detail view (replaces both columns' content)
- Task list order reflects execution/dependency order

---

### Step 24: Task Overview Panel (Left Column)

**What it is:** The left column of the task detail view showing task info and AI status.

**Composed of:**
- Task description text
- Assignee
- AI status indicator (molecule: Step 15)
- "Mark Done" button (atom: Step 10) — conditionally shown

**Requirements:**
- Replaces the story overview panel when user drills into a task
- Shows the task description and current AI state
- "Mark Done" button appears only when AI has finished its work
- AI status updates in real-time

---

### Step 25: Stories Table

**What it is:** The main table view — the primary interface of the entire application.

**Composed of:**
- Table header: column labels (Name, Status, Owner)
- "+ New" button in the header area
- Ordered list of story table rows (molecule: Step 17)

**Requirements:**
- Row order = priority — top is highest
- Rows are drag-and-drop reorderable for prioritization
- Clicking any row opens the story detail modal
- "+ New" button opens the story creation flow
- Table should support basic search/filter (§6.8 — full-text search across story names, task names, Q&A content)
- Done stories appear with reduced visual emphasis but remain visible (not hidden)

---

### Step 26: Task Decomposition Review

**What it is:** The interface for reviewing, merging, splitting, and reordering AI-proposed tasks before execution begins.

**Requirements:**
- Appears after planning Q&A is completed
- Shows AI-proposed list of tasks with: task name, type icon, estimated scope, dependencies
- User can:
  - Reorder tasks by dragging
  - Merge two or more tasks into one (select + merge action)
  - Split a task into smaller tasks (opens inline editor)
  - Edit task names/descriptions
- A "Confirm" action locks in the task decomposition and begins task-level Q&A
- Dependencies between tasks are visually indicated (e.g., "depends on Task 1")

---

### Step 27: Story Creation Flow

**What it is:** The flow for creating a new story.

**Requirements:**
- Triggered by "+ New" button in the stories table
- Opens a modal/form with fields: Title, Description (brief — 1-2 sentences), Owner, Story Type (Feature / Bug / Refactor)
- After submission:
  - AI expands the brief into a structured description
  - User reviews, edits, and approves the expanded description
  - Story is created in "To Do" status
- Story type selection determines which pipeline stages the story will go through

---

## Layer 4: Templates

Templates define the layout structure of full views by composing organisms.

---

### Step 28: Story Detail Modal — Story View

**What it is:** The modal layout when viewing a story (not drilled into a task).

**Composed of:**
- Breadcrumb (atom: Step 8) — `Project > Story`
- Close button (X)
- Two-column layout (40% / 60%):
  - Left: Story overview panel (organism: Step 23)
  - Right: Tab switcher (atom: Step 9) with two tabs:
    - Q&A Thread (organism: Step 21) — default tab
    - Decision Trail (organism: Step 22)
- Confirmation warning dialog (molecule: Step 19) — triggered on close when needed

**Layout requirements:**
- Modal covers ~80% of viewport, centered
- No backdrop click dismissal — only X button or Escape key
- Confirmation dialog appears before closing if there are pending questions or unsent draft
- **Mobile:** Collapses to single column with tab bar: "Overview" | "Q&A" | "Decisions"

---

### Step 29: Story Detail Modal — Task View

**What it is:** The modal layout when drilled into a specific task.

**Composed of:**
- Breadcrumb (atom: Step 8) — `Project > Story > Task` (Story is clickable to navigate back)
- Close button (X)
- Two-column layout (40% / 60%):
  - Left: Task overview panel (organism: Step 24)
  - Right: Tab switcher (atom: Step 9) with two tabs:
    - Q&A Thread (organism: Step 21) — task-scoped, single-owner
    - Decision Trail (organism: Step 22) — task-scoped only

**Navigation requirements:**
- Clicking story name in breadcrumb returns to story view
- When returning to story view, the right column restores whichever tab (Q&A or Decision Trail) was last active at the story level
- Same modal shell — only interior content changes
- **Mobile:** Same collapsed tab bar pattern

---

### Step 30: Main Application Layout

**What it is:** The top-level page layout.

**Composed of:**
- Application header (project name, search, user menu)
- Stories table (organism: Step 25) as the main content
- Story detail modal (templates: Step 28 / Step 29) — opens on top of the table

**Requirements:**
- Table is the primary and default view
- Modal overlays the table when a story is opened
- Search in the header searches across story names, task names, Q&A content, and decision trail
- Real-time notifications when AI pauses with a new question (push notification integration)

---

## Build Order Summary

This is the serialized order. Each step depends only on previously completed steps.

| Step | Name | Layer | Depends On |
|------|------|-------|------------|
| 1 | Status Badge | Atom | — |
| 2 | Attention Dot | Atom | — |
| 3 | Domain Tag | Atom | — |
| 4 | Story Type Tag | Atom | — |
| 5 | Task Type Icon | Atom | — |
| 6 | Superseded Badge | Atom | — |
| 7 | Rollback Point Badge | Atom | — |
| 8 | Breadcrumb | Atom | — |
| 9 | Tab Switcher | Atom | — |
| 10 | "Mark Done" Button | Atom | — |
| 11 | "New Question ↓" Indicator | Atom | — |
| 12 | Course Correction Chat Input | Atom | — |
| 13 | Answer Option Card | Molecule | — |
| 14 | "Other" Option + Free-form | Molecule | 13 |
| 15 | AI Status Indicator | Molecule | 1 |
| 16 | Decision Trail Entry | Molecule | 3, 6 |
| 17 | Story Table Row | Molecule | 1, 2, 4 |
| 18 | Task List Item | Molecule | 1, 2, 5 |
| 19 | Confirmation Warning Dialog | Molecule | — |
| 20 | Question Block | Organism | 3, 7, 13, 14 |
| 21 | Q&A Thread | Organism | 7, 11, 12, 20 |
| 22 | Decision Trail | Organism | 16 |
| 23 | Story Overview Panel | Organism | 1, 18 |
| 24 | Task Overview Panel | Organism | 10, 15 |
| 25 | Stories Table | Organism | 17 |
| 26 | Task Decomposition Review | Organism | 5 |
| 27 | Story Creation Flow | Organism | — |
| 28 | Story Detail Modal (Story View) | Template | 8, 9, 19, 21, 22, 23 |
| 29 | Story Detail Modal (Task View) | Template | 8, 9, 21, 22, 24 |
| 30 | Main Application Layout | Template | 25, 27, 28, 29 |