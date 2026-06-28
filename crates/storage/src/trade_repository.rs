use domain::Trade;
use sqlx::{Pool, Postgres};

pub struct TradeRepository {
    pool: Pool<Postgres>,
}

impl TradeRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn save_trade(
        &self,
        trade: &Trade,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO trades (
                trade_id,
                maker_order_id,
                taker_order_id,
                maker_user_id,
                taker_user_id,
                buyer_user_id,
                seller_user_id,
                base_asset,
                quote_asset,
                price,
                quantity
            )
            VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11
            )
            "#,
        )
        .bind(trade.trade_id as i64)
        .bind(trade.maker_order_id.0 as i64)
        .bind(trade.taker_order_id.0 as i64)
        .bind(trade.maker_user_id as i64)
        .bind(trade.taker_user_id as i64)
        .bind(trade.buyer_user_id as i64)
        .bind(trade.seller_user_id as i64)
        .bind(trade.market.base.to_string())
        .bind(trade.market.quote.to_string())
        .bind(trade.price as i64)
        .bind(trade.quantity as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}