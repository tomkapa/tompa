CREATE TABLE users (
    id                   uuid        PRIMARY KEY,
    email                text        NOT NULL UNIQUE,
    display_name         text        NOT NULL,
    avatar_url           text        NULL,
    oauth_provider       text        NOT NULL,
    oauth_provider_id    text        NOT NULL,
    created_at           timestamptz NOT NULL DEFAULT now(),
    updated_at           timestamptz NOT NULL DEFAULT now(),
    deleted_at           timestamptz NULL,

    CONSTRAINT uq_users_oauth UNIQUE (oauth_provider, oauth_provider_id)
);
