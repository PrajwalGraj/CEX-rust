use serde::{Deserialize, Serialize};
use domain::Asset;
use domain::{Market, Side};

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