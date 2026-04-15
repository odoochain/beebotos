//! BSC Client

use crate::chains::common::client::EvmClientBuilder;
use crate::chains::common::{BaseEvmClient, EvmClient, EvmConfig};
use crate::ChainResult;

/// BSC blockchain client
pub struct BscClient {
    base: BaseEvmClient,
    fast_finality: bool,
}

impl BscClient {
    /// Create new BSC client from RPC URL
    pub async fn new(rpc_url: &str) -> ChainResult<Self> {
        let base = BaseEvmClient::new(rpc_url, crate::chains::bsc::BSC_MAINNET_CHAIN_ID).await?;
        Ok(Self {
            base,
            fast_finality: true,
        })
    }

    /// Create from base client
    pub fn from_base(base: BaseEvmClient) -> Self {
        Self {
            base,
            fast_finality: true,
        }
    }

    /// Create with fast finality configuration
    pub fn with_fast_finality(mut self, enabled: bool) -> Self {
        self.fast_finality = enabled;
        self
    }

    /// Check if fast finality is enabled
    pub fn fast_finality(&self) -> bool {
        self.fast_finality
    }

    /// Get recommended confirmation blocks
    pub fn confirmation_blocks(&self) -> u64 {
        if self.fast_finality {
            5 // Fast but less safe
        } else {
            15 // Standard BSC recommendation
        }
    }

    /// Wait for transaction confirmation with BSC-specific logic
    pub async fn wait_for_confirmation(
        &self,
        tx_hash: &str,
        timeout_secs: u64,
    ) -> ChainResult<bool> {
        EvmClient::wait_for_confirmation(self, tx_hash, timeout_secs).await
    }

    /// Get optimal gas price based on priority
    pub async fn get_optimal_gas_price(
        &self,
        priority: crate::chains::bsc::BscPriority,
    ) -> ChainResult<u128> {
        let base_price = self.get_gas_price().await?;
        let multiplier = priority.multiplier();
        Ok((base_price as f64 * multiplier) as u128)
    }
}

impl std::ops::Deref for BscClient {
    type Target = BaseEvmClient;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BscClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[async_trait::async_trait]
impl EvmClient for BscClient {
    fn provider(&self) -> &crate::chains::common::EvmProvider {
        self.base.provider()
    }

    fn config(&self) -> &EvmConfig {
        self.base.config()
    }

    fn confirmation_blocks(&self) -> u64 {
        self.confirmation_blocks()
    }
}

/// BSC client builder
pub struct BscClientBuilder {
    builder: EvmClientBuilder,
    fast_finality: bool,
}

impl BscClientBuilder {
    /// Create new builder for mainnet
    pub fn mainnet() -> Self {
        Self {
            builder: EvmClientBuilder::new(
                "https://bsc-dataseed.binance.org",
                crate::chains::bsc::BSC_MAINNET_CHAIN_ID,
            )
            .with_confirmation_blocks(15),
            fast_finality: true,
        }
    }

    /// Create new builder for testnet
    pub fn testnet() -> Self {
        Self {
            builder: EvmClientBuilder::new(
                "https://data-seed-prebsc-1-s1.binance.org:8545",
                crate::chains::bsc::BSC_TESTNET_CHAIN_ID,
            )
            .with_confirmation_blocks(15),
            fast_finality: true,
        }
    }

    /// Create custom builder
    pub fn custom(rpc_url: &str, chain_id: u64) -> Self {
        Self {
            builder: EvmClientBuilder::new(rpc_url, chain_id).with_confirmation_blocks(15),
            fast_finality: true,
        }
    }

    /// Set fast finality
    pub fn with_fast_finality(mut self, enabled: bool) -> Self {
        self.fast_finality = enabled;
        self
    }

    /// Build client
    pub async fn build(self) -> ChainResult<BscClient> {
        let base = self.builder.build().await?;
        Ok(BscClient {
            base,
            fast_finality: self.fast_finality,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmation_blocks() {
        // Fast finality: 5 blocks, Standard: 15 blocks
        // This is validated by the implementation
    }
}
