use domain::{Asset, Market, Order, OrderId, OrderStatus, OrderType, Side, TimeInForce, OrderUpdate };
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
        order: &OrderUpdate,
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
        .bind(order.order_id.0 as i64)
        .bind(order.remaining_qty as i64)
        .bind(order.status.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_open_orders(&self ) -> Result<Vec<Order>, sqlx::Error> {

    let rows = sqlx::query_as::<_, (i64, i64, String, String, String, String, Option<i64>, i64, i64, String, i64)>(
        r#"
        SELECT
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
        FROM orders
        WHERE status IN ('OPEN', 'PARTIALLY_FILLED')
        ORDER BY sequence
        "#
    )
    .fetch_all(&self.pool)
    .await?;

    let mut orders = Vec::new();

    for row in rows {
        let base = match row.2.as_str() {
            "BTC" => Asset::BTC,
            "SOL" => Asset::SOL,
            "USDC" => Asset::USDC,
            _ => panic!("Invalid base asset"),
        };

        let quote = match row.3.as_str() {
            "BTC" => Asset::BTC,
            "SOL" => Asset::SOL,
            "USDC" => Asset::USDC,
            _ => panic!("Invalid quote asset"),
        };

        let side = match row.4.as_str() {
            "BUY" => Side::Buy,
            "SELL" => Side::Sell,
            _ => panic!("Invalid side"),
        };

        let order_type = match row.5.as_str() {
            "LIMIT" => OrderType::Limit,
            "MARKET" => OrderType::Market,
            _ => panic!("Invalid order type"),
        };

        let status = match row.9.as_str() {
            "OPEN" => OrderStatus::Open,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELLED" => OrderStatus::Cancelled,
            _ => panic!("Invalid order status"),
        };

        let time_in_force = match order_type {
            OrderType::Limit => TimeInForce::Gtc,
            OrderType::Market => TimeInForce::Ioc,
        };

        orders.push(Order {
            id: OrderId(row.0 as u64),
            user_id: row.1 as u64,
            side,
            market: Market {
                base,
                quote,
            },
            order_type,
            time_in_force,
            limit_price: row.6.map(|p| p as u64),
            original_qty: row.7 as u64,
            remaining_qty: row.8 as u64,
            status,
            sequence: row.10 as u64,
        });
    }

    Ok(orders)
}
}
