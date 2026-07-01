use std::time::Instant;

use anyhow::Result;

use crate::{
    client::BenchmarkClient,
    stats::BenchmarkResult,
};

pub async fn run() -> Result<()> {
    let client = BenchmarkClient::new();

    client.deposit(1, "SOL", 1_000_000).await?;

    let mut samples = Vec::new();

    for i in 1..=100 {

        let order_id = i;

        // Place a resting order
        client
            .place_order(
                order_id,
                1,
                "Sell",
                100,
                1,
                order_id,
            )
            .await?;

        // Measure cancellation
        let start = Instant::now();

        client
            .cancel_order(order_id)
            .await?;

        samples.push(start.elapsed());
    }

    BenchmarkResult {
        name: "Cancel Order".to_string(),
        samples,
    }
    .print();

    Ok(())
}