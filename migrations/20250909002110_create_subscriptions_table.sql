-- Create Subscriptions Table
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    person_name TEXT NOT NULL,
    subscribed_at TIMESTAMPTZ NOT NULL
);
