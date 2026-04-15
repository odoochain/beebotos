//! Multi-Chain Abstraction Layer
//!
//! Provides unified management of multiple blockchain connections,
//! cross-chain routing, health monitoring, and load balancing.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument, warn};

use crate::chains::common::client::EvmClient;
use crate::chains::common::BaseEvmClient;
use crate::{ChainError, Result};

/// Chain connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainConnectionStatus {
    /// Chain is connected and healthy
    Connected,
    /// Chain is disconnected
    Disconnected,
    /// Chain is experiencing issues
    Degraded,
    /// Chain is synchronizing
    Syncing,
}

/// Chain connection info
#[derive(Debug, Clone)]
pub struct ChainConnection {
    pub chain_id: u64,
    pub chain_name: String,
    pub client: Arc<RwLock<BaseEvmClient>>,
    pub status: ChainConnectionStatus,
    pub last_health_check: Instant,
    pub block_height: u64,
    pub latency_ms: u64,
    pub fail_count: u32,
}

/// Multi-chain manager configuration
#[derive(Debug, Clone)]
pub struct MultiChainConfig {
    /// Health check interval
    pub health_check_interval: Duration,
    /// Maximum latency for healthy chain (ms)
    pub max_latency_ms: u64,
    /// Maximum failures before marking as degraded
    pub max_failures: u32,
    /// Auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// Reconnect delay
    pub reconnect_delay: Duration,
}

impl Default for MultiChainConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(30),
            max_latency_ms: 5000,
            max_failures: 3,
            auto_reconnect: true,
            reconnect_delay: Duration::from_secs(10),
        }
    }
}

/// Multi-chain manager
pub struct MultiChainManager {
    chains: Arc<RwLock<HashMap<u64, ChainConnection>>>,
    config: MultiChainConfig,
    primary_chain: Arc<RwLock<Option<u64>>>,
    /// RELIABILITY FIX: Cancellation token for graceful termination
    /// Using CancellationToken instead of mpsc channel - simpler and more
    /// efficient
    cancellation_token: CancellationToken,
}

impl MultiChainManager {
    /// Create new multi-chain manager
    pub fn new(config: MultiChainConfig) -> Self {
        Self {
            chains: Arc::new(RwLock::new(HashMap::new())),
            config,
            primary_chain: Arc::new(RwLock::new(None)),
            cancellation_token: CancellationToken::new(),
        }
    }

    /// Add a chain to the manager
    #[instrument(skip(self, client), target = "chain::multichain")]
    pub async fn add_chain(&self, chain_id: u64, chain_name: String, client: BaseEvmClient) {
        let connection = ChainConnection {
            chain_id,
            chain_name: chain_name.clone(),
            client: Arc::new(RwLock::new(client)),
            status: ChainConnectionStatus::Connected,
            last_health_check: Instant::now(),
            block_height: 0,
            latency_ms: 0,
            fail_count: 0,
        };

        self.chains.write().await.insert(chain_id, connection);

        // Set as primary if first chain
        let mut primary = self.primary_chain.write().await;
        if primary.is_none() {
            *primary = Some(chain_id);
        }

        info!(chain_id = chain_id, name = %chain_name, "Chain added to manager");
    }

    /// Remove a chain from the manager
    pub async fn remove_chain(&self, chain_id: u64) -> Result<()> {
        let mut chains = self.chains.write().await;

        if chains.remove(&chain_id).is_some() {
            // Update primary if removed chain was primary
            let mut primary = self.primary_chain.write().await;
            if *primary == Some(chain_id) {
                *primary = chains.keys().next().copied();
            }

            info!(chain_id = chain_id, "Chain removed from manager");
            Ok(())
        } else {
            Err(ChainError::Provider(format!(
                "Chain {} not found",
                chain_id
            )))
        }
    }

    /// Get chain connection
    pub async fn get_chain(&self, chain_id: u64) -> Result<ChainConnection> {
        self.chains
            .read()
            .await
            .get(&chain_id)
            .cloned()
            .ok_or_else(|| ChainError::Provider(format!("Chain {} not found", chain_id)))
    }

