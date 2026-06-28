use domain::{Asset, Market, Order, OrderId, OrderStatus, OrderType, Side, TimeInForce};
use sqlx::{Pool, Postgres};
mod balance_repository;
mod order_repository;
mod trade_repository;

pub use trade_repository::TradeRepository;
pub use order_repository::OrderRepository;
pub use balance_repository::BalanceRepository;

#[derive(Debug)]
pub struct Database {
    pool: Pool<Postgres>,
}
impl Database {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::PgPool::connect(database_url).await?;

        Ok(Self { pool })
    }
    pub fn pool(&self) -> Pool<Postgres> {
        self.pool.clone()
    }
}


#[cfg(test)]
async fn test_db() -> Database {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    Database::connect(&database_url)
        .await
        .unwrap()
}


#[tokio::test]
async fn can_store_and_get_balance() {
    let db = test_db().await;

    let repo = BalanceRepository::new(db.pool());

    let user_id = 10001;

    repo.deposit(user_id, Asset::SOL, 100).await.unwrap();

    let balance = repo.get_balance(user_id, Asset::SOL).await.unwrap().unwrap();

    assert_eq!(balance.available, 100);
    assert_eq!(balance.locked, 0);
}

#[tokio::test]
async fn can_lock_balance() {
    let db = test_db().await;

    let repo = BalanceRepository::new(db.pool());

    let user_id = 10002;

    repo.deposit(user_id, Asset::SOL, 100).await.unwrap();

    assert!(repo.lock(user_id, Asset::SOL, 40).await.unwrap());

    let balance = repo.get_balance(user_id, Asset::SOL).await.unwrap().unwrap();

    assert_eq!(balance.available, 60);
    assert_eq!(balance.locked, 40);
}

#[tokio::test]
async fn can_unlock_balance() {
    let db = test_db().await;

    let repo = BalanceRepository::new(db.pool());

    let user_id = 10003;

    repo.deposit(user_id, Asset::SOL, 100).await.unwrap();

    repo.lock(user_id, Asset::SOL, 50).await.unwrap();

    assert!(repo.unlock(user_id, Asset::SOL, 20).await.unwrap());

    let balance = repo.get_balance(user_id, Asset::SOL).await.unwrap().unwrap();

    assert_eq!(balance.available, 70);
    assert_eq!(balance.locked, 30);
}

#[tokio::test]
async fn can_debit_locked_balance() {
    let db = test_db().await;

    let repo = BalanceRepository::new(db.pool());

    let user_id = 10004;

    repo.deposit(user_id, Asset::SOL, 100).await.unwrap();

    repo.lock(user_id, Asset::SOL, 80).await.unwrap();

    assert!(repo.debit_locked(user_id, Asset::SOL, 30).await.unwrap());

    let balance = repo.get_balance(user_id, Asset::SOL).await.unwrap().unwrap();

    assert_eq!(balance.available, 20);
    assert_eq!(balance.locked, 50);
}

#[tokio::test]
async fn can_credit_available_balance() {
    let db = test_db().await;

    let repo = BalanceRepository::new(db.pool());

    let user_id = 10005;

    repo.credit_available(user_id, Asset::SOL, 250).await.unwrap();

    let balance = repo.get_balance(user_id, Asset::SOL).await.unwrap().unwrap();

    assert_eq!(balance.available, 250);
    assert_eq!(balance.locked, 0);
}

#[tokio::test]
async fn lock_should_fail_when_insufficient_balance() {
    let db = test_db().await;

    let repo = BalanceRepository::new(db.pool());

    let user_id = 10006;

    repo.deposit(user_id, Asset::SOL, 50).await.unwrap();

    let success = repo.lock(user_id, Asset::SOL, 100).await.unwrap();

    assert!(!success);

    let balance = repo.get_balance(user_id, Asset::SOL).await.unwrap().unwrap();

    assert_eq!(balance.available, 50);
    assert_eq!(balance.locked, 0);
}

#[tokio::test]
async fn unlock_should_fail_when_insufficient_locked_balance() {
    let db = test_db().await;

    let repo = BalanceRepository::new(db.pool());

    let user_id = 10007;

    repo.deposit(user_id, Asset::SOL, 100).await.unwrap();

    repo.lock(user_id, Asset::SOL, 20).await.unwrap();

    let success = repo.unlock(user_id, Asset::SOL, 50).await.unwrap();

    assert!(!success);

    let balance = repo.get_balance(user_id, Asset::SOL).await.unwrap().unwrap();

    assert_eq!(balance.available, 80);
    assert_eq!(balance.locked, 20);
}

#[tokio::test]
async fn can_save_order() {
    let db = test_db().await;

    let repo = OrderRepository::new(db.pool());

    let order = Order {
        id: OrderId(10001),
        user_id: 1,
        side: Side::Buy,
        market: Market {
            base: Asset::SOL,
            quote: Asset::USDC,
        },
        order_type: OrderType::Limit,
        time_in_force: TimeInForce::Gtc,
        limit_price: Some(100),
        original_qty: 10,
        remaining_qty: 10,
        status: OrderStatus::Open,
        sequence: 1,
    };

    repo.save_order(&order).await.unwrap();

    let row = sqlx::query_as::<_, (i64, i64, String)>(
        r#"
        SELECT remaining_qty, original_qty, status
        FROM orders
        WHERE id = $1
        "#,
    )
    .bind(order.id.0 as i64)
    .fetch_one(&db.pool())
    .await
    .unwrap();

    assert_eq!(row.0, 10);
    assert_eq!(row.1, 10);
    assert_eq!(row.2, "OPEN");
}

#[tokio::test]
async fn can_update_order() {
    let db = test_db().await;

    let repo = OrderRepository::new(db.pool());

    let mut order = Order {
        id: OrderId(10002),
        user_id: 1,
        side: Side::Buy,
        market: Market {
            base: Asset::SOL,
            quote: Asset::USDC,
        },
        order_type: OrderType::Limit,
        time_in_force: TimeInForce::Gtc,
        limit_price: Some(100),
        original_qty: 10,
        remaining_qty: 10,
        status: OrderStatus::Open,
        sequence: 2,
    };

    repo.save_order(&order).await.unwrap();

    order.remaining_qty = 4;
    order.status = OrderStatus::PartiallyFilled;

    repo.update_order(&order).await.unwrap();

    let row = sqlx::query_as::<_, (i64, String)>(
        r#"
        SELECT remaining_qty, status
        FROM orders
        WHERE id = $1
        "#,
    )
    .bind(order.id.0 as i64)
    .fetch_one(&db.pool())
    .await
    .unwrap();

    assert_eq!(row.0, 4);
    assert_eq!(row.1, "PARTIALLY_FILLED");
}