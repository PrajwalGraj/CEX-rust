use domain::SettlementBatch;
use sqlx::{Pool, Postgres};

pub struct SettlementRepository {
    pool: Pool<Postgres>,
}

impl SettlementRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn persist(
        &self,
        batch: SettlementBatch,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        for update in batch.balance_updates {
            sqlx::query(
                r#"
                INSERT INTO balances (user_id, asset, available, locked)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (user_id, asset)
                DO UPDATE
                SET
                    available = balances.available + EXCLUDED.available,
                    locked = balances.locked + EXCLUDED.locked
                "#,
            )
            .bind(update.user_id as i64)
            .bind(update.asset.to_string())
            .bind(update.available_delta)
            .bind(update.locked_delta)
            .execute(&mut *tx)
            .await?;
        }

        for update in batch.order_updates {
            sqlx::query(
                r#"
                UPDATE orders
                SET
                    remaining_qty = $2,
                    status = $3
                WHERE id = $1
                "#,
            )
            .bind(update.order_id.0 as i64)
            .bind(update.remaining_qty as i64)
            .bind(update.status.to_string())
            .execute(&mut *tx)
            .await?;
        }

        for trade in batch.trades {
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
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
