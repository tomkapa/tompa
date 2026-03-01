CREATE TABLE tasks (
    id                 uuid        PRIMARY KEY,
    org_id             uuid        NOT NULL REFERENCES organizations (id),
    story_id           uuid        NOT NULL REFERENCES stories (id),
    name               text        NOT NULL,
    description        text        NOT NULL DEFAULT '',
    task_type          text        NOT NULL CHECK (task_type IN ('design', 'test', 'code')),
    state              text        NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'qa', 'running', 'paused', 'blocked', 'done')),
    position           integer     NOT NULL,
    assignee_id        uuid        NULL REFERENCES users (id),
    claude_session_id  text        NULL,
    ai_status_text     text        NULL,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz NULL
);

CREATE INDEX idx_tasks_story_position
    ON tasks (story_id, position)
    WHERE deleted_at IS NULL;
