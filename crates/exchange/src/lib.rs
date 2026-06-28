use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use crate::balance_actor::{BalanceActor, BalanceCommand};
use crate::market_actor::{MarketActor, MarketCommand};
use std::collections::HashMap;
use engine::OrderBook;
use balance::BalanceManager;
use domain::{Market, Order, Side, Trade,OrderId, Asset};
use balance::{BalanceError, Balance};
pub mod balance_actor;
pub mod market_actor;
pub struct Exchange {
    markets: HashMap<Market, Sender<MarketCommand>>,
    balance_tx: Sender<BalanceCommand>,
}

impl Exchange {
    pub fn new() -> Self{

        let (tx, rx) = mpsc::channel(32);
        let actor = BalanceActor::new(rx);
        tokio::spawn(actor.run());

        Self { 
            markets : HashMap::new(),
            balance_tx: tx,
        }
    }

    pub async fn submit_order(&mut self, order: Order) -> Result<Vec<Trade>, BalanceError>{
        self.reserve_funds(&order).await?;

        let market = order.market;
        let sender = self.markets.entry(market).or_insert_with(|| {
            let (tx, rx) = mpsc::channel(32);

            let actor = MarketActor::new(rx);
            tokio::spawn(actor.run());

            tx
        });

        let (tx, rx) = oneshot::channel();

        sender.send(MarketCommand::PlaceOrder { order, reply_to: tx }).await.unwrap();

        let trades = rx.await.unwrap();

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
        let (tx, rx) = oneshot::channel();

        self.balance_tx.send(BalanceCommand::ApplyTrade { trade: trade.clone(), reply_to: tx }).await.unwrap();

        rx.await.unwrap()
    }

    pub async fn deposit(&mut self, user_id: u64, asset: Asset, amount: u64 ){
        let (tx, rx) = oneshot::channel();
        self.balance_tx.send(BalanceCommand::Deposit { user_id, asset, amount , reply_to: tx}).await.unwrap();
        rx.await.unwrap();
    }

    pub async fn get_balance(&self, user_id: u64, asset: Asset ) -> Option<Balance>{
        let (tx, rx) = oneshot::channel();

        self.balance_tx.send(BalanceCommand::GetBalance { user_id, asset, reply_to: tx }).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn best_bid(&self, market: &Market) -> Option<u64>{
        let sender = self.markets.get(market)?;
        let (tx, rx) = oneshot::channel();
        sender.send(MarketCommand::GetBestBid { reply_to: tx }).await.unwrap();
        rx.await.unwrap()
    }

    pub async fn best_ask(&self, market: &Market) -> Option<u64>{
        let sender = self.markets.get(market)?;

        let (tx, rx) = oneshot::channel();
        sender.send(MarketCommand::GetBestAsk { reply_to: tx }).await.unwrap();
        rx.await.unwrap()
    }
}