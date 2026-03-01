CREATE TABLE stories (
    id              uuid        PRIMARY KEY,
    org_id          uuid        NOT NULL REFERENCES organizations (id),
    project_id      uuid        NOT NULL REFERENCES projects (id),
    title           text        NOT NULL,
    description     text        NOT NULL DEFAULT '',
    story_type      text        NOT NULL CHECK (story_type IN ('feature', 'bug', 'refactor')),
    status          text        NOT NULL DEFAULT 'todo' CHECK (status IN ('todo', 'in_progress', 'done')),
    owner_id        uuid        NOT NULL REFERENCES users (id),
    rank            text        NOT NULL,
    pipeline_stage  text        NULL CHECK (pipeline_stage IN ('grooming', 'planning', 'decomposition', 'implementation', 'testing', 'review')),
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz NULL
);

CREATE INDEX idx_stories_project_rank
    ON stories (project_id, rank)
    WHERE deleted_at IS NULL;
