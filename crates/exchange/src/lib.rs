use engine::OrderBook;
use balance::BalanceManager;

pub struct Exchange{
    order_book: OrderBook,
    balances: BalanceManager,
}

impl Exchange {
    pub fn new() -> Self{
        Self { 
            order_book: OrderBook::new(), 
            balances: BalanceManager::new() 
        }
    }
}