use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use crate::balance_actor::{BalanceActor, BalanceCommand};

use std::collections::HashMap;
use engine::OrderBook;
use balance::BalanceManager;
use domain::{Market, Order, Side, Trade,OrderId, Asset};
use balance::{BalanceError, Balance};
pub mod balance_actor;
pub struct Exchange {
    order_books: HashMap< Market, OrderBook>,
    balance_tx: Sender<BalanceCommand>,
}

impl Exchange {
    pub fn new() -> Self{

        let (tx, rx) = mpsc::channel(32);
        let actor = BalanceActor::new(rx);
        tokio::spawn(actor.run());

        Self { 
            order_books: HashMap::new(), 
            balance_tx: tx,
        }
    }

    pub async fn submit_order(&mut self, order: Order) -> Result<Vec<Trade>, BalanceError>{
        self.reserve_funds(&order).await?;

        let market = order.market;
        let order_book = self.order_books.entry(market).or_insert(OrderBook::new());

        let trades = order_book.submit_order(order);

        for trade in &trades {
            self.apply_trade(trade).await?;
        }

        Ok(trades)
        
    }

    async fn reserve_funds(&mut self, order: &Order) -> Result<(), BalanceError>{
        match order.side {
            Side::Buy => {
                let asset = order.market.quote;
                let price = order.limit_price.ok_or(BalanceError::MarketBuyNotSupported)?;
                let amount = order.remaining_qty * price;

                let (tx, rx) = oneshot::channel();
                self.balance_tx.send(BalanceCommand::Lock { user_id: order.user_id, asset, amount, reply_to: tx }).await.unwrap();
                rx.await.unwrap()
            },
            Side::Sell => {
                let asset = order.market.base;
                let amount = order.remaining_qty;

                let (tx, rx) = oneshot::channel();
                self.balance_tx.send(BalanceCommand::Lock { user_id: order.user_id, asset, amount, reply_to: tx }).await.unwrap();
                rx.await.unwrap()
            }
        }
    }

    async fn apply_trade( &mut self, trade: &Trade ) -> Result<(), BalanceError>{
        let (base_asset, quote_asset ) = (trade.market.base, trade.market.quote);

        let quote_amount = trade.quantity * trade.price;
        let base_amount = trade.quantity;

        {
            let (tx, rx) = oneshot::channel();
            self.balance_tx.send(BalanceCommand::DebitLocked { user_id: trade.buyer_user_id, asset: quote_asset, amount: quote_amount, reply_to: tx }).await.unwrap();
            rx.await.unwrap()?;
        }

        {
            let (tx, rx) = oneshot::channel();
            self.balance_tx.send(BalanceCommand::CreditAvailable { user_id: trade.buyer_user_id, asset: base_asset, amount: base_amount, reply_to: tx }).await.unwrap();
            rx.await.unwrap()?;
        }

        {
            let (tx, rx) = oneshot::channel();
            self.balance_tx.send(BalanceCommand::DebitLocked { user_id: trade.seller_user_id, asset: base_asset, amount: base_amount, reply_to: tx}).await.unwrap();
            rx.await.unwrap()?;
        }
        
        {
            let (tx, rx) = oneshot::channel();
            self.balance_tx.send(BalanceCommand::CreditAvailable { user_id: trade.seller_user_id, asset: quote_asset, amount: quote_amount, reply_to: tx }).await.unwrap();
            rx.await.unwrap()?;
        }   

        Ok(())
        
    }

    pub async fn deposit(&mut self, user_id: u64, asset: Asset, amount: u64 ){
        self.balance_tx.send(BalanceCommand::Deposit { user_id, asset, amount }).await.unwrap();
    }

    pub async fn get_balance(&self, user_id: u64, asset: Asset ) -> Option<Balance>{
        let (tx, rx) = oneshot::channel();

        self.balance_tx.send(BalanceCommand::GetBalance { user_id, asset, reply_to: tx }).await.unwrap();
        rx.await.unwrap()
    }

    pub fn best_bid(&self, market: &Market) -> Option<u64>{
        self.order_books.get(market).and_then(|book| book.best_bid())
    }

    pub fn best_ask(&self, market: &Market) -> Option<u64>{
        self.order_books.get(market).and_then(|book| book.best_ask())
    }
}