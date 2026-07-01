use serde::{Deserialize, Serialize};
use domain::Asset;
use domain::{Market, OrderBookLevel, Side};
use balance::Balance;

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub user_id: u64,
    pub asset: Asset,
    pub amount: u64,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct PlaceOrderRequest {
    pub order_id: u64,
    pub user_id: u64,
    pub market: Market,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
    pub sequence: u64,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub btc: Option<Balance>,
    pub sol: Option<Balance>,
    pub usdc: Option<Balance>,
}

#[derive(Debug, Serialize)]
pub struct OrderBookResponse {
    pub market: String,
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
}

#[derive(Debug, Serialize)]
pub struct MarketResponse {
    pub market: String,
    pub best_bid: Option<u64>,
    pub best_ask: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
    pub market: Market,
}
