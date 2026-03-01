CREATE TABLE container_api_keys (
    id                  uuid        PRIMARY KEY,
    org_id              uuid        NOT NULL REFERENCES organizations (id),
    project_id          uuid        NOT NULL REFERENCES projects (id),
    key_hash            text        NOT NULL,
    label               text        NOT NULL,
    container_mode      text        NOT NULL CHECK (container_mode IN ('project', 'dev', 'standalone')),
    last_connected_at   timestamptz NULL,
    created_at          timestamptz NOT NULL DEFAULT now(),
    revoked_at          timestamptz NULL
);
