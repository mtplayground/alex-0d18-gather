ALTER TABLE invitations
    ADD COLUMN rsvp_status TEXT,
    ADD COLUMN rsvp_responded_at TIMESTAMPTZ,
    ADD CONSTRAINT invitations_rsvp_status_valid CHECK (
        rsvp_status IS NULL OR rsvp_status IN ('yes', 'no', 'maybe')
    ),
    ADD CONSTRAINT invitations_rsvp_responded_at_consistent CHECK (
        (rsvp_status IS NULL AND rsvp_responded_at IS NULL)
        OR (rsvp_status IS NOT NULL AND rsvp_responded_at IS NOT NULL)
    );

CREATE INDEX invitations_event_rsvp_status_idx
    ON invitations (event_id, rsvp_status, rsvp_responded_at DESC)
    WHERE rsvp_status IS NOT NULL;
