use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use storage::BalanceRepository;

use balance::{Balance, BalanceError, BalanceManager};
use domain::{Asset, Trade};

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
    ApplyTrade {
        trade: Trade,
        reply_to: oneshot::Sender<Result<(), BalanceError>>,
    }
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
                BalanceCommand::ApplyTrade { trade, reply_to } => {
                    let result = self.settle_trade(&trade).await;
                    
                    reply_to.send(result).unwrap();
                }
            }
        }
    }

    async fn settle_trade(
        &mut self,
        trade: &Trade,
    ) -> Result<(), BalanceError> {
        let (base_asset, quote_asset) = (trade.market.base, trade.market.quote);

        let quote_amount = trade.quantity * trade.price;
        let base_amount = trade.quantity;

        self.manager
            .debit_locked(trade.buyer_user_id, quote_asset, quote_amount)?;

        self.repository
            .debit_locked(trade.buyer_user_id, quote_asset, quote_amount)
            .await
            .unwrap();


        self.manager
            .credit_available(trade.buyer_user_id, base_asset, base_amount)?;

        self.repository
            .credit_available(trade.buyer_user_id, base_asset, base_amount)
            .await
            .unwrap();


        self.manager
            .debit_locked(trade.seller_user_id, base_asset, base_amount)?;

        self.repository
            .debit_locked(trade.seller_user_id, base_asset, base_amount)
            .await
            .unwrap();


        self.manager
            .credit_available(trade.seller_user_id, quote_asset, quote_amount)?;

        self.repository
            .credit_available(trade.seller_user_id, quote_asset, quote_amount)
            .await
            .unwrap();

        Ok(())
    }
}