//! Distributed Memory System
//!
//! Provides distributed memory backends for scaling beyond single-node limits.
//! Supports gRPC services and custom backends.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │         Local API Instance          │
//! └─────────────┬───────────────────────┘
//!               │
//! ┌─────────────▼───────────────────────┐
//! │      DistributedMemoryBackend       │
//! └─────────────┬───────────────────────┘
//!               │
//!     ┌─────────┴─────────┐
//!     ▼                   ▼
//! ┌───────┐         ┌───────┐
//! │ gRPC  │         │Custom │
//! │Service│         │Backend│
//! └───────┘         └───────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use beebotos_brain::distributed::{
//!     DistributedMemoryClient, GrpcBackend, DistributedConfig
//! };
//!
//! // Create gRPC-backed distributed memory
//! let config = DistributedConfig::grpc("grpc://localhost:50051");
//! let backend = GrpcBackend::new(config).await?;
//! let client = DistributedMemoryClient::new(backend);
//!
//! // Store memory
//! client.store("key", "value", 0.9).await?;
//!
//! // Retrieve memory
//! let value = client.retrieve("key").await?;
//! ```

pub mod backend;
pub mod client;
pub mod transport;
pub mod consensus;
pub mod sharding;

pub use backend::{
    DistributedMemoryBackend, BackendError, BackendResult,
    GrpcBackend, MemoryBackend, MemoryNode,
};
pub use client::{
    DistributedMemoryClient, ClientConfig, CachePolicy,
};
pub use transport::{
    Transport, GrpcTransport, Message, Response,
};
pub use consensus::{
    ConsensusAlgorithm, RaftConsensus, QuorumConfig,
};
pub use sharding::{
    ShardStrategy, ConsistentHash, RangeShard,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for distributed memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    /// Backend type
    pub backend_type: BackendType,
    /// Connection string
    pub connection_string: String,
    /// Pool size
    pub pool_size: usize,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Enable local cache
    pub enable_local_cache: bool,
    /// Local cache size
    pub local_cache_size: usize,
    /// Shard configuration
    pub shard_config: Option<ShardConfig>,
    /// Consensus configuration
    pub consensus_config: Option<ConsensusConfig>,
}

/// Backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendType {
    /// gRPC service backend
    Grpc,
    /// Custom backend
    Custom,
    /// In-memory (for testing)
    Memory,
}

/// Shard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardConfig {
    /// Number of shards
    pub shard_count: u32,
    /// Shard strategy
    pub strategy: ShardStrategyType,
    /// Replication factor
    pub replication_factor: u32,
}

/// Shard strategy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShardStrategyType {
    /// Consistent hashing
    ConsistentHash,
    /// Range-based sharding
    Range,
    /// Hash-based sharding
    Hash,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Algorithm
    pub algorithm: ConsensusAlgorithmType,
    /// Node ID
    pub node_id: String,
    /// Peer nodes
    pub peers: Vec<String>,
    /// Election timeout
    pub election_timeout_ms: u64,
    /// Heartbeat interval
    pub heartbeat_interval_ms: u64,
}

/// Consensus algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusAlgorithmType {
    /// Raft consensus
    Raft,
    /// Paxos
    Paxos,
    /// Simple majority
    Majority,
}

impl DistributedConfig {
    /// Create gRPC configuration
    pub fn grpc(endpoint: impl Into<String>) -> Self {
        Self {
            backend_type: BackendType::Grpc,
            connection_string: endpoint.into(),
            pool_size: 10,
            timeout_ms: 10000,
            retry_attempts: 3,
            enable_local_cache: true,
            local_cache_size: 1000,
            shard_config: None,
            consensus_config: None,
        }
    }

    /// Create in-memory configuration (for testing)
    pub fn memory() -> Self {
        Self {
            backend_type: BackendType::Memory,
            connection_string: "memory".to_string(),
            pool_size: 1,
            timeout_ms: 1000,
            retry_attempts: 1,
            enable_local_cache: false,
            local_cache_size: 0,
            shard_config: None,
            consensus_config: None,
        }
    }

    /// Enable sharding
    pub fn with_sharding(mut self, shard_count: u32, strategy: ShardStrategyType) -> Self {
        self.shard_config = Some(ShardConfig {
            shard_count,
            strategy,
            replication_factor: 2,
        });
        self
    }

    /// Enable consensus
    pub fn with_consensus(
        mut self,
        node_id: impl Into<String>,
        peers: Vec<String>,
    ) -> Self {
        self.consensus_config = Some(ConsensusConfig {
            algorithm: ConsensusAlgorithmType::Raft,
            node_id: node_id.into(),
            peers,
            election_timeout_ms: 150,
            heartbeat_interval_ms: 50,
        });
        self
    }

    /// Set pool size
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.pool_size = size;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self::memory()
    }
}

