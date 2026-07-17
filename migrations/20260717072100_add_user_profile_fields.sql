ALTER TABLE users
    ADD COLUMN bio TEXT,
    ADD COLUMN location TEXT,
    ADD COLUMN website_url TEXT,
    ADD CONSTRAINT users_display_name_length CHECK (
        display_name IS NULL OR length(display_name) <= 120
    ),
    ADD CONSTRAINT users_full_name_length CHECK (
        full_name IS NULL OR length(full_name) <= 120
    ),
    ADD CONSTRAINT users_bio_length CHECK (
        bio IS NULL OR length(bio) <= 500
    ),
    ADD CONSTRAINT users_location_length CHECK (
        location IS NULL OR length(location) <= 120
    ),
    ADD CONSTRAINT users_website_url_length CHECK (
        website_url IS NULL OR length(website_url) <= 2048
    );
