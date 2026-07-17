CREATE TABLE invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    invitee_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    invitee_email TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    share_token TEXT NOT NULL DEFAULT encode(gen_random_bytes(32), 'hex'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT invitations_invitee_present CHECK (
        invitee_user_id IS NOT NULL OR invitee_email IS NOT NULL
    ),
    CONSTRAINT invitations_invitee_email_not_blank CHECK (
        invitee_email IS NULL OR length(btrim(invitee_email)) > 0
    ),
    CONSTRAINT invitations_invitee_email_length CHECK (
        invitee_email IS NULL OR length(invitee_email) <= 320
    ),
    CONSTRAINT invitations_status_valid CHECK (
        status IN ('pending', 'accepted', 'declined', 'revoked')
    ),
    CONSTRAINT invitations_share_token_not_blank CHECK (
        length(btrim(share_token)) > 0
    ),
    CONSTRAINT invitations_share_token_length CHECK (
        length(share_token) BETWEEN 32 AND 128
    )
);

CREATE UNIQUE INDEX invitations_share_token_unique_idx
    ON invitations (share_token);

CREATE UNIQUE INDEX invitations_event_invitee_user_unique_idx
    ON invitations (event_id, invitee_user_id)
    WHERE invitee_user_id IS NOT NULL;

CREATE UNIQUE INDEX invitations_event_invitee_email_unique_idx
    ON invitations (event_id, lower(invitee_email))
    WHERE invitee_email IS NOT NULL;

CREATE INDEX invitations_event_status_idx
    ON invitations (event_id, status, created_at DESC);

CREATE INDEX invitations_invitee_user_idx
    ON invitations (invitee_user_id, created_at DESC)
    WHERE invitee_user_id IS NOT NULL;
