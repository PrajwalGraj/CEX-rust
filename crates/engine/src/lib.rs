use domain::{Order, OrderStatus, OrderType, Side, Trade};
use std::cmp::Reverse;
use std::collections::{BTreeMap, VecDeque};

pub struct OrderBook {
    bids: BTreeMap<Reverse<u64>, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
    next_trade_id: u64,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            next_trade_id: 1,
        }
    }

    fn take_next_trade_id(&mut self) -> u64 {
        let trade_id = self.next_trade_id;
        self.next_trade_id += 1;
        trade_id
    }

    pub fn add_resting_order(&mut self, order: Order) {
        match order.order_type {
            OrderType::Limit => {
                let price = order
                    .limit_price
                    .expect("Resting order must have a limit price");
                match order.side {
                    Side::Buy => {
                        self.bids
                            .entry(Reverse(price))
                            .or_default()
                            .push_back(order);
                    }
                    Side::Sell => {
                        self.asks.entry(price).or_default().push_back(order);
                    }
                }
            }
            OrderType::Market => {
                panic!("Market orders cannot rest on the order book");
            }
        }
    }

    pub fn best_bid(&self) -> Option<u64> {
        self.bids.first_key_value().map(|(price, _queue)| price.0)
    }

    pub fn best_ask(&self) -> Option<u64> {
        self.asks.first_key_value().map(|(price, _queue)| *price)
    }

    pub fn can_match(&self, incoming_order: &Order) -> bool {
        match incoming_order.order_type {
            OrderType::Market => {
                return self.opposite_best_price(incoming_order.side).is_some();
            }
            OrderType::Limit => match incoming_order.side {
                Side::Buy => {
                    if let Some(incoming_price) = incoming_order.limit_price {
                        if let Some(best_price) = self.opposite_best_price(Side::Buy) {
                            if incoming_price >= best_price {
                                return true;
                            } else {
                                return false;
                            }
                        }
                    }
                }
                Side::Sell => {
                    if let Some(incoming_price) = incoming_order.limit_price {
                        if let Some(best_price) = self.opposite_best_price(Side::Sell) {
                            if incoming_price <= best_price {
                                return true;
                            } else {
                                return false;
                            }
                        }
                    }
                }
            },
        }
        false
    }

    pub fn opposite_best_price(&self, side: Side) -> Option<u64> {
        match side {
            Side::Buy => {
                return self.best_ask();
            }
            Side::Sell => {
                return self.best_bid();
            }
        }
    }

    pub fn submit_order(&mut self, mut taker: Order) -> Vec<Trade> {
        let mut trade = Vec::new();

        while taker.remaining_qty > 0 && self.can_match(&taker) {
            let mut maker = self
                .pop_best_opposite_order(taker.side)
                .expect("no maker order was found");

            let next_trade_id = self.take_next_trade_id();
            let order_trade = self.execute_one_match(&mut taker, &mut maker, next_trade_id);

            trade.push(order_trade);

            if maker.remaining_qty > 0 {
                self.reinsert_front(maker);
            }
        }

        if taker.remaining_qty > 0 {
            match taker.order_type {
                OrderType::Limit => {
                    self.add_resting_order(taker);
                }
                OrderType::Market => {
                    taker.status = OrderStatus::Cancelled;
                }
            }
        }

        trade
    }

    fn pop_best_opposite_order(&mut self, incoming_side: Side) -> Option<Order> {
        match incoming_side {
            Side::Buy => {
                let best_ask = self.best_ask()?;
                let popped_queue = {
                    let queue = self.asks.get_mut(&best_ask)?;
                    queue.pop_front()
                };
                let is_empty = self
                    .asks
                    .get(&best_ask)
                    .is_some_and(|queue| queue.is_empty());
                if is_empty {
                    self.asks.remove(&best_ask);
                }
                popped_queue
            }
            Side::Sell => {
                let best_bid = self.best_bid()?;

                let popped_queue = {
                    let queue = self.bids.get_mut(&Reverse(best_bid))?;
                    queue.pop_front()
                };

                let is_empty = self
                    .bids
                    .get(&Reverse(best_bid))
                    .is_some_and(|queue| queue.is_empty());
                if is_empty {
                    self.bids.remove(&Reverse(best_bid));
                }

                popped_queue
            }
        }
    }

    fn execute_one_match(&mut self, taker: &mut Order, maker: &mut Order, next_trade_id: u64 ) -> Trade {
        let trade_qty = maker.remaining_qty.min(taker.remaining_qty);
        taker.remaining_qty -= trade_qty;
        maker.remaining_qty -= trade_qty;

        if taker.remaining_qty == 0 {
            taker.status = domain::OrderStatus::Filled;
        } else if taker.original_qty == taker.remaining_qty {
            taker.status = domain::OrderStatus::Open;
        } else {
            taker.status = domain::OrderStatus::PartiallyFilled;
        }

        if maker.remaining_qty == 0 {
            maker.status = domain::OrderStatus::Filled;
        } else if maker.original_qty == maker.remaining_qty {
            maker.status = domain::OrderStatus::Open;
        } else {
            maker.status = domain::OrderStatus::PartiallyFilled;
        }

        let trade_price = maker.limit_price.expect("Makers Limit price not availabe");

        Trade {
            trade_id: next_trade_id,
            maker_order_id: maker.id.clone(),
            taker_order_id: taker.id.clone(),
            maker_user_id: maker.user_id,
            taker_user_id: taker.user_id,
            price: trade_price,
            quantity: trade_qty,
        }
    }

    fn reinsert_front(&mut self, order: Order) {
        match order.order_type {
            OrderType::Limit => {
                let order_price = order.limit_price.expect("Failed limit price");
                match order.side {
                    Side::Buy => {
                        self.bids
                            .entry(Reverse(order_price))
                            .or_default()
                            .push_front(order);
                    }
                    Side::Sell => {
                        self.asks.entry(order_price).or_default().push_front(order);
                    }
                }
            }
            OrderType::Market => {
                panic!("Market orders cannot be added into the order book");
            }
        }
    }
}
