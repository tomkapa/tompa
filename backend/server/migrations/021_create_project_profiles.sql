-- Table: project_profiles
CREATE TABLE project_profiles (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                  UUID NOT NULL REFERENCES organizations(id),
    project_id              UUID NOT NULL REFERENCES projects(id),
    content                 JSONB NOT NULL DEFAULT '{}',
    patterns_at_generation  INT NOT NULL DEFAULT 0,
    generated_by            TEXT NOT NULL DEFAULT 'auto',
    generated_at            TIMESTAMPTZ,
    edited_at               TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,

    CONSTRAINT uq_profile_per_project UNIQUE (project_id)
);

CREATE INDEX idx_pp_org_project ON project_profiles(org_id, project_id);

ALTER TABLE project_profiles ENABLE ROW LEVEL SECURITY;
CREATE POLICY pp_org_isolation ON project_profiles
    USING (org_id = current_setting('app.current_org_id')::uuid);
