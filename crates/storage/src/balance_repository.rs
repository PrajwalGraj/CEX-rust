use domain::Asset;
use sqlx::{Pool, Postgres};
use balance::Balance;

pub struct BalanceRepository {
    pool: Pool<Postgres>,
}

impl BalanceRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
    
    pub async fn deposit(
        &self,
        user_id: u64,
        asset: Asset,
        amount: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO balances (user_id, asset, available)
            VALUES ($1, $2, $3)

            ON CONFLICT (user_id, asset)
            DO UPDATE
            SET available = balances.available + EXCLUDED.available
            "#,
        )
        .bind(user_id as i64)
        .bind(asset.to_string())
        .bind(amount as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_balance(
        &self,
        user_id: u64,
        asset: Asset,
    ) -> Result<Option<Balance>, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64, i64)>(
            r#"
            SELECT available, locked
            FROM balances
            WHERE user_id = $1
            AND asset = $2
            "#,
        )
        .bind(user_id as i64)
        .bind(asset.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((available, locked)) => Ok(Some(Balance {
                available: available as u64,
                locked: locked as u64,
            })),
            None => Ok(None),
        }
    }

    pub async fn lock(
        &self,
        user_id: u64,
        asset: Asset,
        amount: u64,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE balances
            SET
                available = available - $3,
                locked = locked + $3
            WHERE
                user_id = $1
                AND asset = $2
                AND available >= $3
            "#,
        )
        .bind(user_id as i64)
        .bind(asset.to_string())
        .bind(amount as i64)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn unlock(
        &self,
        user_id: u64,
        asset: Asset,
        amount: u64,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE balances
            SET
                available = available + $3,
                locked = locked - $3
            WHERE
                user_id = $1
                AND asset = $2
                AND locked >= $3
            "#,
        )
        .bind(user_id as i64)
        .bind(asset.to_string())
        .bind(amount as i64)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn debit_locked(
        &self,
        user_id: u64,
        asset: Asset,
        amount: u64,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE balances
            SET
                locked = locked - $3
            WHERE
                user_id = $1
                AND asset = $2
                AND locked >= $3
            "#,
        )
        .bind(user_id as i64)
        .bind(asset.to_string())
        .bind(amount as i64)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() == 1)
    }

    pub async fn credit_available(
        &self,
        user_id: u64,
        asset: Asset,
        amount: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO balances (user_id, asset, available)
            VALUES ($1, $2, $3)

            ON CONFLICT (user_id, asset)
            DO UPDATE
            SET available = balances.available + EXCLUDED.available
            "#,
        )
        .bind(user_id as i64)
        .bind(asset.to_string())
        .bind(amount as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
