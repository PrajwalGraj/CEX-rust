mod models;
use std::sync::Arc;

use axum::{
    Router, extract::{Json, State}, http::request, routing::{get, post},
};
use domain::Order;
use crate::models::{ ApiResponse, DepositRequest, PlaceOrderRequest };

use tokio::sync::Mutex;
use exchange::Exchange;

async fn health() -> &'static str {
    "AstraX is running"
}

#[tokio::main]
async fn main() {
    let exchange = Arc::new(Mutex::new(Exchange::new()));

    let app = Router::new()
        .route("/", get(health))
        .route("/deposit", post(deposit))
        .route("/orders", post(place_order))
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