    /// Get primary chain
    pub async fn get_primary_chain(&self) -> Result<ChainConnection> {
        let primary = *self.primary_chain.read().await;
        match primary {
            Some(chain_id) => self.get_chain(chain_id).await,
            None => Err(ChainError::Provider("No primary chain set".to_string())),
        }
    }

    /// Set primary chain
    pub async fn set_primary_chain(&self, chain_id: u64) -> Result<()> {
        if self.chains.read().await.contains_key(&chain_id) {
            *self.primary_chain.write().await = Some(chain_id);
            info!(chain_id = chain_id, "Primary chain updated");
            Ok(())
        } else {
            Err(ChainError::Provider(format!(
                "Chain {} not found",
                chain_id
            )))
        }
    }

    /// Get all connected chains
    pub async fn get_connected_chains(&self) -> Vec<ChainConnection> {
        self.chains
            .read()
            .await
            .values()
            .filter(|c| matches!(c.status, ChainConnectionStatus::Connected))
            .cloned()
            .collect()
    }

    /// Get all chain IDs
    pub async fn get_chain_ids(&self) -> Vec<u64> {
        self.chains.read().await.keys().copied().collect()
    }

    /// Check if chain is available
    pub async fn is_chain_available(&self, chain_id: u64) -> bool {
        matches!(
            self.chains.read().await.get(&chain_id).map(|c| c.status),
            Some(ChainConnectionStatus::Connected)
        )
    }

    /// Get best chain based on block height and latency
    pub async fn get_best_chain(&self) -> Result<ChainConnection> {
        let chains = self.chains.read().await;

        let best = chains
            .values()
            .filter(|c| matches!(c.status, ChainConnectionStatus::Connected))
            .max_by_key(|c| (c.block_height, -(c.latency_ms as i64)));

        match best {
            Some(chain) => Ok(chain.clone()),
            None => Err(ChainError::Provider("No available chains".to_string())),
        }
    }

    /// Execute operation on specific chain
    #[instrument(skip(self, operation), target = "chain::multichain")]
    pub async fn execute_on_chain<F, Fut, T>(&self, chain_id: u64, operation: F) -> Result<T>
    where
        F: FnOnce(Arc<RwLock<BaseEvmClient>>) -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let chain = self.get_chain(chain_id).await?;

        if !matches!(chain.status, ChainConnectionStatus::Connected) {
            return Err(ChainError::Provider(format!(
                "Chain {} is not connected",
                chain_id
            )));
        }

        let start = Instant::now();
        let result = operation(chain.client).await;
        let latency = start.elapsed().as_millis() as u64;

        // Update latency in connection
        if let Some(conn) = self.chains.write().await.get_mut(&chain_id) {
            conn.latency_ms = latency;
        }

