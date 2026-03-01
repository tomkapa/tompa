CREATE TABLE projects (
    id               uuid        PRIMARY KEY,
    org_id           uuid        NOT NULL REFERENCES organizations (id),
    name             text        NOT NULL,
    description      text        NULL,
    github_repo_url  text        NULL,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at       timestamptz NULL
);
