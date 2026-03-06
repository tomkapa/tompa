CREATE TABLE agent_sessions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      UUID NOT NULL REFERENCES organizations(id),
    project_id  UUID NOT NULL REFERENCES projects(id),
    story_id    UUID REFERENCES stories(id),
    task_id     UUID REFERENCES tasks(id),
    stage       TEXT NOT NULL,
    role        TEXT,
    session_id  UUID NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
