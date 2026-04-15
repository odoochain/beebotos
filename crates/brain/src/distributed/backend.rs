//! Distributed Backend Implementations
//!
//! Provides concrete implementations of distributed memory backends.

use super::{
    DistributedConfig, DistributedMemoryEntry, QueryOptions, BackendStats, HealthStatus,
    BackendType, BackendResult, BackendError,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Trait for distributed memory backends
#[async_trait]
pub trait DistributedMemoryBackend: Send + Sync {
    /// Initialize the backend
    async fn initialize(&mut self) -> BackendResult<()>;
    
    /// Store an entry
    async fn store(&self, entry: DistributedMemoryEntry) -> BackendResult<()>;
    
    /// Retrieve an entry by key
    async fn retrieve(&self, key: &str) -> BackendResult<Option<DistributedMemoryEntry>>;
    
    /// Delete an entry
    async fn delete(&self, key: &str) -> BackendResult<bool>;
    
    /// Query entries
    async fn query(&self, query: &str, options: QueryOptions) -> BackendResult<Vec<DistributedMemoryEntry>>;
    
    /// Search by vector similarity
    async fn vector_search(&self, vector: &[f32], top_k: usize) -> BackendResult<Vec<(DistributedMemoryEntry, f32)>>;
    
    /// Check health
    async fn health_check(&self) -> HealthStatus;
    
    /// Get statistics
    async fn stats(&self) -> BackendStats;
    
    /// Shutdown the backend
    async fn shutdown(&self) -> BackendResult<()>;
}

/// Backend error types
#[derive(Debug, Clone)]
pub enum BackendError {
    /// Connection error
    Connection(String),
    /// Timeout
    Timeout,
    /// Not found
    NotFound,
    /// Serialization error
    Serialization(String),
    /// Quorum not reached
    QuorumNotReached { required: usize, actual: usize },
    /// Node unavailable
    NodeUnavailable(String),
    /// Invalid configuration
    InvalidConfig(String),
    /// Operation not supported
    NotSupported,
    /// Other error
    Other(String),
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::Connection(msg) => write!(f, "Connection error: {}", msg),
            BackendError::Timeout => write!(f, "Operation timed out"),
            BackendError::NotFound => write!(f, "Entry not found"),
            BackendError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            BackendError::QuorumNotReached { required, actual } => {
                write!(f, "Quorum not reached: required {}, got {}", required, actual)
            }
            BackendError::NodeUnavailable(node) => write!(f, "Node unavailable: {}", node),
            BackendError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            BackendError::NotSupported => write!(f, "Operation not supported"),
            BackendError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for BackendError {}

/// Backend result type
pub type BackendResult<T> = Result<T, BackendError>;

/// In-memory backend for testing
#[derive(Debug)]
pub struct MemoryBackend {
    storage: Arc<RwLock<HashMap<String, DistributedMemoryEntry>>>,
    stats: Arc<RwLock<BackendStats>>,
}

impl MemoryBackend {
    /// Create new in-memory backend
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(BackendStats::default())),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DistributedMemoryBackend for MemoryBackend {
    async fn initialize(&mut self) -> BackendResult<()> {
        Ok(())
    }

    async fn store(&self, entry: DistributedMemoryEntry) -> BackendResult<()> {
        let mut storage = self.storage.write()
            .map_err(|_| BackendError::Other("Lock poisoned".to_string()))?;
        let mut stats = self.stats.write()
            .map_err(|_| BackendError::Other("Lock poisoned".to_string()))?;
        
        storage.insert(entry.key.clone(), entry);
        stats.total_operations += 1;
        stats.successful_operations += 1;
        
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> BackendResult<Option<DistributedMemoryEntry>> {
        let storage = self.storage.read()
            .map_err(|_| BackendError::Other("Lock poisoned".to_string()))?;
        let mut stats = self.stats.write()
            .map_err(|_| BackendError::Other("Lock poisoned".to_string()))?;
        
        let result = storage.get(key).cloned();
        
        stats.total_operations += 1;
        if result.is_some() {
            stats.successful_operations += 1;
            stats.cache_hits += 1;
        } else {
            stats.cache_misses += 1;
        }
        
        Ok(result)
    }

    async fn delete(&self, key: &str) -> BackendResult<bool> {
        let mut storage = self.storage.write()
            .map_err(|_| BackendError::Other("Lock poisoned".to_string()))?;
        let mut stats = self.stats.write()
            .map_err(|_| BackendError::Other("Lock poisoned".to_string()))?;
        
        let existed = storage.remove(key).is_some();
        stats.total_operations += 1;
        stats.successful_operations += 1;
        
        Ok(existed)
    }

    async fn query(&self, query: &str, options: QueryOptions) -> BackendResult<Vec<DistributedMemoryEntry>> {
        let storage = self.storage.read()
            .map_err(|_| BackendError::Other("Lock poisoned".to_string()))?;
        
        let query_lower = query.to_lowercase();
        let mut results: Vec<_> = storage
            .values()
            .filter(|entry| {
                // Check expiration
                if !options.include_expired && entry.is_expired() {
                    return false;
                }
                
                // Check importance
                if entry.importance < options.min_importance {
                    return false;
                }
                
                // Check content match
                let value_str = String::from_utf8_lossy(&entry.value);
                entry.key.to_lowercase().contains(&query_lower)
                    || value_str.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();
        
        // Sort
        match options.sort_by {
            super::SortBy::Importance => {
                results.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
            }
            super::SortBy::Timestamp => {
                results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            }
            super::SortBy::Relevance => {
                // Simple relevance: importance * recency
                results.sort_by(|a, b| {
                    let score_a = a.importance * (a.timestamp as f32);
                    let score_b = b.importance * (b.timestamp as f32);
                    score_b.partial_cmp(&score_a).unwrap()
                });
            }
        }
        
        // Apply limit
        if options.limit > 0 {
            results.truncate(options.limit);
        }
        
        Ok(results)
    }

    async fn vector_search(&self, _vector: &[f32], _top_k: usize) -> BackendResult<Vec<(DistributedMemoryEntry, f32)>> {
        // In-memory backend doesn't support vector search yet
        Ok(vec![])
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }

    async fn stats(&self) -> BackendStats {
        self.stats.read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    async fn shutdown(&self) -> BackendResult<()> {
        let mut storage = self.storage.write()
            .map_err(|_| BackendError::Other("Lock poisoned".to_string()))?;
        storage.clear();
        Ok(())
    }
}

/// gRPC backend (placeholder for actual implementation)
#[derive(Debug)]
pub struct GrpcBackend {
    config: DistributedConfig,
    stats: Arc<RwLock<BackendStats>>,
}

impl GrpcBackend {
    /// Create new gRPC backend
    pub fn new(config: DistributedConfig) -> Self {
        Self {
            config,
            stats: Arc::new(RwLock::new(BackendStats::default())),
        }
    }
}

#[async_trait]
impl DistributedMemoryBackend for GrpcBackend {
    async fn initialize(&mut self) -> BackendResult<()> {
        tracing::info!("Initializing gRPC backend: {}", self.config.connection_string);
        Ok(())
    }

    async fn store(&self, entry: DistributedMemoryEntry) -> BackendResult<()> {
        tracing::debug!("Storing key {} via gRPC", entry.key);
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> BackendResult<Option<DistributedMemoryEntry>> {
        tracing::debug!("Retrieving key {} via gRPC", key);
        Ok(None)
    }

    async fn delete(&self, key: &str) -> BackendResult<bool> {
        tracing::debug!("Deleting key {} via gRPC", key);
        Ok(true)
    }

    async fn query(&self, query: &str, options: QueryOptions) -> BackendResult<Vec<DistributedMemoryEntry>> {
        tracing::debug!("Querying via gRPC: {}", query);
        Ok(vec![])
    }

    async fn vector_search(&self, _vector: &[f32], _top_k: usize) -> BackendResult<Vec<(DistributedMemoryEntry, f32)>> {
        Ok(vec![])
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }

    async fn stats(&self) -> BackendStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    async fn shutdown(&self) -> BackendResult<()> {
        tracing::info!("Shutting down gRPC backend");
        Ok(())
    }
}

/// Memory node in a distributed cluster
#[derive(Debug, Clone)]
pub struct MemoryNode {
    /// Node ID
    pub id: String,
    /// Node address
    pub address: String,
    /// Node role
    pub role: NodeRole,
    /// Node status
    pub status: NodeStatus,
    /// Last heartbeat
    pub last_heartbeat: u64,
}

/// Node role
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    /// Leader node (for Raft)
    Leader,
    /// Follower node
    Follower,
    /// Candidate (during election)
    Candidate,
    /// Learner (read-only)
    Learner,
}

/// Node status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is healthy
    Healthy,
    /// Node is suspected to be down
    Suspected,
    /// Node is down
    Down,
    /// Node is joining
    Joining,
    /// Node is leaving
    Leaving,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_backend_basic() {
        let backend = MemoryBackend::new();
        
        // Store
        let entry = DistributedMemoryEntry::new("key1", "value1");
        backend.store(entry).await.unwrap();
        
        // Retrieve
        let retrieved = backend.retrieve("key1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().key, "key1");
        
        // Delete
        let deleted = backend.delete("key1").await.unwrap();
        assert!(deleted);
        
        let not_found = backend.retrieve("key1").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_memory_backend_query() {
        let backend = MemoryBackend::new();
        
        // Store multiple entries
        for i in 0..5 {
            let entry = DistributedMemoryEntry::new(
                format!("key{}", i),
                format!("value{}", i),
            ).with_importance(0.5 + (i as f32 * 0.1));
            backend.store(entry).await.unwrap();
        }
        
        // Query
        let results = backend.query("value", QueryOptions::new().with_limit(3)).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_memory_backend_stats() {
        let backend = MemoryBackend::new();
        
        let entry = DistributedMemoryEntry::new("key", "value");
        backend.store(entry).await.unwrap();
        backend.retrieve("key").await.unwrap();
        backend.retrieve("nonexistent").await.unwrap();
        
        let stats = backend.stats().await;
        assert!(stats.total_operations >= 3);
        assert!(stats.cache_hits >= 1);
        assert!(stats.cache_misses >= 1);
    }
}
