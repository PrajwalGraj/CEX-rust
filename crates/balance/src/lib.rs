use domain::Asset;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize )]
pub struct Balance {
    pub available: u64,
    pub locked: u64,
}

#[derive(Debug)]
pub struct BalanceAccount {
    balances: HashMap<Asset, Balance>,
}

#[derive(Debug)]
pub struct BalanceManager {
    accounts: HashMap<u64, BalanceAccount>,
}

#[derive(Debug)]
pub enum BalanceError {
    InsufficientBalance,
    InsufficientLockedBalance,
    AccountNotFound,
    AssetNotFound,
    MarketBuyNotSupported,
}

impl BalanceManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, user_id: u64, asset: Asset, amount: u64) {
        let account = self.accounts.entry(user_id).or_insert(BalanceAccount {
            balances: HashMap::new(),
        });

        let balance = account.balances.entry(asset).or_insert(Balance {
            available: 0,
            locked: 0,
        });
        balance.available += amount;
    }

    pub fn lock(&mut self, user_id: u64, asset: Asset, amount: u64) -> Result<(), BalanceError> {
        let account = self
            .accounts
            .get_mut(&user_id)
            .ok_or(BalanceError::AccountNotFound)?;
        let balance = account
            .balances
            .get_mut(&asset)
            .ok_or(BalanceError::AssetNotFound)?;

        if balance.available >= amount {
            balance.available -= amount;
            balance.locked += amount;

            Ok(())
        } else {
            return Err(BalanceError::InsufficientBalance);
        }
    }

    pub fn unlock(&mut self, user_id: u64, asset: Asset, amount: u64) -> Result<(), BalanceError> {
        let account = self
            .accounts
            .get_mut(&user_id)
            .ok_or(BalanceError::AccountNotFound)?;
        let balance = account
            .balances
            .get_mut(&asset)
            .ok_or(BalanceError::AssetNotFound)?;

        if balance.locked >= amount {
            balance.available += amount;
            balance.locked -= amount;
            Ok(())
        } else {
            Err(BalanceError::InsufficientLockedBalance)
        }
    }

    pub fn debit_locked(
        &mut self,
        user_id: u64,
        asset: Asset,
        amount: u64,
    ) -> Result<(), BalanceError> {
        let account = self
            .accounts
            .get_mut(&user_id)
            .ok_or(BalanceError::AccountNotFound)?;
        let balance = account
            .balances
            .get_mut(&asset)
            .ok_or(BalanceError::AssetNotFound)?;

        if balance.locked >= amount {
            balance.locked -= amount;
            Ok(())
        } else {
            Err(BalanceError::InsufficientLockedBalance)
        }
    }

    pub fn credit_available(
        &mut self,
        user_id: u64,
        asset: Asset,
        amount: u64,
    ) -> Result<(), BalanceError> {
        let account = self
            .accounts
            .get_mut(&user_id)
            .ok_or(BalanceError::AccountNotFound)?;
        let balance = account.balances.entry(asset).or_insert(Balance {
            available: 0,
            locked: 0,
        });

        balance.available += amount;
        Ok(())
    }

    pub fn get_balance(&self, user_id: u64, asset: Asset) -> Option<Balance> {
        let account = self.accounts.get(&user_id)?;
        let balance = account.balances.get(&asset)?;

        Some(balance.clone())
    }

    pub fn load_balance(&mut self, user_id: u64, asset: Asset, available: u64, locked: u64 ) {
        let account = self
            .accounts
            .entry(user_id)
            .or_insert(BalanceAccount { balances: HashMap::new()});

        account.balances.insert(
            asset,
            Balance {
                available,
                locked,
            },
        );
    }
}