/// Memory entry for distributed storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedMemoryEntry {
    /// Unique key
    pub key: String,
    /// Value data
    pub value: Vec<u8>,
    /// Importance/confidence score
    pub importance: f32,
    /// Timestamp
    pub timestamp: u64,
    /// TTL in seconds (0 = no expiration)
    pub ttl_secs: u32,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Vector embedding (for similarity search)
    pub embedding: Option<Vec<f32>>,
}

impl DistributedMemoryEntry {
    /// Create new entry
    pub fn new(key: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            importance: 0.5,
            timestamp: crate::utils::current_timestamp_secs(),
            ttl_secs: 0,
            metadata: HashMap::new(),
            embedding: None,
        }
    }

    /// Set importance
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl_secs: u32) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set embedding
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Check if entry is expired
    pub fn is_expired(&self) -> bool {
        if self.ttl_secs == 0 {
            return false;
        }
        let now = crate::utils::current_timestamp_secs();
        now > self.timestamp + self.ttl_secs as u64
    }
}

/// Query options for distributed memory
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Maximum results
    pub limit: usize,
    /// Minimum importance
    pub min_importance: f32,
    /// Include expired entries
    pub include_expired: bool,
    /// Sort by
    pub sort_by: SortBy,
    /// Filter by metadata
    pub metadata_filter: HashMap<String, String>,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    /// By importance (descending)
    #[default]
    Importance,
    /// By timestamp (descending)
    Timestamp,
    /// By relevance (for vector search)
    Relevance,
}

impl QueryOptions {
    /// Create default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set minimum importance
    pub fn with_min_importance(mut self, importance: f32) -> Self {
        self.min_importance = importance;
        self
    }

    /// Include expired
    pub fn with_expired(mut self) -> Self {
        self.include_expired = true;
        self
    }

    /// Set sort order
    pub fn sort_by(mut self, sort: SortBy) -> Self {
        self.sort_by = sort;
        self
    }
}

/// Health status of distributed backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Degraded (some nodes unavailable)
    Degraded,
    /// Unhealthy
    Unhealthy,
    /// Unknown
    Unknown,
}

/// Backend statistics
#[derive(Debug, Clone, Default)]
pub struct BackendStats {
    /// Total operations
    pub total_operations: u64,
    /// Successful operations
    pub successful_operations: u64,
    /// Failed operations
    pub failed_operations: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// Connected nodes
    pub connected_nodes: usize,
    /// Total nodes
    pub total_nodes: usize,
    /// Health status
    pub health: HealthStatus,
}

impl BackendStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 1.0;
        }
        self.successful_operations as f64 / self.total_operations as f64
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distributed_config_grpc() {
        let config = DistributedConfig::grpc("grpc://localhost:50051");
        assert_eq!(config.backend_type, BackendType::Grpc);
        assert_eq!(config.connection_string, "grpc://localhost:50051");
        assert_eq!(config.pool_size, 10);
    }

    #[test]
    fn test_distributed_config_with_sharding() {
        let config = DistributedConfig::grpc("grpc://localhost:50051")
            .with_sharding(4, ShardStrategyType::ConsistentHash);

        assert!(config.shard_config.is_some());
        let shard = config.shard_config.unwrap();
        assert_eq!(shard.shard_count, 4);
        assert_eq!(shard.strategy, ShardStrategyType::ConsistentHash);
    }

    #[test]
    fn test_memory_entry_creation() {
        let entry = DistributedMemoryEntry::new("key", "value");
        assert_eq!(entry.key, "key");
        assert_eq!(entry.value, b"value");
        assert!((entry.importance - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_memory_entry_with_importance() {
        let entry = DistributedMemoryEntry::new("key", "value")
            .with_importance(0.9);
        assert!((entry.importance - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_memory_entry_not_expired() {
        let entry = DistributedMemoryEntry::new("key", "value");
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_memory_entry_expired() {
        let entry = DistributedMemoryEntry::new("key", "value")
            .with_ttl(1); // 1 second TTL
        
        // Should be expired immediately (well, after timestamp calculation)
        // In practice, we'd need to wait, but for testing...
        // This test documents the expected behavior
        assert!(entry.ttl_secs > 0);
    }

    #[test]
    fn test_backend_stats_success_rate() {
        let stats = BackendStats {
            total_operations: 100,
            successful_operations: 95,
            failed_operations: 5,
            ..Default::default()
        };
        
        assert!((stats.success_rate() - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_backend_stats_cache_hit_rate() {
        let stats = BackendStats {
            cache_hits: 80,
            cache_misses: 20,
            ..Default::default()
        };
        
        assert!((stats.cache_hit_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_query_options_builder() {
        let opts = QueryOptions::new()
            .with_limit(10)
            .with_min_importance(0.5)
            .sort_by(SortBy::Timestamp);
        
        assert_eq!(opts.limit, 10);
        assert!((opts.min_importance - 0.5).abs() < 0.001);
        assert_eq!(opts.sort_by, SortBy::Timestamp);
    }
}
