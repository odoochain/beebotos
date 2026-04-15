//! Memory Module
//!
//! Agent memory management.
//! 
//! 🟠 HIGH FIX: Memory entry size limits to prevent memory exhaustion.
//!
//! ## High Priority Features (OpenClaw Compatible)
//! - **Hybrid Search**: Vector + BM25 mixed search mechanism
//! - **Memory Flush**: Automatic memory persistence on context window limits
//! - **Markdown Storage**: File-based memory storage (File is Truth)

pub mod local;
pub mod qmd;
pub mod sync;
pub mod backup;
pub mod hybrid_search;
pub mod memory_flush;
pub mod markdown_storage;
pub mod hybrid_search_sqlite;
pub mod embedding;
pub mod markdown_search;
pub mod memory_flush_llm;

// 🟢 P1 FIX: Unified search interface
pub mod search;

pub use local::LocalMemory;
pub use hybrid_search::{
    HybridSearchEngine, HybridSearchConfig,
    DEFAULT_VECTOR_WEIGHT as HYBRID_DEFAULT_VECTOR_WEIGHT, 
    DEFAULT_BM25_WEIGHT as HYBRID_DEFAULT_BM25_WEIGHT, 
    DEFAULT_MAX_RESULTS as HYBRID_DEFAULT_MAX_RESULTS,
};
pub use memory_flush::{
    MemoryFlushManager, MemoryFlushConfig, FlushEvent, FlushTrigger,
    FlushStatistics, ImportanceAnalysis, MemoryCategory, FlushedMemoryEntry,
    ContextWindowState, DEFAULT_TOKEN_THRESHOLD, DEFAULT_FLUSH_INTERVAL_SECS,
};
pub use markdown_storage::{
    MarkdownStorage, MarkdownStorageConfig, MarkdownMemoryEntry,
    MemoryFileType, SearchMatch,
    CORE_MEMORY_FILE, USER_PROFILE_FILE, SOUL_FILE, 
    AGENTS_MANUAL_FILE, HEARTBEAT_FILE, MEMORY_SUBDIR,
};
pub use hybrid_search_sqlite::{
    HybridSearchSqlite, SqliteMemoryEntry, SqliteSearchResult,
    SearchDatabaseStats, DEFAULT_SEARCH_DB,
};
pub use embedding::{
    EmbeddingProvider, EmbeddingConfig, EmbeddingProviderFactory,
    ProviderType, CachedEmbeddingProvider,
    DEFAULT_EMBEDDING_DIMENSION, DEFAULT_EMBEDDING_TIMEOUT_SECS,
    MAX_EMBEDDING_TEXT_LENGTH,
};
pub use markdown_search::{
    UnifiedMemorySystem, UnifiedMemoryConfig, MemorySearchResult,
    MemoryFileWatcher, IndexingStats, IndexingProgressCallback,
};
pub use memory_flush_llm::{
    LLMMemoryFlushOrchestrator, LLMMemoryFlushConfig, LLMImportanceAnalysis,
    LLMProvider, OpenAILLMProvider, MockLLMProvider,
    CompressionResult, ConversationMessage,
};

// 🟢 P1 FIX: Unified search exports - these are the canonical types
pub use search::{
    MemorySearch, SearchConfig, SearchResult, SearchStats,
    VectorEmbedding, BM25IndexEntry,
    DEFAULT_VECTOR_WEIGHT, DEFAULT_BM25_WEIGHT, DEFAULT_MAX_RESULTS,
};

