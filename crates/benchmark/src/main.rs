use anyhow::Result;

mod client;
mod stats;
mod place_order;
mod match_order;
mod cancel_order;

#[tokio::main]
async fn main() -> Result<()> {
    println!("==============================");
    println!("      CEX Benchmarks");
    println!("==============================");

    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("place") => {
            place_order::run().await?;
        }
        Some("match") => {
            match_order::run().await?;
        }
        Some("cancel") => {
            cancel_order::run().await?;
        }
        _ => {
            println!("Usage:");
            println!("cargo run -p benchmark -- place");
            println!("cargo run -p benchmark -- match");
            println!("cargo run -p benchmark -- cancel");
        }
    }

    Ok(())
}