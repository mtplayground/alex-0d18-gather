CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organizer_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ,
    timezone TEXT,
    location_name TEXT,
    location_address TEXT,
    cover_image_object_key TEXT,
    pdf_attachment_object_keys TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT events_title_not_blank CHECK (length(btrim(title)) > 0),
    CONSTRAINT events_title_length CHECK (length(title) <= 200),
    CONSTRAINT events_description_length CHECK (
        description IS NULL OR length(description) <= 5000
    ),
    CONSTRAINT events_ends_after_start CHECK (
        ends_at IS NULL OR ends_at > starts_at
    ),
    CONSTRAINT events_timezone_length CHECK (
        timezone IS NULL OR length(timezone) <= 100
    ),
    CONSTRAINT events_location_name_length CHECK (
        location_name IS NULL OR length(location_name) <= 200
    ),
    CONSTRAINT events_location_address_length CHECK (
        location_address IS NULL OR length(location_address) <= 500
    ),
    CONSTRAINT events_cover_key_not_blank CHECK (
        cover_image_object_key IS NULL OR length(btrim(cover_image_object_key)) > 0
    ),
    CONSTRAINT events_cover_key_relative CHECK (
        cover_image_object_key IS NULL OR cover_image_object_key NOT LIKE '/%'
    ),
    CONSTRAINT events_pdf_attachment_count CHECK (
        cardinality(pdf_attachment_object_keys) <= 20
    ),
    CONSTRAINT events_pdf_attachment_keys_no_null CHECK (
        array_position(pdf_attachment_object_keys, NULL) IS NULL
    ),
    CONSTRAINT events_pdf_attachment_keys_not_blank CHECK (
        array_position(pdf_attachment_object_keys, '') IS NULL
    )
);

CREATE INDEX events_organizer_starts_at_idx
    ON events (organizer_user_id, starts_at DESC);

CREATE INDEX events_starts_at_idx ON events (starts_at DESC);
