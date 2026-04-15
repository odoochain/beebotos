//! Lending Protocol Implementation

use crate::compat::Address;

/// Compound style lending protocol
pub struct CompoundStyleLending {
    #[allow(dead_code)]
    comptroller: Address,
    #[allow(dead_code)]
    cether: Address,
}

impl CompoundStyleLending {
    pub fn new(comptroller: Address, cether: Address) -> Self {
        Self {
            comptroller,
            cether,
        }
    }
}

/// Aave style lending protocol
pub struct AaveStyleLending {
    #[allow(dead_code)]
    pool: Address,
    #[allow(dead_code)]
    pool_data_provider: Address,
}

impl AaveStyleLending {
    pub fn new(pool: Address, pool_data_provider: Address) -> Self {
        Self {
            pool,
            pool_data_provider,
        }
    }
}
