use domain::{Order, OrderType, Side};
use std::cmp::Reverse;
use std::collections::{BTreeMap, VecDeque};

pub struct OrderBook {
    bids: BTreeMap<Reverse<u64>, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
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

    pub fn submit_order(&mut self, order: Order) {
        if self.can_match(&order) {
            println!("Order can match");
        } else {
            match order.order_type {
                OrderType::Limit => {
                    println!("No match found: Resting in order book");
                    self.add_resting_order(order);
                }
                OrderType::Market => {
                    println!("No liquidity: Order cancelled");
                }
            }
        }
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
}