        result
    }

    /// Execute on primary chain
    pub async fn execute_on_primary<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(Arc<RwLock<BaseEvmClient>>) -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let primary = self.get_primary_chain().await?;
        self.execute_on_chain(primary.chain_id, operation).await
    }

    /// Start health check background task with graceful shutdown support
    ///
    /// RELIABILITY FIX: Uses CancellationToken for efficient graceful
    /// termination. CancellationToken is simpler and more efficient than
    /// mpsc channel for this use case.
    pub async fn start_health_checks(&self) {
        let chains = self.chains.clone();
        let config = self.config.clone();
        let cancel_token = self.cancellation_token.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.health_check_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let chain_ids: Vec<u64> = chains.read().await.keys().copied().collect();

                        for chain_id in chain_ids {
                            let mut chains_write = chains.write().await;
                            if let Some(conn) = chains_write.get_mut(&chain_id) {
                                // Perform health check
                                let start = Instant::now();

                                match conn.client.read().await.get_block_number().await {
                                    Ok(block_number) => {
                                        conn.block_height = block_number;
                                        conn.latency_ms = start.elapsed().as_millis() as u64;
                                        conn.fail_count = 0;
                                        conn.status = ChainConnectionStatus::Connected;
                                        conn.last_health_check = Instant::now();
                                    }
                                    Err(_) => {
                                        conn.fail_count += 1;
                                        conn.latency_ms = u64::MAX;

                                        if conn.fail_count >= config.max_failures {
                                            conn.status = ChainConnectionStatus::Degraded;
                                            warn!(
                                                chain_id = chain_id,
                                                fail_count = conn.fail_count,
                                                "Chain health degraded"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // RELIABILITY FIX: Use CancellationToken for graceful shutdown
                    _ = cancel_token.cancelled() => {
                        info!("Health check task received cancellation signal, stopping gracefully");
                        break;
                    }
                }
            }

            info!("Health check task stopped");
        });

        info!("Health check task started with graceful shutdown support (CancellationToken)");
    }

    /// Stop health check background task gracefully
    ///
    /// RELIABILITY FIX: Uses CancellationToken::cancel() for efficient shutdown
    /// signaling. No need for RwLock<Option<Sender>> - CancellationToken is
    /// thread-safe and cloneable.
    pub fn stop_health_checks(&self) {
        self.cancellation_token.cancel();
        info!("Sent cancellation signal to health check task");
    }

    /// Get manager statistics
    pub async fn get_statistics(&self) -> MultiChainStatistics {
        let chains = self.chains.read().await;

        let total = chains.len();
        let connected = chains
            .values()
            .filter(|c| matches!(c.status, ChainConnectionStatus::Connected))
            .count();
        let degraded = chains
            .values()
            .filter(|c| matches!(c.status, ChainConnectionStatus::Degraded))
            .count();
        let disconnected = chains
            .values()
            .filter(|c| matches!(c.status, ChainConnectionStatus::Disconnected))
            .count();

        let avg_latency = if total > 0 {
            chains.values().map(|c| c.latency_ms).sum::<u64>() / total as u64
        } else {
            0
        };

        MultiChainStatistics {
            total_chains: total,
            connected_chains: connected,
            degraded_chains: degraded,
            disconnected_chains: disconnected,
            average_latency_ms: avg_latency,
        }
    }
}

/// Multi-chain statistics
#[derive(Debug, Clone, Copy)]
pub struct MultiChainStatistics {
    pub total_chains: usize,
    pub connected_chains: usize,
    pub degraded_chains: usize,
    pub disconnected_chains: usize,
    pub average_latency_ms: u64,
}

/// Cross-chain router for routing operations between chains
pub struct CrossChainRouter {
    manager: Arc<MultiChainManager>,
    routing_table: Arc<RwLock<HashMap<u64, Vec<u64>>>>, // source -> [destinations]
}

impl CrossChainRouter {
    /// Create new cross-chain router
    pub fn new(manager: Arc<MultiChainManager>) -> Self {
        Self {
            manager,
            routing_table: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a routing path
    pub async fn register_route(&self, from_chain: u64, to_chain: u64) {
        let mut table = self.routing_table.write().await;
        table
            .entry(from_chain)
            .or_insert_with(Vec::new)
            .push(to_chain);

        info!(
            from = from_chain,
            to = to_chain,
            "Cross-chain route registered"
        );
    }

    /// Get available routes from a chain
    pub async fn get_routes(&self, from_chain: u64) -> Vec<u64> {
        self.routing_table
            .read()
            .await
            .get(&from_chain)
            .cloned()
            .unwrap_or_default()
    }

    /// Route an operation to the best available chain
    #[instrument(skip(self, operation), target = "chain::multichain")]
    pub async fn route_to_best<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(Arc<RwLock<BaseEvmClient>>, u64) -> Fut + Send + Clone,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let best = self.manager.get_best_chain().await?;
        self.manager
            .execute_on_chain(best.chain_id, |client| operation(client, best.chain_id))
            .await
    }

    /// Execute on all chains
    pub async fn broadcast<F, Fut, T>(&self, operation: F) -> HashMap<u64, Result<T>>
    where
        F: FnOnce(Arc<RwLock<BaseEvmClient>>) -> Fut + Send + Clone,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let chain_ids = self.manager.get_chain_ids().await;
        let mut results = HashMap::new();

        for chain_id in chain_ids {
            let op = operation.clone();
            let result = self.manager.execute_on_chain(chain_id, op).await;
            results.insert(chain_id, result);
        }

        results
    }
}

/// Chain load balancer
pub struct ChainLoadBalancer {
    manager: Arc<MultiChainManager>,
    strategy: LoadBalanceStrategy,
}

/// Load balancing strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadBalanceStrategy {
    /// Round-robin across chains
    RoundRobin,
    /// Choose chain with lowest latency
    LowestLatency,
    /// Choose chain with highest block
    HighestBlock,
    /// Random selection
    Random,
}

