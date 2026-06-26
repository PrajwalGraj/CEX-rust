use std::collections::HashMap;
use engine::OrderBook;
use balance::BalanceManager;
use domain::{Market, Order, Side, Trade};
use balance::{BalanceError};

pub struct Exchange {
    order_books: HashMap< Market, OrderBook>,
    balances: BalanceManager,
}

impl Exchange {
    pub fn new() -> Self{
        Self { 
            order_books: HashMap::new(), 
            balances: BalanceManager::new() 
        }
    }

    pub fn submit_order(&mut self, order: Order) -> Result<Vec<Trade>, BalanceError>{
        self.reserve_funds(&order)?;

        let market = order.market;
        let order_book = self.order_books.entry(market).or_insert(OrderBook::new());

        Ok( order_book.submit_order(order))
        
    }

    fn reserve_funds(&mut self, order: &Order) -> Result<(), BalanceError>{
        match order.side {
            Side::Buy => {
                let asset = order.market.quote;
                let price = order.limit_price.ok_or(BalanceError::MarketBuyNotSupported)?;
                let amount = order.remaining_qty * price;

                self.balances.lock(order.user_id, asset, amount)
            },
            Side::Sell => {
                let asset = order.market.base;
                let amount = order.remaining_qty;

                self.balances.lock(order.user_id, asset, amount)
            }
        }
    }
}