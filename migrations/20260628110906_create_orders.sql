-- Add migration script here
CREATE TABLE orders (
    id BIGINT PRIMARY KEY,

    user_id BIGINT NOT NULL,

    base_asset TEXT NOT NULL,
    quote_asset TEXT NOT NULL,

    side TEXT NOT NULL,
    order_type TEXT NOT NULL,

    limit_price BIGINT,

    original_qty BIGINT NOT NULL,
    remaining_qty BIGINT NOT NULL,

    status TEXT NOT NULL,

    sequence BIGINT NOT NULL
);