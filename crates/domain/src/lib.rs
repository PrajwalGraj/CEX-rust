use std::fmt;

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Asset {
    BTC,
    USDC,
    SOL,
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Asset::BTC => write!(f, "BTC"),
            Asset::SOL => write!(f, "SOL"),
            Asset::USDC => write!(f, "USDC"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq , Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderType::Limit => write!(f, "LIMIT"),
            OrderType::Market => write!(f, "MARKET"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq , Serialize, Deserialize)]
pub enum TimeInForce {
    Gtc,
    Ioc,
}
impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeInForce::Gtc => write!(f, "GTC"),
            TimeInForce::Ioc => write!(f, "IOC"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderStatus::Open => write!(f, "OPEN"),
            OrderStatus::PartiallyFilled => write!(f, "PARTIALLY_FILLED"),
            OrderStatus::Filled => write!(f, "FILLED"),
            OrderStatus::Cancelled => write!(f, "CANCELLED"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_id: u64,
    pub maker_order_id: OrderId,
    pub taker_order_id: OrderId,
    pub maker_user_id: u64,
    pub taker_user_id: u64,
    pub price: u64,
    pub quantity: u64,
    pub market: Market,

    pub buyer_user_id: u64,
    pub seller_user_id: u64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash , Serialize, Deserialize)]
pub struct Market {
    pub base: Asset,
    pub quote: Asset,
}

#[derive(Debug)]
pub struct MatchResult {
    pub trades: Vec<Trade>,
    pub new_orders: Vec<Order>,
    pub updated_orders: Vec<Order>,
}


#[derive(Debug, Clone, Serialize)]
pub struct OrderBookLevel {
    pub price: u64,
    pub quantity: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderBookSnapshot {
    pub bids: Vec<OrderBookLevel>,
    pub asks: Vec<OrderBookLevel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "order-{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub user_id: u64,

    pub side: Side,
    pub market: Market,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,

    // For limit orders: Some(price).
    // For market orders: None.
    pub limit_price: Option<u64>,

    pub original_qty: u64,
    pub remaining_qty: u64,

    pub status: OrderStatus,
    pub sequence: u64,
}

impl Order {
    pub fn new_limit(
        id: u64,
        user_id: u64,
        side: Side,
        market: Market,
        price: u64,
        quantity: u64,
        sequence: u64,
    ) -> Self {
        Self {
            id: OrderId(id),
            user_id,
            side,
            market,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::Gtc,
            limit_price: Some(price),
            original_qty: quantity,
            remaining_qty: quantity,
            status: OrderStatus::Open,
            sequence,
        }
    }

    pub fn new_market(
        id: u64,
        user_id: u64,
        side: Side,
        market: Market,
        quantity: u64,
        sequence: u64,
    ) -> Self {
        Self {
            id: OrderId(id),
            user_id,
            side,
            market,
            order_type: OrderType::Market,
            time_in_force: TimeInForce::Ioc,
            limit_price: None,
            original_qty: quantity,
            remaining_qty: quantity,
            status: OrderStatus::Open,
            sequence,
        }
    }
}
