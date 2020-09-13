CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE CHECK (char_length(email) > 0),
    password VARCHAR(255) NOT NULL CHECK (char_length(email) > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)