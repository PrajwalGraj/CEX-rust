-- Add migration script here
CREATE TABLE balances (
    user_id BIGINT NOT NULL,
    asset TEXT NOT NULL,

    available BIGINT NOT NULL DEFAULT 0,
    locked BIGINT NOT NULL DEFAULT 0,

    PRIMARY KEY (user_id, asset)
);