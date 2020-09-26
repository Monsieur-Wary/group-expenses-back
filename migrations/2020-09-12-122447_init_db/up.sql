CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR(100) NOT NULL UNIQUE CHECK (char_length(email) > 0),
    password VARCHAR(255) NOT NULL CHECK (char_length(email) > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS dashboards (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS persons (
    id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    resources INT NOT NULL CHECK (resources >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (dashboard_id) REFERENCES dashboards(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS expenses (
    id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL,
    person_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    amount INT NOT NULL CHECK (amount > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (dashboard_id) REFERENCES dashboards(id) ON DELETE CASCADE,
    FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE
);