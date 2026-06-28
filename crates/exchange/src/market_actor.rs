use tokio::sync::oneshot;

use balance::BalanceError;
use domain::{Order, Trade};
use tokio::sync::mpsc::Receiver;

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
}

impl MarketActor {
    pub fn new(rx: Receiver<MarketCommand>) -> Self {
        Self {
            rx,
            order_book: OrderBook::new(),
        }
    }

    pub async fn run(mut self) {
        while let Some(command) = self.rx.recv().await {
            match command {
                MarketCommand::PlaceOrder { order, reply_to} => {
                    let trades = self.order_book.submit_order(order);

                    reply_to.send(trades).unwrap();
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



fn main(){

}