use std::time::Instant;

use anyhow::Result;

use crate::{
    client::BenchmarkClient,
    stats::BenchmarkResult,
};

pub async fn run() -> Result<()> {
    let client = BenchmarkClient::new();

    // Seed balances once
    client.deposit(1, "SOL", 1_000_000).await?;

    let mut samples = Vec::new();

    for i in 1..=100 {
        let start = Instant::now();

        client
            .place_order(
                i,
                1,
                "Sell",
                100,
                1,
                i,
            )
            .await?;

        samples.push(start.elapsed());
    }

    BenchmarkResult {
        name: "Place Order".to_string(),
        samples,
    }
    .print();

    Ok(())
}