//! DEX Implementation

use crate::compat::Address;

/// Uniswap V2 style DEX
pub struct UniswapV2DEX {
    #[allow(dead_code)]
    router: Address,
    #[allow(dead_code)]
    factory: Address,
}

impl UniswapV2DEX {
    pub fn new(router: Address, factory: Address) -> Self {
        Self { router, factory }
    }

    pub fn router(&self) -> Address {
        self.router
    }

    pub fn factory(&self) -> Address {
        self.factory
    }
}

/// Uniswap V3 style DEX
pub struct UniswapV3DEX {
    #[allow(dead_code)]
    router: Address,
    #[allow(dead_code)]
    factory: Address,
    #[allow(dead_code)]
    quoter: Address,
}

impl UniswapV3DEX {
    pub fn new(router: Address, factory: Address, quoter: Address) -> Self {
        Self {
            router,
            factory,
            quoter,
        }
    }
}
