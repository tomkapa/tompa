CREATE TABLE org_members (
    id          uuid        PRIMARY KEY,
    org_id      uuid        NOT NULL REFERENCES organizations (id),
    user_id     uuid        NOT NULL REFERENCES users (id),
    role        text        NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    created_at  timestamptz NOT NULL DEFAULT now(),

    CONSTRAINT uq_org_members_org_user UNIQUE (org_id, user_id)
);
