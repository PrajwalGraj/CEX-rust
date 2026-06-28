-- Add migration script here
CREATE TABLE trades (
    trade_id BIGINT PRIMARY KEY,

    maker_order_id BIGINT NOT NULL,
    taker_order_id BIGINT NOT NULL,

    maker_user_id BIGINT NOT NULL,
    taker_user_id BIGINT NOT NULL,

    buyer_user_id BIGINT NOT NULL,
    seller_user_id BIGINT NOT NULL,

    base_asset TEXT NOT NULL,
    quote_asset TEXT NOT NULL,

    price BIGINT NOT NULL,
    quantity BIGINT NOT NULL
);