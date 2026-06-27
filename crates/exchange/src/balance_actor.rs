use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;

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
    // DebitLocked{
    //     user_id: u64,
    //     asset: Asset,
    //     amount: u64,
    //     reply_to: oneshot::Sender<Result<(), BalanceError>>,
    // },
    // CreditAvailable{
    //     user_id: u64,
    //     asset: Asset,
    //     amount: u64,
    //     reply_to: oneshot::Sender<Result<(), BalanceError>>,
    // },
    ApplyTrade {
        trade: Trade,
        reply_to: oneshot::Sender<Result<(), BalanceError>>,
    }
}

pub struct BalanceActor {
    rx: Receiver<BalanceCommand>,
    manager: BalanceManager,
}

impl BalanceActor {

    pub fn new(rx: Receiver<BalanceCommand>) -> Self {
        Self {
            rx,
            manager: BalanceManager::new(),
        }
    }

    pub async fn run(mut self) {
        while let Some(command) = self.rx.recv().await {
            match command {
                BalanceCommand::Deposit { user_id, asset, amount , reply_to} =>{
                    self.manager.deposit(user_id, asset, amount);
                    reply_to.send(()).unwrap();
                },
                BalanceCommand::GetBalance { user_id, asset, reply_to } => {
                    let balance = self.manager.get_balance(user_id, asset);

                    reply_to.send(balance).unwrap();
                },
                BalanceCommand::Lock { user_id, asset, amount, reply_to } => {
                    let balance = self.manager.lock(user_id, asset, amount);
                    reply_to.send(balance).unwrap();
                },
                BalanceCommand::Unlock { user_id, asset, amount, reply_to } =>{
                    let balance = self.manager.unlock(user_id, asset, amount);
                    reply_to.send(balance).unwrap();
                },
                // BalanceCommand::DebitLocked { user_id, asset, amount, reply_to } =>{
                //     let balance = self.manager.debit_locked(user_id, asset, amount);
                //     reply_to.send(balance).unwrap();
                // },
                // BalanceCommand::CreditAvailable { user_id, asset, amount, reply_to } =>{
                //     let balance = self.manager.credit_available(user_id, asset, amount);
                //     reply_to.send(balance).unwrap();
                // },
                BalanceCommand::ApplyTrade { trade, reply_to } => {
                    let (base_asset, quote_asset) = (trade.market.base, trade.market.quote);

                    let quote_amount = trade.quantity * trade.price;
                    let base_amount = trade.quantity;

                    let result = (|| -> Result<(), BalanceError>{
                        self.manager.debit_locked(trade.buyer_user_id, quote_asset, quote_amount )?;
                        self.manager.credit_available(trade.buyer_user_id, base_asset, base_amount )?;
                        self.manager.debit_locked(trade.seller_user_id, base_asset, base_amount )?;
                        self.manager.credit_available(trade.seller_user_id, quote_asset, quote_amount )?;

                        Ok(())
                    })();
                    
                    reply_to.send(result).unwrap();
                }
            }
        }
    }
}

fn main(){

}