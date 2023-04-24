CREATE TABLE event_tracking (
    id SERIAL PRIMARY KEY,
    url VARCHAR(255) NOT NULL,
    referrer VARCHAR(255) NOT NULL,
    user_agent VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
