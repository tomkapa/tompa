CREATE TABLE qa_rounds (
    id            uuid        PRIMARY KEY,
    org_id        uuid        NOT NULL REFERENCES organizations (id),
    story_id      uuid        NOT NULL REFERENCES stories (id),
    task_id       uuid        NULL REFERENCES tasks (id),
    stage         text        NOT NULL CHECK (stage IN ('grooming', 'planning', 'task_qa', 'implementation')),
    round_number  integer     NOT NULL,
    status        text        NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'superseded')),
    content       jsonb       NOT NULL DEFAULT '{}',
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_qa_rounds_story_task_stage
    ON qa_rounds (story_id, task_id, stage, round_number);
