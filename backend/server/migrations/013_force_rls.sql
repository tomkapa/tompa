-- FORCE RLS on tenant-scoped data tables — even the table owner must
-- go through RLS policies on these tables.
ALTER TABLE projects              FORCE ROW LEVEL SECURITY;
ALTER TABLE stories               FORCE ROW LEVEL SECURITY;
ALTER TABLE tasks                 FORCE ROW LEVEL SECURITY;
ALTER TABLE qa_rounds             FORCE ROW LEVEL SECURITY;
ALTER TABLE knowledge_entries     FORCE ROW LEVEL SECURITY;

-- organizations, org_members, container_api_keys keep ENABLE only:
-- the table owner bypasses ENABLE-only RLS, which is needed for
-- cross-org queries (list_orgs_for_user, verify_api_key, etc.)
