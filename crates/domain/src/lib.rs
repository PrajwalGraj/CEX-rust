use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Asset {
    BTC,
    USDC,
    SOL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    Gtc,
    Ioc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
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
        market : Market,
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

    pub fn new_market(id: u64, user_id: u64, side: Side, market: Market, quantity: u64, sequence: u64) -> Self {
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


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Market {
    pub base: Asset,
    pub quote: Asset,
}