use tokio::sync::oneshot;

use domain::{MatchResult, Order, OrderBookSnapshot, OrderId, OrderUpdate};
use tokio::sync::mpsc::Receiver;
use storage::OrderRepository;

use engine::OrderBook;

pub enum MarketCommand {
    PlaceOrder {
        order: Order,
        reply_to: oneshot::Sender<MatchResult>,
    },
    GetBestBid {
        reply_to: oneshot::Sender<Option<u64>>,
    },

    GetBestAsk {
        reply_to: oneshot::Sender<Option<u64>>,
    },
    GetOrderBook {
        reply_to: oneshot::Sender<OrderBookSnapshot>,
    },
    CancelOrder {
        order_id: OrderId,
        reply_to: oneshot::Sender<Option<Order>>,
    },
    LoadOrder {
        order: Order,
    },
}

pub struct MarketActor {
    rx: Receiver<MarketCommand>,
    order_book: OrderBook,
    order_repository: OrderRepository,
}

impl MarketActor {
    pub fn new(rx: Receiver<MarketCommand>, order_repository: OrderRepository) -> Self {
        Self {
            rx,
            order_book: OrderBook::new(),
            order_repository,
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

                    let result = self.order_book.submit_order(order);

                    for order in &result.new_orders {
                        self.order_repository
                            .save_order(order)
                            .await
                            .unwrap();
                    }

                    reply_to.send(result).unwrap();
                },
                MarketCommand::GetBestBid { reply_to } => {
                    let bid = self.order_book.best_bid();
                    reply_to.send(bid).unwrap();
                },
                MarketCommand::GetBestAsk { reply_to } => {
                    let ask = self.order_book.best_ask();
                    reply_to.send(ask).unwrap();
                },
                MarketCommand::GetOrderBook { reply_to } => {
                    let snapshot = self.order_book.snapshot();
                    reply_to.send(snapshot).unwrap();
                },
                MarketCommand::CancelOrder { order_id, reply_to } => {
                    let cancelled_order = self.order_book.cancel_order(order_id);

                    if let Some(mut order) = cancelled_order {
                        order.status = domain::OrderStatus::Cancelled;

                        let update = OrderUpdate {
                            order_id: order.id.clone(),
                            remaining_qty: order.remaining_qty,
                            status: order.status.clone(),
                        };

                        self.order_repository
                            .update_order(&update)
                            .await
                            .unwrap();

                        reply_to.send(Some(order)).unwrap();
                    } else {
                        reply_to.send(None).unwrap();
                    }
                },
                MarketCommand::LoadOrder { order } => {
                    self.order_book.add_resting_order(order);
                }
            }
        }
    }
}
