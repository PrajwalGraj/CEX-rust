use reqwest::Client;
use anyhow::Result;
use serde_json::json;

pub struct BenchmarkClient {
    pub client: Client,
    pub base_url: String,
}

impl BenchmarkClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:3000".to_string(),
        }
    }
    pub async fn deposit(
        &self,
        user_id: u64,
        asset: &str,
        amount: u64,
    ) -> Result<()> {
        self.client
            .post(format!("{}/deposit", self.base_url))
            .json(&json!({
                "user_id": user_id,
                "asset": asset,
                "amount": amount
            }))
            .send()
            .await?;

        Ok(())
    }

    pub async fn place_order(
        &self,
        order_id: u64,
        user_id: u64,
        side: &str,
        price: u64,
        quantity: u64,
        sequence: u64,
    ) -> Result<()> {
        self.client
            .post(format!("{}/orders", self.base_url))
            .json(&json!({
                "order_id": order_id,
                "user_id": user_id,
                "market": {
                    "base": "SOL",
                    "quote": "USDC"
                },
                "side": side,
                "price": price,
                "quantity": quantity,
                "sequence": sequence
            }))
            .send()
            .await?;

        Ok(())
    }

    pub async fn cancel_order(&self, order_id: u64 ) -> anyhow::Result<()> {
        self.client
            .post(format!(
                "{}/orders/{}/cancel",
                self.base_url,
                order_id
            ))
            .json(&serde_json::json!({
                "market": {
                    "base": "SOL",
                    "quote": "USDC"
                }
            }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}