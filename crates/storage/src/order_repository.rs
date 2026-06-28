use domain::Order;
use sqlx::{Pool, Postgres};

pub struct OrderRepository {
    pool: Pool<Postgres>,
}

impl OrderRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn save_order(
        &self,
        order: &Order,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO orders (
                id,
                user_id,
                base_asset,
                quote_asset,
                side,
                order_type,
                limit_price,
                original_qty,
                remaining_qty,
                status,
                sequence
            )
            VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11
            )
            "#,
        )
        .bind(order.id.0 as i64)
        .bind(order.user_id as i64)
        .bind(order.market.base.to_string())
        .bind(order.market.quote.to_string())
        .bind(order.side.to_string())
        .bind(order.order_type.to_string())
        .bind(order.limit_price.map(|p| p as i64))
        .bind(order.original_qty as i64)
        .bind(order.remaining_qty as i64)
        .bind(order.status.to_string())
        .bind(order.sequence as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_order(
        &self,
        order: &Order,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE orders
            SET
                remaining_qty = $2,
                status = $3
            WHERE id = $1
            "#,
        )
        .bind(order.id.0 as i64)
        .bind(order.remaining_qty as i64)
        .bind(order.status.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}