//! Beechain Client

use crate::chains::beechain::types::BlockTimeStats;
use crate::chains::common::client::EvmClientBuilder;
use crate::chains::common::{BaseEvmClient, EvmClient, EvmConfig};
use crate::ChainResult;

/// Beechain blockchain client
pub struct BeechainClient {
    base: BaseEvmClient,
    block_stats: BlockTimeStats,
}

impl BeechainClient {
    /// Create new Beechain client from RPC URL
    pub async fn new(rpc_url: &str) -> ChainResult<Self> {
        let base =
            BaseEvmClient::new(rpc_url, crate::chains::beechain::BEECHAIN_MAINNET_CHAIN_ID).await?;
        Ok(Self {
            base,
            block_stats: BlockTimeStats::default(),
        })
    }

    /// Create from base client
    pub fn from_base(base: BaseEvmClient) -> Self {
        Self {
            base,
            block_stats: BlockTimeStats::default(),
        }
    }

    /// Get recommended confirmation blocks (2 blocks for finality)
    pub fn confirmation_blocks(&self) -> u64 {
        crate::chains::beechain::FINALITY_CONFIRMATION_BLOCKS
    }

    /// Get safe confirmation blocks
    pub fn safe_confirmation_blocks(&self) -> u64 {
        crate::chains::beechain::SAFE_CONFIRMATION_BLOCKS
    }

    /// Get estimated time to finality in seconds
    pub fn finality_time_seconds(&self) -> f64 {
        0.8 // 2 blocks * 0.4s
    }

    /// Get current block time statistics
    pub fn block_stats(&self) -> &BlockTimeStats {
        &self.block_stats
    }

    /// Update block statistics
    pub async fn update_block_stats(&mut self) -> ChainResult<()> {
        // In real implementation, fetch block timestamps and calculate
        self.block_stats.update(400.0); // Placeholder
        Ok(())
    }

    /// Check if network is healthy
    pub fn is_network_healthy(&self) -> bool {
        self.block_stats.is_healthy()
    }

    /// Wait for safe confirmation
    pub async fn wait_for_safe_confirmation(
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
                self.safe_confirmation_blocks(),
                Duration::from_secs(timeout_secs),
            )
            .await
            .map_err(|e| crate::ChainError::Provider(e.to_string()))?;

        Ok(true)
    }

    /// Get explorer URL for transaction
    pub fn get_explorer_url(&self, tx_hash: &str) -> String {
        format!("https://scan.beechain.ai/tx/{}", tx_hash)
    }

    /// Get explorer URL for address
    pub fn get_address_explorer_url(&self, address: &str) -> String {
        format!("https://scan.beechain.ai/address/{}", address)
    }

    /// Get estimated TPS
    pub fn estimated_tps(&self) -> u32 {
        if self.is_network_healthy() {
            crate::chains::beechain::TARGET_TPS
        } else {
            1000 // Conservative estimate
        }
    }
}

impl std::ops::Deref for BeechainClient {
    type Target = BaseEvmClient;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BeechainClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[async_trait::async_trait]
impl EvmClient for BeechainClient {
    fn provider(&self) -> &crate::chains::common::EvmProvider {
        self.base.provider()
    }

    fn config(&self) -> &EvmConfig {
        self.base.config()
    }

    fn confirmation_blocks(&self) -> u64 {
        self.confirmation_blocks()
    }

    async fn wait_for_confirmation(&self, tx_hash: &str, timeout_secs: u64) -> ChainResult<bool> {
        use std::time::Duration;

        let _hash: alloy_primitives::B256 = tx_hash
            .parse()
            .map_err(|_| crate::ChainError::Provider("Invalid tx hash".into()))?;

        // Poll every 100ms for fast chains like Beechain
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout {
            if let Ok(Some(receipt)) = self.get_transaction_receipt(tx_hash).await {
                if let Some(block_number) = receipt.block_number {
                    let current = self.get_block_number().await?;
                    if current >= block_number + self.confirmation_blocks() {
                        return Ok(true);
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(false)
    }
}

/// Beechain client builder
pub struct BeechainClientBuilder {
    builder: EvmClientBuilder,
}

impl BeechainClientBuilder {
    /// Create new builder for mainnet
    pub fn mainnet() -> Self {
        Self {
            builder: EvmClientBuilder::new(
                "https://rpc.beechain.ai",
                crate::chains::beechain::BEECHAIN_MAINNET_CHAIN_ID,
            )
            .with_confirmation_blocks(crate::chains::beechain::FINALITY_CONFIRMATION_BLOCKS),
        }
    }

    /// Create new builder for testnet
    pub fn testnet() -> Self {
        Self {
            builder: EvmClientBuilder::new(
                "https://testnet-rpc.beechain.ai",
                crate::chains::beechain::BEECHAIN_TESTNET_CHAIN_ID,
            )
            .with_confirmation_blocks(crate::chains::beechain::FINALITY_CONFIRMATION_BLOCKS),
        }
    }

    /// Create custom builder
    pub fn custom(rpc_url: &str, chain_id: u64) -> Self {
        Self {
            builder: EvmClientBuilder::new(rpc_url, chain_id)
                .with_confirmation_blocks(crate::chains::beechain::FINALITY_CONFIRMATION_BLOCKS),
        }
    }

    /// Build client
    pub async fn build(self) -> ChainResult<BeechainClient> {
        let base = self.builder.build().await?;
        Ok(BeechainClient {
            base,
            block_stats: BlockTimeStats::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmation_blocks() {
        // Finality: 2 blocks
        // Safe: 3 blocks
        assert_eq!(crate::chains::beechain::FINALITY_CONFIRMATION_BLOCKS, 2);
        assert_eq!(crate::chains::beechain::SAFE_CONFIRMATION_BLOCKS, 3);
    }

    #[test]
    fn test_explorer_urls() {
        // Test explorer URL generation
    }
}
