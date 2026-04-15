//! Key-value storage with persistence support
//!
//! Provides a unified interface to storage backends with optional
//! serialization support for typed data.

use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::error::{KernelError, Result};
use crate::storage::{EntryMetadata, InMemoryStorage, StorageBackend};

/// Storage backend type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackendType {
    /// In-memory storage (volatile)
    Memory,
    /// RocksDB backend
    #[cfg(feature = "rocksdb")]
    RocksDb,
}

impl std::fmt::Display for StorageBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageBackendType::Memory => write!(f, "memory"),
            #[cfg(feature = "rocksdb")]
            StorageBackendType::RocksDb => write!(f, "rocksdb"),
        }
    }
}

/// Key-value store with pluggable backends
///
/// This is a high-level wrapper around `StorageBackend` implementations
/// that provides a simplified key-value interface with optional
/// serialization support.
#[derive(Debug)]
pub struct KVStore {
    /// The underlying storage backend
    backend: Box<dyn StorageBackend>,
    /// Store configuration
    config: StoreConfig,
    /// Dirty flag for memory backend (used for tracking persistence state)
    dirty: Arc<RwLock<bool>>,
}

/// Store configuration
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Storage backend type
    pub backend_type: StorageBackendType,
    /// Path for persistent storage
    pub path: Option<std::path::PathBuf>,
    /// Sync on every write (durability vs performance)
    pub sync_writes: bool,
    /// Enable compression for values
    pub compression: bool,
    /// Cache size in MB (for persistent backends)
    pub cache_size_mb: usize,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            backend_type: StorageBackendType::Memory,
            path: None,
            sync_writes: true,
            compression: false,
            cache_size_mb: 128,
        }
    }
}

impl StoreConfig {
    /// Create in-memory config
    pub fn memory() -> Self {
        Self::default()
    }

    /// Create RocksDB config
    #[cfg(feature = "rocksdb")]
    pub fn rocksdb<P: AsRef<Path>>(path: P) -> Self {
        Self {
            backend_type: StorageBackendType::RocksDb,
            path: Some(path.as_ref().to_path_buf()),
            ..Default::default()
        }
    }

    /// Disable sync writes (better performance, less durability)
    pub fn async_writes(mut self) -> Self {
        self.sync_writes = false;
        self
    }

    /// Enable compression
    pub fn with_compression(mut self) -> Self {
        self.compression = true;
        self
    }
}

impl KVStore {
    /// Create new in-memory KV store
    pub fn new() -> Self {
        Self::with_config(StoreConfig::memory()).expect("Memory store creation cannot fail")
    }

