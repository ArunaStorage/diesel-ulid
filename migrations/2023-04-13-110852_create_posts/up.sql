-- Your SQL goes here
CREATE TABLE posts (
    id UUID PRIMARY KEY,
    body TEXT NOT NULL,
    published BOOLEAN NOT NULL DEFAULT FALSE
)