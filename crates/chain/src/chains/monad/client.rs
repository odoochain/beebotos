//! Monad Client

use crate::chains::common::client::EvmClientBuilder;
use crate::chains::common::{BaseEvmClient, EvmClient, EvmConfig};
use crate::ChainResult;

/// Monad blockchain client
pub struct MonadClient {
    base: BaseEvmClient,
    parallel_execution: bool,
}

impl MonadClient {
    /// Create new Monad client from RPC URL
    pub async fn new(rpc_url: &str) -> ChainResult<Self> {
        let base = BaseEvmClient::new(rpc_url, crate::chains::monad::MONAD_CHAIN_ID).await?;
        Ok(Self {
            base,
            parallel_execution: true,
        })
    }

    /// Create from base client
    pub fn from_base(base: BaseEvmClient) -> Self {
        Self {
            base,
            parallel_execution: true,
        }
    }

    /// Create with chain ID
    pub fn new_with_chain_id(_rpc_url: &str, _chain_id: u64) -> ChainResult<Self> {
        // Async required for provider creation
        Err(crate::ChainError::Provider("Use async new()".into()))
    }

    /// Get the underlying base client
    pub fn base(&self) -> &BaseEvmClient {
        &self.base
    }

    /// Get the underlying provider
    pub fn provider(&self) -> &crate::chains::common::EvmProvider {
        self.base.provider()
    }

    /// Check if parallel execution is enabled
    pub fn parallel_execution(&self) -> bool {
        self.parallel_execution
    }

    /// Enable/disable parallel execution
    pub fn set_parallel_execution(&mut self, enabled: bool) {
        self.parallel_execution = enabled;
    }

    /// Get chain ID
    pub fn chain_id(&self) -> u64 {
        self.base.chain_id()
    }

    /// Get recommended confirmation blocks
    pub fn confirmation_blocks(&self) -> u64 {
        1 // Monad has fast finality
    }
}

impl std::ops::Deref for MonadClient {
    type Target = BaseEvmClient;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for MonadClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[async_trait::async_trait]
impl EvmClient for MonadClient {
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

/// Monad client builder
pub struct MonadClientBuilder {
    builder: EvmClientBuilder,
    parallel_execution: bool,
}

impl MonadClientBuilder {
    /// Create new builder for mainnet
    pub fn mainnet() -> Self {
        Self {
            builder: EvmClientBuilder::new(
                "https://rpc.monad.xyz",
                crate::chains::monad::MONAD_CHAIN_ID,
            )
            .with_confirmation_blocks(1),
            parallel_execution: true,
        }
    }

    /// Create new builder for testnet
    pub fn testnet() -> Self {
        Self {
            builder: EvmClientBuilder::new(
                "https://rpc.testnet.monad.xyz",
                crate::chains::monad::MONAD_TESTNET_CHAIN_ID,
            )
            .with_confirmation_blocks(1),
            parallel_execution: true,
        }
    }

    /// Create custom builder
    pub fn custom(rpc_url: &str, chain_id: u64) -> Self {
        Self {
            builder: EvmClientBuilder::new(rpc_url, chain_id).with_confirmation_blocks(1),
            parallel_execution: true,
        }
    }

    /// Set parallel execution
    pub fn with_parallel_execution(mut self, enabled: bool) -> Self {
        self.parallel_execution = enabled;
        self
    }

    /// Build client
    pub async fn build(self) -> ChainResult<MonadClient> {
        let base = self.builder.build().await?;
        Ok(MonadClient {
            base,
            parallel_execution: self.parallel_execution,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monad_client_builder() {
        // Test builder configuration
    }
}
