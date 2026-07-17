CREATE TABLE IF NOT EXISTS event_reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    invitation_id UUID NOT NULL REFERENCES invitations(id) ON DELETE CASCADE,
    reminder_kind TEXT NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT event_reminders_kind_not_blank CHECK (length(trim(reminder_kind)) > 0),
    CONSTRAINT event_reminders_kind_length CHECK (length(reminder_kind) <= 80),
    CONSTRAINT event_reminders_unique_invitation_kind UNIQUE (invitation_id, reminder_kind)
);

CREATE INDEX IF NOT EXISTS event_reminders_event_sent_at_idx
    ON event_reminders (event_id, sent_at DESC);
