CREATE TABLE knowledge_entries (
    id          uuid        PRIMARY KEY,
    org_id      uuid        NOT NULL REFERENCES organizations (id),
    project_id  uuid        NULL REFERENCES projects (id),
    story_id    uuid        NULL REFERENCES stories (id),
    category    text        NOT NULL CHECK (category IN ('convention', 'adr', 'api_doc', 'design_system', 'custom')),
    title       text        NOT NULL,
    content     text        NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    deleted_at  timestamptz NULL
);

CREATE INDEX idx_knowledge_entries_org_project
    ON knowledge_entries (org_id, project_id)
    WHERE deleted_at IS NULL;
