use std::collections::HashMap;
use engine::OrderBook;
use balance::BalanceManager;
use domain::{Market, Order, Side, Trade,OrderId, Asset};
use balance::{BalanceError, Balance};

pub struct Exchange {
    order_books: HashMap< Market, OrderBook>,
    balances: BalanceManager,
}

impl Exchange {
    pub fn new() -> Self{
        Self { 
            order_books: HashMap::new(), 
            balances: BalanceManager::new() ,
        }
    }

    pub fn submit_order(&mut self, order: Order) -> Result<Vec<Trade>, BalanceError>{
        self.reserve_funds(&order)?;

        let market = order.market;
        let order_book = self.order_books.entry(market).or_insert(OrderBook::new());

        let trades = order_book.submit_order(order);

        for trade in &trades {
            self.apply_trade(trade)?;
        }

        Ok(trades)
        
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

    fn apply_trade( &mut self, trade: &Trade ) -> Result<(), BalanceError>{
        let (base_asset, quote_asset ) = (trade.market.base, trade.market.quote);

        let quote_amount = trade.quantity * trade.price;
        let base_amount = trade.quantity;

        self.balances.debit_locked(trade.buyer_user_id, quote_asset, quote_amount)?;
        self.balances.credit_available(trade.buyer_user_id, base_asset, base_amount)?;

        self.balances.debit_locked(trade.seller_user_id, base_asset, base_amount)?;
        self.balances.credit_available(trade.seller_user_id, quote_asset, quote_amount)?;
        

        Ok(())
    }

    pub fn deposit(&mut self, user_id: u64, asset: Asset, amount: u64 ){
        self.balances.deposit(user_id, asset, amount);
    }

    pub fn get_balance(&self, user_id: u64, asset: Asset ) -> Option<&Balance>{
        self.balances.get_balance(user_id, asset)
    }

    pub fn best_bid(&self, market: &Market) -> Option<u64>{
        self.order_books.get(market).and_then(|book| book.best_bid())
    }

    pub fn best_ask(&self, market: &Market) -> Option<u64>{
        self.order_books.get(market).and_then(|book| book.best_ask())
    }
}