CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL,
    password_hash TEXT,
    oauth_provider TEXT,
    oauth_subject TEXT,
    display_name TEXT,
    full_name TEXT,
    avatar_object_key TEXT,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    email_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT users_email_not_blank CHECK (length(btrim(email)) > 0),
    CONSTRAINT users_password_hash_not_blank CHECK (
        password_hash IS NULL OR length(btrim(password_hash)) > 0
    ),
    CONSTRAINT users_oauth_identity_complete CHECK (
        (oauth_provider IS NULL AND oauth_subject IS NULL)
        OR (oauth_provider IS NOT NULL AND oauth_subject IS NOT NULL)
    ),
    CONSTRAINT users_oauth_provider_not_blank CHECK (
        oauth_provider IS NULL OR length(btrim(oauth_provider)) > 0
    ),
    CONSTRAINT users_oauth_subject_not_blank CHECK (
        oauth_subject IS NULL OR length(btrim(oauth_subject)) > 0
    ),
    CONSTRAINT users_email_verified_at_consistent CHECK (
        email_verified = TRUE OR email_verified_at IS NULL
    )
);

CREATE UNIQUE INDEX users_email_unique_idx ON users (lower(email));

CREATE UNIQUE INDEX users_oauth_identity_unique_idx
    ON users (oauth_provider, oauth_subject)
    WHERE oauth_provider IS NOT NULL AND oauth_subject IS NOT NULL;

CREATE INDEX users_last_seen_at_idx ON users (last_seen_at DESC);
