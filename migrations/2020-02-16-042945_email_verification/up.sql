CREATE TABLE user_verification (
    user_id BIGINT PRIMARY KEY REFERENCES users(user_id),
    token TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
