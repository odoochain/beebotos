//! Indexer Module

use crate::compat::B256;

/// Indexer configuration
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub start_block: u64,
    pub batch_size: u64,
    pub poll_interval_secs: u64,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            start_block: 0,
            batch_size: 1000,
            poll_interval_secs: 10,
        }
    }
}

/// Indexer state
#[derive(Debug, Clone)]
pub struct IndexerState {
    pub last_indexed_block: u64,
    pub last_indexed_hash: B256,
}

/// Block indexer
pub struct BlockIndexer {
    config: IndexerConfig,
    state: IndexerState,
}

impl BlockIndexer {
    pub fn new(config: IndexerConfig) -> Self {
        let state = IndexerState {
            last_indexed_block: config.start_block,
            last_indexed_hash: B256::ZERO,
        };
        Self { config, state }
    }
    
    pub fn state(&self) -> &IndexerState {
        &self.state
    }
    
    pub async fn index_batch(&mut self) -> anyhow::Result<u64> {
        // Implementation will be added
        Ok(0)
    }
}

pub mod queries;
pub mod sync;
