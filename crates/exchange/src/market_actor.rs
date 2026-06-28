use tokio::sync::oneshot;

use balance::BalanceError;
use domain::{Order, Trade};
use tokio::sync::mpsc::Receiver;
use storage::{OrderRepository, TradeRepository};

use engine::OrderBook;

pub enum MarketCommand {
    PlaceOrder {
        order: Order,
        reply_to: oneshot::Sender<Vec<Trade>>,
    },
    GetBestBid {
        reply_to: oneshot::Sender<Option<u64>>,
    },

    GetBestAsk {
        reply_to: oneshot::Sender<Option<u64>>,
    },
}

pub struct MarketActor {
    rx: Receiver<MarketCommand>,
    order_book: OrderBook,
    order_repository: OrderRepository,
    trade_repository: TradeRepository,
}

impl MarketActor {
    pub fn new(rx: Receiver<MarketCommand>, order_repository: OrderRepository,  trade_repository: TradeRepository,) -> Self {
        Self {
            rx,
            order_book: OrderBook::new(),
            order_repository,
            trade_repository
        }
    }

    pub async fn run(mut self) {
        while let Some(command) = self.rx.recv().await {
            match command {
                MarketCommand::PlaceOrder { order, reply_to} => {

                    // self.order_repository
                    //     .save_order(&order)
                    //     .await
                    //     .unwrap();

                    let trades = self.order_book.submit_order(order);

                    for order in &trades.new_orders {
                        self.order_repository
                            .save_order(order)
                            .await
                            .unwrap();
                    }

                    for order in &trades.updated_orders {
                        self.order_repository
                            .update_order(order)
                            .await
                            .unwrap();
                    }

                    for trade in &trades.trades {
                        self.trade_repository
                            .save_trade(trade)
                            .await
                            .unwrap();
                    }

                    reply_to.send(trades.trades).unwrap();
                },
                MarketCommand::GetBestBid { reply_to } => {
                    let bid = self.order_book.best_bid();
                    reply_to.send(bid).unwrap();
                },
                MarketCommand::GetBestAsk { reply_to } => {
                    let ask = self.order_book.best_ask();
                    reply_to.send(ask).unwrap();
                }
            }
        }
    }
}