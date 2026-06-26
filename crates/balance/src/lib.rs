use std::collections::{HashMap, hash_map};
use domain::{Asset};



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Balance{
    pub available: u64,
    pub locked : u64
}

#[derive(Debug)]
pub struct BalanceAccount {
    balances: HashMap<Asset,Balance>,
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
    
}

impl BalanceManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn deposit( &mut self, user_id: u64, asset: Asset, amount: u64){
        let account = self.accounts.entry(user_id).or_insert(BalanceAccount { balances: HashMap::new() });

        let balance = account.balances.entry(asset).or_insert(Balance { available: 0, locked: 0 });
        balance.available += amount;
    }

    pub fn lock(&mut self, user_id: u64, asset: Asset, amount: u64) -> Result<(), BalanceError>{
        let account = self.accounts.get_mut(&user_id).ok_or(BalanceError::AccountNotFound)?;
        let balance = account.balances.get_mut(&asset).ok_or(BalanceError::AssetNotFound)?;

        if balance.available >= amount {
            balance.available -= amount;
            balance.locked += amount;

            Ok(())
        }else{
            return Err(BalanceError::InsufficientBalance);
        }

    }

    pub fn unlock(&mut self, user_id: u64, asset: Asset, amount: u64) -> Result<(), BalanceError>{
        let account = self.accounts.get_mut(&user_id).ok_or(BalanceError::AccountNotFound)?;
        let balance = account.balances.get_mut(&asset).ok_or(BalanceError::AssetNotFound)?;

        if balance.locked >= amount {
            balance.available += amount;
            balance.locked -= amount;
            Ok(())
        }else{
            Err(BalanceError::InsufficientLockedBalance)
        }

    }
}