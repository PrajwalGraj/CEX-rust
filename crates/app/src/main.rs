use domain::{Asset, Market, Order, Side};
use engine::OrderBook;

fn main() {

    let sol_usdc = Market {
        base: Asset::SOL,
        quote: Asset::USDC,
    };
    
    let mut book = OrderBook::new();

    // Resting Sell #1
    book.submit_order(Order::new_limit(
        1,
        101,
        Side::Sell,
        sol_usdc,
        100,
        3,
        1,
    ));

    // Resting Sell #2
    book.submit_order(Order::new_limit(
        2,
        102,
        Side::Sell,
        sol_usdc,
        102,
        5,
        2,
    ));

    println!("Initial Order Book");
    println!("Best Bid: {:?}", book.best_bid());
    println!("Best Ask: {:?}", book.best_ask());

    println!("\nSubmitting Buy Order...\n");

    let trades = book.submit_order(Order::new_limit(
        3,
        201,
        Side::Buy,
        sol_usdc,
        105,
        6,
        3,
    ));

    println!("Trades Executed:");

    for trade in trades {
        println!("{:#?}", trade);
    }

    println!("\nOrder Book After Matching");
    println!("Best Bid: {:?}", book.best_bid());
    println!("Best Ask: {:?}", book.best_ask());
}