mod models;
use std::sync::Arc;
use axum::extract::Path;


use axum::{
    Router, extract::{Json, State}, http::request, routing::{get, post},
};
use domain::{Asset, Market, Order};
use crate::models::{ ApiResponse, BalanceResponse, DepositRequest, MarketResponse, OrderBookResponse, PlaceOrderRequest };

use tokio::sync::Mutex;
use exchange::Exchange;

async fn health() -> &'static str {
    "AstraX is running"
}

#[tokio::main]
async fn main() {
    let exchange = Arc::new(Mutex::new(Exchange::new().await));

    let app = Router::new()
        .route("/", get(health))
        .route("/deposit", post(deposit))
        .route("/orders", post(place_order))
        .route("/balances/{user_id}", get(get_balance))
        .route("/book/{market}", get(get_order_book))
        .route("/markets", get(get_markets))
        .with_state(exchange);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn deposit(
    State(exchange) : State<Arc<Mutex<Exchange>>>,
    Json(request): Json<DepositRequest>,
) -> Json<ApiResponse> {
    let mut exchange = exchange.lock().await;

    exchange.deposit(request.user_id, request.asset, request.amount).await;

    let balance = exchange.get_balance(request.user_id, request.asset).await;
    println!("{:?}",balance);

    Json(ApiResponse { status: "success".to_string() })
}

async fn place_order(
    State(exchange): State<Arc<Mutex<Exchange>>>,
    Json(request): Json<PlaceOrderRequest>
) -> Json<ApiResponse> {

    let mut exchange = exchange.lock().await;

    let order = Order::new_limit(request.order_id, request.user_id, request.side, request.market, request.price, request.quantity, request.sequence);

    let trade = exchange.submit_order(order).await;

    match trade {
        Ok(_) => Json(ApiResponse { status: "success".to_string() }),
        Err(_) => Json( ApiResponse { status: "failed".to_string()}),
    }

}

async fn get_balance(
    State(exchange) : State<Arc<Mutex<Exchange>>>,
    Path(user_id) : Path<u64>
) -> Json<BalanceResponse> {
    let mut exchange = exchange.lock().await;

    let btc = exchange.get_balance(user_id, Asset::BTC).await;
    let sol = exchange.get_balance(user_id, Asset::SOL).await;
    let usdc = exchange.get_balance(user_id, Asset::USDC).await;


    Json(BalanceResponse{
        btc,
        sol,
        usdc
    })
}


async fn get_order_book(
    State(exchange): State<Arc<Mutex<Exchange>>>,
    Path(market_name): Path<String>
) -> Json<OrderBookResponse> {
    let mut exchange = exchange.lock().await;

    let market = match market_name.as_str(){
        "SOL-USDC" => {
            Market{
                base: Asset::SOL,
                quote: Asset::USDC
            }
        },
        "BTC-USDC" => {
            Market { base: Asset::BTC, quote: Asset::USDC }
        },
        _ => panic!("invalid market"),
    };

    let best_bid = exchange.best_bid(&market).await;
    let best_ask = exchange.best_ask(&market).await;

    Json(OrderBookResponse { market: market_name, best_bid, best_ask })
}

async fn get_markets(
    State(exchange): State<Arc<Mutex<Exchange>>>,
) -> Json<Vec<MarketResponse>> {
    let exchange = exchange.lock().await;


    let markets = vec![
        Market {
            base: Asset::SOL,
            quote: Asset::USDC,
        },
        Market {
            base: Asset::BTC,
            quote: Asset::USDC,
        },
    ];

    let mut responder = Vec::new();

    for market in markets {
        let best_ask = exchange.best_ask(&market).await;
        let best_bid = exchange.best_bid(&market).await;

        responder.push(MarketResponse {
            market: format!("{:?}/{:?}", market.base, market.quote),
            best_ask,
            best_bid,
        });
    }

    Json(responder)
}