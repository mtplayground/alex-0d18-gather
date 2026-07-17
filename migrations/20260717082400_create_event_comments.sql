CREATE TABLE event_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    author_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT event_comments_body_not_blank CHECK (length(btrim(body)) > 0),
    CONSTRAINT event_comments_body_length CHECK (length(body) <= 2000)
);

CREATE INDEX event_comments_event_created_at_idx
    ON event_comments (event_id, created_at ASC);

CREATE INDEX event_comments_author_created_at_idx
    ON event_comments (author_user_id, created_at DESC);