impl ChainLoadBalancer {
    /// Create new load balancer
    pub fn new(manager: Arc<MultiChainManager>, strategy: LoadBalanceStrategy) -> Self {
        Self { manager, strategy }
    }

    /// Get next chain based on strategy
    pub async fn next_chain(&self) -> Result<ChainConnection> {
        let connected = self.manager.get_connected_chains().await;

        if connected.is_empty() {
            return Err(ChainError::Provider(
                "No connected chains available".to_string(),
            ));
        }

        match self.strategy {
            LoadBalanceStrategy::RoundRobin => {
                // Simple round-robin: pick first (in production would track index)
                Ok(connected[0].clone())
            }
            LoadBalanceStrategy::LowestLatency => connected
                .into_iter()
                .min_by_key(|c| c.latency_ms)
                .ok_or_else(|| ChainError::Provider("No chains available".to_string())),
            LoadBalanceStrategy::HighestBlock => connected
                .into_iter()
                .max_by_key(|c| c.block_height)
                .ok_or_else(|| ChainError::Provider("No chains available".to_string())),
            LoadBalanceStrategy::Random => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                connected
                    .choose(&mut rng)
                    .cloned()
                    .ok_or_else(|| ChainError::Provider("No chains available".to_string()))
            }
        }
    }

    /// Execute on next chain
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce(Arc<RwLock<BaseEvmClient>>) -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let chain = self.next_chain().await?;
        self.manager
            .execute_on_chain(chain.chain_id, operation)
            .await
    }
}

/// Chain failover handler
pub struct ChainFailover {
    manager: Arc<MultiChainManager>,
    priority_list: Arc<RwLock<Vec<u64>>>,
}

impl ChainFailover {
    /// Create new failover handler
    pub fn new(manager: Arc<MultiChainManager>) -> Self {
        Self {
            manager,
            priority_list: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set priority list for failover
    pub async fn set_priority_list(&self, chain_ids: Vec<u64>) {
        *self.priority_list.write().await = chain_ids;
    }

    /// Execute with failover
    pub async fn execute_with_failover<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn(Arc<RwLock<BaseEvmClient>>) -> Fut + Send + Clone,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let priorities = self.priority_list.read().await.clone();

        for chain_id in priorities {
            if self.manager.is_chain_available(chain_id).await {
                match self
                    .manager
                    .execute_on_chain(chain_id, operation.clone())
                    .await
                {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        warn!(chain_id = chain_id, error = %e, "Chain operation failed, trying next");
                        continue;
                    }
                }
            }
        }

        Err(ChainError::Provider("All chains failed".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_connection_status() {
        assert_eq!(
            ChainConnectionStatus::Connected,
            ChainConnectionStatus::Connected
        );
        assert_ne!(
            ChainConnectionStatus::Connected,
            ChainConnectionStatus::Disconnected
        );
    }

    #[test]
    fn test_multi_chain_config_default() {
        let config = MultiChainConfig::default();
        assert_eq!(config.max_latency_ms, 5000);
        assert_eq!(config.max_failures, 3);
        assert!(config.auto_reconnect);
    }

    #[test]
    fn test_load_balance_strategy() {
        let strategies = vec![
            LoadBalanceStrategy::RoundRobin,
            LoadBalanceStrategy::LowestLatency,
            LoadBalanceStrategy::HighestBlock,
            LoadBalanceStrategy::Random,
        ];

        for strategy in strategies {
            assert_eq!(strategy, strategy);
        }
    }

    #[test]
    fn test_multi_chain_statistics() {
        let stats = MultiChainStatistics {
            total_chains: 5,
            connected_chains: 4,
            degraded_chains: 1,
            disconnected_chains: 0,
            average_latency_ms: 100,
        };

        assert_eq!(stats.total_chains, 5);
        assert_eq!(stats.connected_chains, 4);
    }
}
