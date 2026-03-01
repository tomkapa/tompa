-- Enable Row Level Security on all tenant-scoped tables

ALTER TABLE organizations         ENABLE ROW LEVEL SECURITY;
ALTER TABLE org_members           ENABLE ROW LEVEL SECURITY;
ALTER TABLE projects              ENABLE ROW LEVEL SECURITY;
ALTER TABLE stories               ENABLE ROW LEVEL SECURITY;
ALTER TABLE tasks                 ENABLE ROW LEVEL SECURITY;
ALTER TABLE qa_rounds             ENABLE ROW LEVEL SECURITY;
ALTER TABLE container_api_keys    ENABLE ROW LEVEL SECURITY;
ALTER TABLE knowledge_entries     ENABLE ROW LEVEL SECURITY;

-- RLS policies: isolate rows by the org_id set in the session config

CREATE POLICY org_isolation ON organizations
    USING (id::text = current_setting('app.org_id', true));

CREATE POLICY org_isolation ON org_members
    USING (org_id::text = current_setting('app.org_id', true));

CREATE POLICY org_isolation ON projects
    USING (org_id::text = current_setting('app.org_id', true));

CREATE POLICY org_isolation ON stories
    USING (org_id::text = current_setting('app.org_id', true));

CREATE POLICY org_isolation ON tasks
    USING (org_id::text = current_setting('app.org_id', true));

CREATE POLICY org_isolation ON qa_rounds
    USING (org_id::text = current_setting('app.org_id', true));

CREATE POLICY org_isolation ON container_api_keys
    USING (org_id::text = current_setting('app.org_id', true));

CREATE POLICY org_isolation ON knowledge_entries
    USING (org_id::text = current_setting('app.org_id', true));
