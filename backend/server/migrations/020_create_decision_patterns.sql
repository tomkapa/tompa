-- Extension: pg_trgm for fuzzy text similarity
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Table: decision_patterns
CREATE TABLE decision_patterns (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL REFERENCES organizations(id),
    project_id      UUID REFERENCES projects(id),
    domain          TEXT NOT NULL,
    pattern         TEXT NOT NULL,
    rationale       TEXT NOT NULL,
    tags            TEXT[] NOT NULL DEFAULT '{}',
    confidence      REAL NOT NULL DEFAULT 0.8,
    usage_count     INT NOT NULL DEFAULT 0,
    override_count  INT NOT NULL DEFAULT 0,
    source_story_id UUID REFERENCES stories(id),
    source_round_id UUID REFERENCES qa_rounds(id),
    superseded_by   UUID REFERENCES decision_patterns(id),
    search_vector   TSVECTOR,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ
);

-- Trigger to maintain search_vector (to_tsvector is STABLE, not IMMUTABLE,
-- so we cannot use GENERATED ALWAYS AS).
CREATE OR REPLACE FUNCTION dp_search_vector_trigger() RETURNS trigger AS $$
BEGIN
    NEW.search_vector :=
        to_tsvector('english',
            NEW.pattern || ' ' || NEW.rationale || ' ' ||
            array_to_string(NEW.tags, ' ')
        );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER dp_search_vector_update
    BEFORE INSERT OR UPDATE OF pattern, rationale, tags
    ON decision_patterns
    FOR EACH ROW
    EXECUTE FUNCTION dp_search_vector_trigger();

CREATE INDEX idx_dp_org_project   ON decision_patterns(org_id, project_id);
CREATE INDEX idx_dp_domain        ON decision_patterns(org_id, domain);
CREATE INDEX idx_dp_tags          ON decision_patterns USING GIN(tags);
CREATE INDEX idx_dp_pattern_trgm  ON decision_patterns USING GIN(pattern gin_trgm_ops);
CREATE INDEX idx_dp_fts           ON decision_patterns USING GIN(search_vector);
CREATE INDEX idx_dp_active        ON decision_patterns(org_id, project_id)
    WHERE deleted_at IS NULL AND superseded_by IS NULL;

ALTER TABLE decision_patterns ENABLE ROW LEVEL SECURITY;
CREATE POLICY dp_org_isolation ON decision_patterns
    USING (org_id = current_setting('app.current_org_id')::uuid);