#[allow(unused_imports)]
use crate::error::{Result, AgentError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// CODE QUALITY FIX: Memory limits are now configurable via MemoryConfig
/// Default maximum content size: 1MB
pub const DEFAULT_MAX_CONTENT_SIZE: usize = 1024 * 1024;
/// Default maximum metadata entries: 100
pub const DEFAULT_MAX_METADATA_ENTRIES: usize = 100;
/// Default maximum metadata key/value size: 4KB each
pub const DEFAULT_MAX_METADATA_VALUE_SIZE: usize = 4096;

// Backward compatible aliases for code using the old constant names
pub const MAX_CONTENT_SIZE: usize = DEFAULT_MAX_CONTENT_SIZE;
pub const MAX_METADATA_ENTRIES: usize = DEFAULT_MAX_METADATA_ENTRIES;
pub const MAX_METADATA_VALUE_SIZE: usize = DEFAULT_MAX_METADATA_VALUE_SIZE;

/// Memory limits configuration for configurable limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimitsConfig {
    /// Maximum content size in bytes
    pub max_content_size: usize,
    /// Maximum metadata entries
    pub max_metadata_entries: usize,
    /// Maximum metadata value size in bytes
    pub max_metadata_value_size: usize,
}

impl Default for MemoryLimitsConfig {
    fn default() -> Self {
        Self {
            max_content_size: DEFAULT_MAX_CONTENT_SIZE,
            max_metadata_entries: DEFAULT_MAX_METADATA_ENTRIES,
            max_metadata_value_size: DEFAULT_MAX_METADATA_VALUE_SIZE,
        }
    }
}

impl MemoryLimitsConfig {
    /// Create configuration from environment variables
    /// 
    /// Environment variables:
    /// - `AGENT_MEMORY_MAX_CONTENT_SIZE`: Max content size in bytes (default: 1MB)
    /// - `AGENT_MEMORY_MAX_METADATA_ENTRIES`: Max metadata entries (default: 100)
    /// - `AGENT_MEMORY_MAX_METADATA_VALUE_SIZE`: Max metadata value size in bytes (default: 4KB)
    pub fn from_env() -> Self {
        use std::env;
        
        Self {
            max_content_size: env::var("AGENT_MEMORY_MAX_CONTENT_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_MAX_CONTENT_SIZE),
            max_metadata_entries: env::var("AGENT_MEMORY_MAX_METADATA_ENTRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_MAX_METADATA_ENTRIES),
            max_metadata_value_size: env::var("AGENT_MEMORY_MAX_METADATA_VALUE_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_MAX_METADATA_VALUE_SIZE),
        }
    }
}

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Memory validation error
#[derive(Debug, Clone, thiserror::Error)]
pub enum MemoryError {
    #[error("Content too large: {size} bytes (max: {max})")]
    ContentTooLarge { size: usize, max: usize },
    #[error("Too many metadata entries: {count} (max: {max})")]
    TooManyMetadataEntries { count: usize, max: usize },
    #[error("Metadata value too large for key '{key}': {size} bytes (max: {max})")]
    MetadataValueTooLarge { key: String, size: usize, max: usize },
}

impl MemoryEntry {
    /// 🟠 HIGH FIX: Validate memory entry size constraints
    pub fn validate(&self) -> std::result::Result<(), MemoryError> {
        // Check content size
        let content_size = self.content.len();
        if content_size > DEFAULT_MAX_CONTENT_SIZE {
            return Err(MemoryError::ContentTooLarge {
                size: content_size,
                max: DEFAULT_MAX_CONTENT_SIZE,
            });
        }
        
        // Check metadata entry count
        if self.metadata.len() > DEFAULT_MAX_METADATA_ENTRIES {
            return Err(MemoryError::TooManyMetadataEntries {
                count: self.metadata.len(),
                max: DEFAULT_MAX_METADATA_ENTRIES,
            });
        }
        
        // Check metadata value sizes
        for (key, value) in &self.metadata {
            if value.len() > DEFAULT_MAX_METADATA_VALUE_SIZE {
                return Err(MemoryError::MetadataValueTooLarge {
                    key: key.clone(),
                    size: value.len(),
                    max: DEFAULT_MAX_METADATA_VALUE_SIZE,
                });
            }
        }
        
        Ok(())
    }
    
    /// Calculate total size of this entry
    pub fn total_size(&self) -> usize {
        let content_size = self.content.len();
        let metadata_size: usize = self.metadata.iter()
            .map(|(k, v)| k.len() + v.len())
            .sum();
        content_size + metadata_size
    }
}
