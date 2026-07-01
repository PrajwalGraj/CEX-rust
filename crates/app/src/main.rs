use domain::{Asset, Market, Order, Side, Trade};
use exchange::Exchange;

#[tokio::main]
async fn main(){

    let mut exchange = Exchange::new().await;
    
    exchange.deposit(2, Asset::USDC, 1000).await;
    exchange.deposit(1, Asset::SOL, 10).await;

    print_user_balances(&exchange, 1, "Alice").await;
    print_user_balances(&exchange, 2, "Bob").await;

    let sol_usdc = Market {
        base: Asset::SOL,
        quote: Asset::USDC,
    };

    let _ = exchange.submit_order(Order::new_limit(1, 1, Side::Sell, sol_usdc, 100, 3, 1)).await;
    let _ = exchange.submit_order(Order::new_limit(2, 1, Side::Sell, sol_usdc, 102, 2, 2)).await;

    let trades = exchange.submit_order(Order::new_limit(3, 2, Side::Buy, sol_usdc, 105, 6, 3)).await.unwrap();
    print_trades(&trades);

    print_user_balances(&exchange, 1, "Alice").await;
    print_user_balances(&exchange, 2, "Bob").await;

    print_order_book(&exchange, &sol_usdc).await;
}

async fn print_user_balances(exchange: &Exchange, user_id: u64, name: &str) {
    let assets = [Asset::BTC, Asset::SOL, Asset::USDC];
    println!("==========================");
    println!("{} ({})", name, user_id);
    println!("==========================\n");
    for asset in assets {
        println!("{:?}", asset);
        match exchange.get_balance(user_id, asset).await {
            Some(balance) => {
                println!("  Available : {}", balance.available);
                println!("  Locked    : {}\n", balance.locked);
            }
            None => {
                println!("  Available : 0");
                println!("  Locked    : 0\n");
            }
        }
    }
}

fn print_trades(trades: &[Trade]) {
    println!("==========================");
    println!("Trades Executed");
    println!("==========================\n");
    for trade in trades {
        println!("Trade #{}\n", trade.trade_id);
        println!("Buyer : {}", trade.buyer_user_id);
        println!("Seller: {}\n", trade.seller_user_id);
        println!("Market: {:?}/{:?}\n", trade.market.base, trade.market.quote);
        println!("Price : {}", trade.price);
        println!("Qty   : {}\n", trade.quantity);
        println!("--------------------------");
    }
}

async fn print_order_book(exchange: &Exchange, market: &Market) {
    println!("==========================");
    println!("Final Order Book");
    println!("==========================\n");
    println!("Best Bid: {:?}", exchange.best_bid(market).await);
    println!("Best Ask: {:?}\n", exchange.best_ask(market).await);
}
