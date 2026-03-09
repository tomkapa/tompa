# Data Models Codemap
_Updated: 2026-03-07_

## Database: PostgreSQL with RLS

All tables use UUIDv7 primary keys. Row-Level Security enforced via
`SET LOCAL app.current_org_id = '<uuid>'` within transactions.

## Core Tables

### organizations
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| name | text | |
| created_at | timestamptz | |

### users
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| email | text unique | |
| name | text | |
| avatar_url | text | |
| provider | text | google/github |
| provider_id | text | |
| created_at | timestamptz | |

### org_members
| Column | Type | Notes |
|--------|------|-------|
| org_id | uuid FK → organizations | |
| user_id | uuid FK → users | |
| role | OrgRole | owner/admin/member |

### projects
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| org_id | uuid FK (RLS) | |
| name | text | unique per org |
| slug | text | |
| created_at | timestamptz | |

### stories
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| project_id | uuid FK | |
| org_id | uuid FK (RLS) | |
| title | text | |
| description | text | |
| pending_refined_description | text nullable | AI draft awaiting approval |
| story_type | StoryType | feature/bug/refactor |
| status | StoryStatus | todo/in_progress/done |
| pipeline_stage | PipelineStage | grooming/planning/decomposition/… |
| rank | text | fractional index for ordering |
| created_at | timestamptz | |

### tasks
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| story_id | uuid FK | |
| org_id | uuid FK (RLS) | |
| title | text | |
| task_type | TaskType | code/test/design |
| state | TaskState | pending/qa/running/paused/blocked/done |
| created_at | timestamptz | |

### task_dependencies
| Column | Type | Notes |
|--------|------|-------|
| task_id | uuid FK | |
| depends_on_task_id | uuid FK | |

### qa_rounds
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| story_id | uuid FK nullable | |
| task_id | uuid FK nullable | |
| org_id | uuid FK (RLS) | |
| stage | QaStage | grooming/planning/task_qa/implementation |
| questions | jsonb | array of {id, text, options, answer} |
| status | QaRoundStatus | active/superseded |
| assignee_id | uuid FK → users nullable | |
| created_at | timestamptz | |

### agent_sessions
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| story_id | uuid FK | |
| org_id | uuid FK (RLS) | |
| container_id | text | maps to WS registry key |
| stage | PipelineStage | |
| created_at | timestamptz | |

### knowledge_entries
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| project_id | uuid FK | |
| org_id | uuid FK (RLS) | |
| title | text | |
| content | text | |
| category | KnowledgeCategory | convention/adr/api_doc/design_system/custom |
| created_at | timestamptz | |

### container_api_keys
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| org_id | uuid FK (RLS) | |
| key_hash | text | bcrypt hash |
| label | text | |
| created_at | timestamptz | |

### decision_patterns
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| org_id | uuid FK (RLS) | |
| project_id | uuid FK | |
| title | text | |
| pattern | text | |
| confidence | float | 0.0–1.0 |
| tags | text[] | |
| created_at | timestamptz | |

### project_profiles
| Column | Type | Notes |
|--------|------|-------|
| id | uuid PK | |
| project_id | uuid FK unique | |
| org_id | uuid FK (RLS) | |
| grooming_roles | jsonb | domain-specific grooming configuration |
| qa_config | jsonb | per-stage QA configuration |
| created_at | timestamptz | |

## Shared Enums (`shared/enums.rs`)

```
StoryType:      feature | bug | refactor
StoryStatus:    todo | in_progress | done
PipelineStage:  grooming | planning | decomposition | implementation | testing | review
TaskType:       design | test | code
TaskState:      pending | qa | running | paused | blocked | done
QaStage:        grooming | planning | task_qa | implementation
QaRoundStatus:  active | superseded
ContainerMode:  project | dev | standalone
OrgRole:        owner | admin | member
KnowledgeCategory: convention | adr | api_doc | design_system | custom
```

## WebSocket Messages (`shared/messages.rs`)

```
ServerToContainer:
  Execute { session_id, system_prompt, prompt, model }
  Ping

ContainerToServer:
  ExecutionResult { session_id, output: serde_json::Value }
  ExecutionFailed { session_id, error }
  Pong
```

## RLS Pattern
```sql
-- In every repo transaction:
SET LOCAL app.current_org_id = '<org_uuid>';
-- All queries automatically scoped to org via RLS policies
```
Exception: org listing queries bypass RLS with explicit `JOIN org_members ON user_id = $1`.
