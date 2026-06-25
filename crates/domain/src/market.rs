#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Asset{
    SOL,
    BTC,
    USDC
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Markets{
    SOLUSDC,
    BTCUSDC
}


impl Markets{
    pub fn base_asset(self) -> Asset{
        match self{
            Markets::SOLUSDC => Asset::SOL,
            Markets::BTCUSDC => Asset::BTC
        } 
    }

    pub fn quote_asset(self) -> Asset{
        Asset::USDC
    }
    pub fn base_scale(self) -> u64{

    }
}