use crate::{Asset, OrderId, OrderStatus, Trade};

#[derive(Debug, Clone)]
pub struct BalanceUpdate {
    pub user_id: u64,
    pub asset: Asset,

    pub available_delta: i64,
    pub locked_delta: i64,
}

#[derive(Debug, Clone)]
pub struct OrderUpdate {
    pub order_id: OrderId,

    pub remaining_qty: u64,
    pub status: OrderStatus,
}

#[derive(Debug, Clone)]
pub struct SettlementBatch {
    pub balance_updates: Vec<BalanceUpdate>,
    pub order_updates: Vec<OrderUpdate>,
    pub trades: Vec<Trade>,
}