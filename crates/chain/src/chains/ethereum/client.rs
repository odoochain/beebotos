//! Ethereum Client

use crate::chains::common::client::EvmClientBuilder;
use crate::chains::common::{BaseEvmClient, EvmClient, EvmConfig};
use crate::ChainResult;

/// Ethereum blockchain client
pub struct EthereumClient {
    base: BaseEvmClient,
    use_eip1559: bool,
}

impl EthereumClient {
    /// Create new Ethereum client from RPC URL
    pub async fn new(rpc_url: &str) -> ChainResult<Self> {
        let base =
            BaseEvmClient::new(rpc_url, crate::chains::ethereum::ETHEREUM_MAINNET_CHAIN_ID).await?;
        Ok(Self {
            base,
            use_eip1559: true,
        })
    }

    /// Create new Ethereum client with known chain ID
    pub fn new_with_chain_id(_rpc_url: &str, _chain_id: u64) -> ChainResult<Self> {
        // Note: This requires an async runtime, consider using a builder pattern
        Err(crate::ChainError::Provider(
            "Use async new() or builder pattern".into(),
        ))
    }

    /// Create from base client
    pub fn from_base(base: BaseEvmClient) -> Self {
        Self {
            base,
            use_eip1559: true,
        }
    }

    /// Create with EIP-1559 configuration
    pub fn with_eip1559(mut self, enabled: bool) -> Self {
        self.use_eip1559 = enabled;
        self
    }

    /// Check if EIP-1559 is enabled
    pub fn use_eip1559(&self) -> bool {
        self.use_eip1559
    }

    /// Get recommended confirmation blocks for safe finality
    pub fn confirmation_blocks(&self) -> u64 {
        12 // ~2.4 minutes for safe
    }

    /// Get confirmation blocks for finalized blocks
    pub fn finalized_blocks(&self) -> u64 {
        64 // 2 epochs for finality
    }

    /// Wait for transaction finalization (2 epochs)
    pub async fn wait_for_finalization(
        &self,
        tx_hash: &str,
        timeout_secs: u64,
    ) -> ChainResult<bool> {
        use std::time::Duration;

        let hash: alloy_primitives::B256 = tx_hash
            .parse()
            .map_err(|_| crate::ChainError::Provider("Invalid tx hash".into()))?;

        self.provider()
            .wait_for_confirmation(
                hash,
                self.finalized_blocks(),
                Duration::from_secs(timeout_secs),
            )
            .await
            .map_err(|e| crate::ChainError::Provider(e.to_string()))?;

        Ok(true)
    }

    /// Get fee history (EIP-1559)
    pub async fn get_fee_history(
        &self,
        block_count: u64,
        newest_block: alloy_rpc_types::BlockNumberOrTag,
        reward_percentiles: &[f64],
    ) -> ChainResult<alloy_rpc_types::FeeHistory> {
        self.provider()
            .get_fee_history(block_count, newest_block, reward_percentiles)
            .await
            .map_err(|e| crate::ChainError::Provider(e.to_string()))
    }
}

impl std::ops::Deref for EthereumClient {
    type Target = BaseEvmClient;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for EthereumClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[async_trait::async_trait]
impl EvmClient for EthereumClient {
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

/// Ethereum client builder
pub struct EthereumClientBuilder {
    builder: EvmClientBuilder,
    use_eip1559: bool,
}

impl EthereumClientBuilder {
    /// Create new builder
    pub fn new(rpc_url: &str) -> Self {
        Self {
            builder: EvmClientBuilder::new(
                rpc_url,
                crate::chains::ethereum::ETHEREUM_MAINNET_CHAIN_ID,
            ),
            use_eip1559: true,
        }
    }

    /// Create for specific network
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.builder = EvmClientBuilder::new(self.builder.rpc_url(), chain_id);
        self
    }

    /// Set WebSocket URL
    pub fn with_ws(mut self, ws_url: &str) -> Self {
        self.builder = self.builder.with_ws(ws_url);
        self
    }

    /// Set EIP-1559 usage
    pub fn with_eip1559(mut self, enabled: bool) -> Self {
        self.use_eip1559 = enabled;
        self
    }

    /// Build client
    pub async fn build(self) -> ChainResult<EthereumClient> {
        let base = self.builder.build().await?;
        Ok(EthereumClient {
            base,
            use_eip1559: self.use_eip1559,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmation_blocks() {
        // Cannot test without actual provider
        // This is a placeholder for unit tests
    }
}
