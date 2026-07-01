use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot;
use storage::BalanceRepository;
use balance::{Balance, BalanceError, BalanceManager};
use domain::{Asset, BalanceUpdate, MatchResult, SettlementBatch};

pub enum BalanceCommand {
    Deposit {
        user_id: u64,
        asset: Asset,
        amount: u64,
        reply_to: oneshot::Sender<()>,
    },
    GetBalance {
        user_id: u64,
        asset: Asset,

        reply_to: oneshot::Sender<Option<Balance>>,
    },
    Lock {
        user_id: u64,
        asset: Asset,
        amount: u64,
        reply_to: oneshot::Sender<Result<(), BalanceError>>,
    },
    Unlock{
        user_id: u64,
        asset: Asset,
        amount: u64,
        reply_to: oneshot::Sender<Result<(), BalanceError>>,
    },
    ApplyMatch {
        result: MatchResult,
        reply_to: oneshot::Sender<Result<SettlementBatch, BalanceError>>,
    },
    LoadBalance {
        user_id: u64,
        asset: Asset,
        available: u64,
        locked: u64,
    },
}

pub struct BalanceActor {
    rx: Receiver<BalanceCommand>,
    manager: BalanceManager,
    repository: BalanceRepository,
}

impl BalanceActor {

    pub fn new(rx: Receiver<BalanceCommand> , repository: BalanceRepository,
) -> Self {
        Self {
            rx,
            manager: BalanceManager::new(),
            repository
        }
    }

    pub async fn run(mut self) {
        while let Some(command) = self.rx.recv().await {
            match command {
                BalanceCommand::Deposit { user_id, asset, amount , reply_to} =>{
                    self.manager.deposit(user_id, asset, amount);
                    self.repository
                        .deposit(user_id, asset, amount)
                        .await
                        .unwrap();

                    reply_to.send(()).unwrap();
                },
                BalanceCommand::GetBalance { user_id, asset, reply_to } => {
                    let balance = self.manager.get_balance(user_id, asset);

                    reply_to.send(balance).unwrap();
                },
                BalanceCommand::Lock { user_id, asset, amount, reply_to } => {
                    let balance = self.manager.lock(user_id, asset, amount);
                    if balance.is_ok() {
                        self.repository
                            .lock(user_id, asset, amount)
                            .await
                            .unwrap();
                    }
                    reply_to.send(balance).unwrap();
                },
                BalanceCommand::Unlock { user_id, asset, amount, reply_to } =>{
                    let balance = self.manager.unlock(user_id, asset, amount);
                    if balance.is_ok() {
                        self.repository
                            .unlock(user_id, asset, amount)
                            .await
                            .unwrap();
                    }
                    reply_to.send(balance).unwrap();
                },
                BalanceCommand::ApplyMatch { result, reply_to } => {
                    let batch = self
                        .settle_match(result)
                        .await;

                    reply_to.send(batch).unwrap();
                },
                BalanceCommand::LoadBalance {user_id, asset,available, locked} => {
                    self.manager
                        .load_balance(
                            user_id,
                            asset,
                            available,
                            locked,
                        );
                }
            }
        }
    }

    async fn settle_match(
        &mut self,
        result: MatchResult,
    ) -> Result<SettlementBatch, BalanceError> {
        let mut batch = SettlementBatch {
            balance_updates: Vec::new(),
            order_updates: result.updated_orders,
            trades: result.trades.clone(),
        };

        for trade in &result.trades {

            let (base_asset, quote_asset) = (trade.market.base, trade.market.quote);

            let quote_amount = trade.quantity * trade.price;
            let base_amount = trade.quantity;

            self.manager
                .debit_locked(trade.buyer_user_id, quote_asset, quote_amount)?;

            batch.balance_updates.push(BalanceUpdate {
                user_id: trade.buyer_user_id,
                asset: quote_asset,
                available_delta: 0,
                locked_delta: -(quote_amount as i64),
            });


            self.manager
                .credit_available(trade.buyer_user_id, base_asset, base_amount)?;


            batch.balance_updates.push(BalanceUpdate {
                user_id: trade.buyer_user_id,
                asset: base_asset,
                available_delta: base_amount as i64,
                locked_delta: 0,
            });


            self.manager
                .debit_locked(trade.seller_user_id, base_asset, base_amount)?;

            batch.balance_updates.push(BalanceUpdate {
                user_id: trade.seller_user_id,
                asset: base_asset,
                available_delta: 0,
                locked_delta: -(base_amount as i64),
            });


            self.manager
                .credit_available(trade.seller_user_id, quote_asset, quote_amount)?;

            batch.balance_updates.push(BalanceUpdate {
                user_id: trade.seller_user_id,
                asset: quote_asset,
                available_delta:  quote_amount as i64,
                locked_delta: 0
            });

        }

        Ok(batch)
    }
}