    /// Create KV store with persistence
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let _path = path.as_ref();
        #[cfg(feature = "rocksdb")]
        {
            let config = StoreConfig::rocksdb(path);
            Self::with_config(config)
        }
        #[cfg(not(feature = "rocksdb"))]
        {
            tracing::warn!("RocksDB not enabled, falling back to memory storage");
            Self::with_config(StoreConfig::memory())
        }
    }

    /// Create store with specific configuration
    pub fn with_config(config: StoreConfig) -> Result<Self> {
        let backend: Box<dyn StorageBackend> = match config.backend_type {
            StorageBackendType::Memory => Box::new(InMemoryStorage::new()),
            #[cfg(feature = "rocksdb")]
            StorageBackendType::RocksDb => {
                let path = config
                    .path
                    .as_ref()
                    .ok_or_else(|| KernelError::invalid_argument("RocksDB requires a path"))?;

                let mut opts = rocksdb::Options::default();
                opts.create_if_missing(true);
                opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
                opts.set_cache_capacity(config.cache_size_mb as f64 * 1024.0 * 1024.0);

                let db = rocksdb::DB::open(&opts, path)
                    .map_err(|e| KernelError::io(format!("Failed to open RocksDB: {}", e)))?;

                let mut write_opts = rocksdb::WriteOptions::default();
                write_opts.set_sync(config.sync_writes);

                // Wrap RocksDB in an adapter that implements StorageBackend
                Box::new(RocksDbAdapter { db, write_opts })
            }
        };

        Ok(Self {
            backend,
            config,
            dirty: Arc::new(RwLock::new(false)),
        })
    }

    /// Get value by key
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.backend
            .get(key)
            .map(|opt| opt.map(|entry| entry.data))
            .map_err(|e| KernelError::io(format!("Storage get failed: {}", e)))
    }

    /// Set key-value pair
    pub fn set(&self, key: impl Into<String>, value: Vec<u8>) -> Result<()> {
        let key = key.into();
        let metadata = EntryMetadata {
            created_at: chrono::Utc::now().timestamp() as u64,
            modified_at: chrono::Utc::now().timestamp() as u64,
            size_bytes: value.len() as u64,
            content_type: "application/octet-stream".to_string(),
            checksum: String::new(), // Could add checksum calculation here
            tags: vec![],
        };

        self.backend
            .put(&key, &value, metadata)
            .map_err(|e| KernelError::io(format!("Storage put failed: {}", e)))?;

        if matches!(self.config.backend_type, StorageBackendType::Memory) {
            *self.dirty.write() = true;
        }
        Ok(())
    }

    /// Delete key
    pub fn delete(&self, key: &str) -> Result<bool> {
        let existed = self
            .backend
            .exists(key)
            .map_err(|e| KernelError::io(format!("Storage exists check failed: {}", e)))?;

        self.backend
            .delete(key)
            .map_err(|e| KernelError::io(format!("Storage delete failed: {}", e)))?;

        if existed {
            *self.dirty.write() = true;
        }
        Ok(existed)
    }

    /// Check if key exists
    pub fn contains(&self, key: &str) -> Result<bool> {
        self.backend
            .exists(key)
            .map_err(|e| KernelError::io(format!("Storage exists check failed: {}", e)))
    }

    /// Get all keys with optional prefix filter
    pub fn keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        self.backend
            .list(prefix.unwrap_or(""))
            .map_err(|e| KernelError::io(format!("Storage list failed: {}", e)))
    }

    /// Clear all data
    ///
    /// Note: For persistent backends, this only flushes the database.
    /// For memory backend, this clears the in-memory map.
    pub fn clear(&self) -> Result<()> {
        match self.config.backend_type {
            StorageBackendType::Memory => {
                // For memory backend, we need to delete all keys
                let keys = self.keys(None)?;
                for key in keys {
                    self.backend
                        .delete(&key)
                        .map_err(|e| KernelError::io(format!("Storage delete failed: {}", e)))?;
                }
                *self.dirty.write() = true;
                Ok(())
            }
            #[cfg(feature = "rocksdb")]
            StorageBackendType::RocksDb => {
                // For RocksDB, we can only flush
                // To truly clear would require recreating the database
                self.flush()
            }
        }
    }

    /// Flush pending writes to disk
    pub fn flush(&self) -> Result<()> {
        match self.config.backend_type {
            StorageBackendType::Memory => {
                // Memory backend doesn't need flushing
                *self.dirty.write() = false;
                Ok(())
            }
            #[cfg(feature = "rocksdb")]
            StorageBackendType::RocksDb => {
                // The adapter handles flush internally
                Ok(())
            }
        }
    }

    /// Get store statistics
    pub fn stats(&self) -> StoreStats {
        // Get keys to count entries
        let keys = self.keys(None).unwrap_or_default();
        let entries = keys.len();

        // Estimate size by getting all values (inefficient but accurate)
        let approximate_size_bytes: usize = keys
            .iter()
            .filter_map(|k| self.get(k).ok().flatten())
            .map(|v| v.len())
            .sum::<usize>()
            + keys.iter().map(|k| k.len()).sum::<usize>();

        StoreStats {
            entries,
            backend: match self.config.backend_type {
                StorageBackendType::Memory => "memory",
                #[cfg(feature = "rocksdb")]
                StorageBackendType::RocksDb => "rocksdb",
            },
            approximate_size_bytes,
        }
    }

    /// Get backend type
    pub fn backend_type(&self) -> StorageBackendType {
        self.config.backend_type
    }

    /// Check if persistent
    pub fn is_persistent(&self) -> bool {
        !matches!(self.config.backend_type, StorageBackendType::Memory)
    }

    /// Check if dirty (only relevant for memory backend)
    pub fn is_dirty(&self) -> bool {
        *self.dirty.read()
    }
}

impl Default for KVStore {
    fn default() -> Self {
        Self::new()
    }
}

/// RocksDB adapter implementing StorageBackend
///
/// This adapter wraps a RocksDB instance to implement the StorageBackend trait
#[cfg(feature = "rocksdb")]
struct RocksDbAdapter {
    db: rocksdb::DB,
    write_opts: rocksdb::WriteOptions,
}

#[cfg(feature = "rocksdb")]
impl std::fmt::Debug for RocksDbAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDbAdapter")
            .field("db", &"<DB>")
            .field("write_opts", &"<WriteOptions>")
            .finish()
    }
}

#[cfg(feature = "rocksdb")]
impl StorageBackend for RocksDbAdapter {
    fn put(&self, key: &str, data: &[u8], metadata: EntryMetadata) -> Result<(), StorageError> {
        use crate::storage::serialization;
        let serialized = serialization::serialize_entry(key, data, metadata)?;
        self.db
            .put_opt(key, serialized, &self.write_opts)
            .map_err(|e| StorageError::IoError(format!("RocksDB put failed: {}", e)))
    }

