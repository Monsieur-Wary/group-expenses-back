CREATE TABLE users(
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE CHECK (char_length(email) > 0),
    password TEXT NOT NULL CHECK (char_length(email) > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);