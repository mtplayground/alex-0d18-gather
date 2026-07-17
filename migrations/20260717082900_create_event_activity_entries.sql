CREATE TABLE event_activity_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    actor_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    activity_type TEXT NOT NULL,
    subject_type TEXT,
    subject_id UUID,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT event_activity_entries_type_not_blank CHECK (
        length(btrim(activity_type)) > 0
    ),
    CONSTRAINT event_activity_entries_type_length CHECK (
        length(activity_type) <= 80
    ),
    CONSTRAINT event_activity_entries_subject_type_not_blank CHECK (
        subject_type IS NULL OR length(btrim(subject_type)) > 0
    ),
    CONSTRAINT event_activity_entries_subject_type_length CHECK (
        subject_type IS NULL OR length(subject_type) <= 80
    ),
    CONSTRAINT event_activity_entries_message_not_blank CHECK (
        length(btrim(message)) > 0
    ),
    CONSTRAINT event_activity_entries_message_length CHECK (
        length(message) <= 500
    )
);

CREATE INDEX event_activity_entries_event_created_at_idx
    ON event_activity_entries (event_id, created_at DESC);

CREATE INDEX event_activity_entries_actor_created_at_idx
    ON event_activity_entries (actor_user_id, created_at DESC)
    WHERE actor_user_id IS NOT NULL;

CREATE INDEX event_activity_entries_type_created_at_idx
    ON event_activity_entries (activity_type, created_at DESC);