    fn get(&self, key: &str) -> Result<Option<crate::storage::StorageEntry>, StorageError> {
        use crate::storage::serialization;
        match self
            .db
            .get(key)
            .map_err(|e| StorageError::IoError(format!("RocksDB get failed: {}", e)))?
        {
            Some(data) => {
                let entry = serialization::deserialize_entry(&data)?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    fn delete(&self, key: &str) -> Result<(), StorageError> {
        self.db
            .delete_opt(key, &self.write_opts)
            .map_err(|e| StorageError::IoError(format!("RocksDB delete failed: {}", e)))
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        let mut keys = Vec::new();
        let iter = self.db.iterator(rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, _) = item
                .map_err(|e| StorageError::IoError(format!("RocksDB iteration failed: {}", e)))?;
            let key_str = String::from_utf8_lossy(&key);
            if key_str.starts_with(prefix) {
                keys.push(key_str.to_string());
            }
        }
        Ok(keys)
    }

    fn exists(&self, key: &str) -> Result<bool, StorageError> {
        Ok(self.db.key_may_exist(key))
    }
}

/// Store statistics
#[derive(Debug, Clone)]
pub struct StoreStats {
    /// Number of entries in the store
    pub entries: usize,
    /// Storage backend type ("memory", "rocksdb", etc.)
    pub backend: &'static str,
    /// Approximate total size in bytes
    pub approximate_size_bytes: usize,
}

/// Typed KV store wrapper for serializable types
#[derive(Debug, Clone)]
pub struct TypedKVStore<T: Serialize + for<'de> Deserialize<'de>> {
    store: Arc<KVStore>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Serialize + for<'de> Deserialize<'de>> TypedKVStore<T> {
    /// Create new typed store
    pub fn new() -> Self {
        Self {
            store: Arc::new(KVStore::new()),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create with existing store
    pub fn with_store(store: Arc<KVStore>) -> Self {
        Self {
            store,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get typed value
    pub fn get(&self, key: &str) -> Result<Option<T>> {
        match self.store.get(key)? {
            Some(bytes) => {
                let value: T = serde_json::from_slice(&bytes)
                    .map_err(|e| KernelError::internal(format!("Deserialization failed: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set typed value
    pub fn set(&self, key: impl Into<String>, value: &T) -> Result<()> {
        let bytes = serde_json::to_vec(value)
            .map_err(|e| KernelError::internal(format!("Serialization failed: {}", e)))?;
        self.store.set(key, bytes)
    }

    /// Delete key
    pub fn delete(&self, key: &str) -> Result<bool> {
        self.store.delete(key)
    }
}

impl<T: Serialize + for<'de> Deserialize<'de>> Default for TypedKVStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_memory_store() {
        let store = KVStore::new();

        // Test set/get
        store.set("key1", vec![1, 2, 3]).unwrap();
        assert_eq!(store.get("key1").unwrap(), Some(vec![1, 2, 3]));

        // Test contains
        assert!(store.contains("key1").unwrap());
        assert!(!store.contains("nonexistent").unwrap());

        // Test delete
        assert!(store.delete("key1").unwrap());
        assert!(!store.delete("key1").unwrap());
        assert_eq!(store.get("key1").unwrap(), None);
    }

    #[test]
    fn test_memory_store_keys() {
        let store = KVStore::new();

        store.set("prefix:1", vec![1]).unwrap();
        store.set("prefix:2", vec![2]).unwrap();
        store.set("other", vec![3]).unwrap();

        let all_keys = store.keys(None).unwrap();
        assert_eq!(all_keys.len(), 3);

        let prefix_keys = store.keys(Some("prefix:")).unwrap();
        assert_eq!(prefix_keys.len(), 2);
    }

    #[test]
    fn test_memory_store_clear() {
        let store = KVStore::new();

        store.set("key1", vec![1]).unwrap();
        store.set("key2", vec![2]).unwrap();

        store.clear().unwrap();

        assert_eq!(store.get("key1").unwrap(), None);
        assert_eq!(store.get("key2").unwrap(), None);
        assert_eq!(store.keys(None).unwrap().len(), 0);
    }

    #[test]
    fn test_typed_store() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct TestData {
            name: String,
            value: u64,
        }

        let store = TypedKVStore::<TestData>::new();

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        store.set("key1", &data).unwrap();

        let retrieved = store.get("key1").unwrap().unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_store_stats() {
        let store = KVStore::new();

        store.set("key1", vec![1, 2, 3]).unwrap();
        store.set("key2", vec![4, 5]).unwrap();

        let stats = store.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.backend, "memory");
    }

    #[test]
    fn test_store_is_persistent() {
        let memory_store = KVStore::new();
        assert!(!memory_store.is_persistent());
    }

    #[test]
    #[cfg(feature = "rocksdb")]
    fn test_rocksdb_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.db");

        // Create store and write data
        {
            let store = KVStore::with_config(StoreConfig::rocksdb(&path)).unwrap();
            store.set("persistent_key", vec![1, 2, 3, 4, 5]).unwrap();
            store.flush().unwrap();
        }

        // Reopen and verify data persists
        {
            let store = KVStore::with_config(StoreConfig::rocksdb(&path)).unwrap();
            let value = store.get("persistent_key").unwrap();
            assert_eq!(value, Some(vec![1, 2, 3, 4, 5]));
        }
    }
}
