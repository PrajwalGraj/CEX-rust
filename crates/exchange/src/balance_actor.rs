use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;

use balance::{Balance, BalanceError, BalanceManager};
use domain::Asset;

pub enum BalanceCommand {
    Deposit {
        user_id: u64,
        asset: Asset,
        amount: u64,
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
    DebitLocked{
        user_id: u64,
        asset: Asset,
        amount: u64,
        reply_to: oneshot::Sender<Result<(), BalanceError>>,
    },
    CreditAvailable{
        user_id: u64,
        asset: Asset,
        amount: u64,
        reply_to: oneshot::Sender<Result<(), BalanceError>>,
    },
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
                BalanceCommand::Deposit { user_id, asset, amount } =>{
                    self.manager.deposit(user_id, asset, amount);
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
                BalanceCommand::DebitLocked { user_id, asset, amount, reply_to } =>{
                    let balance = self.manager.debit_locked(user_id, asset, amount);
                    reply_to.send(balance).unwrap();
                },
                BalanceCommand::CreditAvailable { user_id, asset, amount, reply_to } =>{
                    let balance = self.manager.credit_available(user_id, asset, amount);
                    reply_to.send(balance).unwrap();
                }
            }
        }
    }
}

fn main(){

}