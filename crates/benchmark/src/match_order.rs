use std::time::Instant;

use anyhow::Result;

use crate::{
    client::BenchmarkClient,
    stats::BenchmarkResult,
};

pub async fn run() -> Result<()> {
    let client = BenchmarkClient::new();

    client.deposit(1, "SOL", 1_000_000).await?;
    client.deposit(2, "USDC", 1_000_000).await?;

    let mut samples = Vec::new();

    for i in 1..=100 {

        // Resting SELL order
        client
            .place_order(
                i * 2 - 1,
                1,
                "Sell",
                100,
                1,
                i * 2 - 1,
            )
            .await?;

        // Measure BUY order (this triggers matching)
        let start = Instant::now();

        client
            .place_order(
                i * 2,
                2,
                "Buy",
                100,
                1,
                i * 2,
            )
            .await?;

        samples.push(start.elapsed());
    }

    BenchmarkResult {
        name: "Match Order".to_string(),
        samples,
    }
    .print();

    Ok(())
}