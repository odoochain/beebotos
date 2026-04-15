//! RocksDB Storage Backend
//!
//! High-performance persistent key-value storage using RocksDB.
//! Requires `rocksdb` feature to be enabled.

#[cfg(feature = "rocksdb")]
use std::path::Path;

#[cfg(feature = "rocksdb")]
use crate::storage::{serialization, StorageBackend, StorageError};

/// RocksDB storage backend
#[cfg(feature = "rocksdb")]
pub struct RocksDbStorage {
    db: rocksdb::DB,
    write_opts: rocksdb::WriteOptions,
}

#[cfg(feature = "rocksdb")]
impl std::fmt::Debug for RocksDbStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDbStorage")
            .field("path", &"<DB>")
            .field("write_opts", &"<WriteOptions>")
            .finish()
    }
}

#[cfg(feature = "rocksdb")]
impl RocksDbStorage {
    /// Open RocksDB at given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        Self::open_with_opts(path, Self::default_opts())
    }

    /// Open with custom options
    pub fn open_with_opts<P: AsRef<Path>>(
        path: P,
        opts: rocksdb::Options,
    ) -> Result<Self, StorageError> {
        let db = rocksdb::DB::open(&opts, path)
            .map_err(|e| StorageError::IoError(format!("Failed to open RocksDB: {}", e)))?;

        let write_opts = rocksdb::WriteOptions::default();

        Ok(Self { db, write_opts })
    }

    /// Create with default options
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let opts = Self::default_opts();
        Self::open_with_opts(path, opts)
    }

    /// Create with sync writes enabled/disabled
    pub fn with_sync_writes<P: AsRef<Path>>(path: P, sync: bool) -> Result<Self, StorageError> {
        let mut storage = Self::open(path)?;
        let mut write_opts = rocksdb::WriteOptions::default();
        write_opts.set_sync(sync);
        storage.write_opts = write_opts;
        Ok(storage)
    }

    /// Get default RocksDB options
    fn default_opts() -> rocksdb::Options {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_compression_per_level(&[
            rocksdb::DBCompressionType::None,
            rocksdb::DBCompressionType::None,
            rocksdb::DBCompressionType::Lz4,
            rocksdb::DBCompressionType::Lz4,
            rocksdb::DBCompressionType::Lz4,
            rocksdb::DBCompressionType::Lz4,
            rocksdb::DBCompressionType::Lz4,
        ]);
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB write buffer
        opts.set_max_write_buffer_number(3);
        opts.set_target_file_size_base(64 * 1024 * 1024);
        opts.set_max_background_jobs(4);
        opts
    }

    /// Set cache size in MB
    pub fn set_cache_size_mb(&mut self, size_mb: usize) {
        // Cache is set during DB creation, this is a placeholder
        // In production, you'd need to create a new DB with updated options
        let _ = size_mb;
    }

    /// Flush data to disk
    pub fn flush(&self) -> Result<(), StorageError> {
        self.db
            .flush()
            .map_err(|e| StorageError::IoError(format!("Failed to flush RocksDB: {}", e)))
    }

    /// Get database property
    pub fn get_property(&self, property: &str) -> Option<String> {
        self.db.property_value(property)
    }

    /// Create checkpoint (backup)
    pub fn create_checkpoint<P: AsRef<Path>>(&self, path: P) -> Result<(), StorageError> {
        let checkpoint = rocksdb::checkpoint::Checkpoint::new(&self.db)
            .map_err(|e| StorageError::IoError(format!("Failed to create checkpoint: {}", e)))?;
        checkpoint
            .create_checkpoint(path)
            .map_err(|e| StorageError::IoError(format!("Failed to save checkpoint: {}", e)))
    }
}

#[cfg(feature = "rocksdb")]
impl StorageBackend for RocksDbStorage {
    fn put(
        &self,
        key: &str,
        data: &[u8],
        metadata: crate::storage::EntryMetadata,
    ) -> Result<(), StorageError> {
        // Use shared serialization helper
        let serialized = serialization::serialize_entry(key, data, metadata)?;

        self.db
            .put_opt(key, serialized, &self.write_opts)
            .map_err(|e| StorageError::IoError(format!("RocksDB put failed: {}", e)))
    }

    fn get(&self, key: &str) -> Result<Option<crate::storage::StorageEntry>, StorageError> {
        match self
            .db
            .get(key)
            .map_err(|e| StorageError::IoError(format!("RocksDB get failed: {}", e)))?
        {
            Some(data) => {
                // Use shared deserialization helper
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
        // Use RocksDB's native key_may_exist for better performance
        Ok(self.db.key_may_exist(key))
    }
}

// Generate stub implementation using macro
crate::define_stub_backend!(RocksDbStorage, "rocksdb");

#[cfg(test)]
#[cfg(feature = "rocksdb")]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::storage::test_utils::create_test_metadata;

    #[test]
    fn test_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = RocksDbStorage::create(temp_dir.path()).unwrap();
        let metadata = create_test_metadata();

        // Test put and get
        storage.put("key1", b"value1", metadata.clone()).unwrap();
        let entry = storage.get("key1").unwrap().unwrap();
        assert_eq!(entry.data, b"value1");

        // Test exists
        assert!(storage.exists("key1").unwrap());

        // Test delete
        storage.delete("key1").unwrap();
        assert!(!storage.exists("key1").unwrap());
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        let metadata = create_test_metadata();

        // Create and write
        {
            let storage = RocksDbStorage::create(path).unwrap();
            storage.put("persistent", b"data", metadata).unwrap();
            storage.flush().unwrap();
        }

        // Reopen and verify
        {
            let storage = RocksDbStorage::open(path).unwrap();
            let entry = storage.get("persistent").unwrap().unwrap();
            assert_eq!(entry.data, b"data");
        }
    }
}
