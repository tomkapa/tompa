CREATE UNIQUE INDEX projects_org_id_name_unique
    ON projects (org_id, lower(name))
    WHERE deleted_at IS NULL;
