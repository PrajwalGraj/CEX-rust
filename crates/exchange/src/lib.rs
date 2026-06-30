use storage::{Database, BalanceRepository, OrderRepository, TradeRepository};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

use crate::balance_actor::{BalanceActor, BalanceCommand};
use crate::market_actor::{MarketActor, MarketCommand};
use std::collections::HashMap;
use engine::OrderBook;
use balance::BalanceManager;
use domain::{Market, Order, Side, Trade,OrderId, Asset, OrderBookSnapshot};
use balance::{BalanceError, Balance};
pub mod balance_actor;
pub mod market_actor;
pub struct Exchange {
    markets: HashMap<Market, Sender<MarketCommand>>,
    balance_tx: Sender<BalanceCommand>,
    database: Database,
}

impl Exchange {
    pub async fn new() -> Self{
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let database = Database::connect( &database_url )
        .await
        .unwrap();

        let balance_repo = BalanceRepository::new(database.pool());
        let order_repo = OrderRepository::new(database.pool());
        let balances = BalanceRepository::new(database.pool()).load_all_balances().await.unwrap();

        let (tx, rx) = mpsc::channel(32);
        let actor = BalanceActor::new(rx, balance_repo);
        
        tokio::spawn(actor.run());

        let open_orders = order_repo.load_open_orders().await.unwrap();
        
        for (user_id, asset, available, locked) in balances {
            tx.send(BalanceCommand::LoadBalance {
                user_id,
                asset,
                available,
                locked,
            })
            .await
            .unwrap();
        }
        
        let mut exchange = Self {
            markets: HashMap::new(),
            balance_tx: tx,
            database,
        };

        for order in open_orders {
            exchange.load_order(order).await;
        }

        exchange
    }

    pub async fn submit_order(&mut self, order: Order) -> Result<Vec<Trade>, BalanceError>{
        self.reserve_funds(&order).await?;

        let market = order.market;
        
        if !self.markets.contains_key(&market) {
            let (tx, rx) = mpsc::channel(32);

            let order_repo = OrderRepository::new(self.database.pool());
            let trade_repo = TradeRepository::new(self.database.pool());

            let actor = MarketActor::new(rx, order_repo, trade_repo);

            tokio::spawn(actor.run());

            self.markets.insert(market, tx);
        }

        let sender = self.markets.get(&market).unwrap();

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

    async fn unlock_reserved_funds(&mut self, order: &Order) {

        let (asset, amount) = match order.side {
            Side::Buy => {
                let price = order.limit_price.unwrap();
                (
                    order.market.quote,
                    order.remaining_qty * price,
                )
            }

            Side::Sell => {
                (
                    order.market.base,
                    order.remaining_qty,
                )
            }
        };

        let (tx, rx) = oneshot::channel();

        self.balance_tx
            .send(BalanceCommand::Unlock {
                user_id: order.user_id,
                asset,
                amount,
                reply_to: tx,
            })
            .await
            .unwrap();

        rx.await.unwrap().unwrap();
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

    pub async fn order_book(&self, market: &Market ) -> Option<OrderBookSnapshot> {
        let sender = self.markets.get(market)?;

        let (tx, rx) = oneshot::channel();
        sender
            .send(MarketCommand::GetOrderBook { reply_to: tx })
            .await
            .unwrap();

        rx.await.ok()
    }

    pub async fn cancel_order( &mut self, market: &Market, order_id: OrderId ) -> bool {
        let sender = match self.markets.get(market) {
            Some(sender) => sender,
            None => return false,
        };

        let (tx, rx) = oneshot::channel();

        sender
            .send(MarketCommand::CancelOrder {
                order_id,
                reply_to: tx,
            })
            .await
            .unwrap();

        let cancelled_order = match rx.await.unwrap() {
            Some(order) => order,
            None => return false,
        };

        self.unlock_reserved_funds(&cancelled_order).await;

        true
    }

    async fn load_order(&mut self, order: Order ) {

    let market = order.market;

    if !self.markets.contains_key(&market) {
        let (tx, rx) = mpsc::channel(32);

        let order_repo = OrderRepository::new(self.database.pool());
        let trade_repo = TradeRepository::new(self.database.pool());

        let actor = MarketActor::new(
            rx,
            order_repo,
            trade_repo,
        );

        tokio::spawn(actor.run());

        self.markets.insert(market, tx);
    }

    let sender = self.markets.get(&market).unwrap();

    sender
        .send(MarketCommand::LoadOrder { order })
        .await
        .unwrap();
}
}