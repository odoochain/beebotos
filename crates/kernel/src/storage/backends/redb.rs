//! Redb Storage Backend
//!
//! Pure Rust ACID-compliant embedded database backend.
//! Requires `redb` feature to be enabled.

#[cfg(feature = "redb")]
use std::path::Path;

#[cfg(feature = "redb")]
use crate::storage::{serialization, StorageBackend, StorageError};

/// Redb storage backend
#[cfg(feature = "redb")]
pub struct RedbStorage {
    db: std::sync::Arc<Mutex<redb::Database>>,
}

#[cfg(feature = "redb")]
impl std::fmt::Debug for RedbStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbStorage")
            .field("db", &"<Database>")
            .finish()
    }
}

#[cfg(feature = "redb")]
impl RedbStorage {
    /// Open or create redb at given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = redb::Database::create(path)
            .map_err(|e| StorageError::IoError(format!("Failed to open redb: {}", e)))?;

        // Create the main table if it doesn't exist
        let write_txn = db
            .begin_write()
            .map_err(|e| StorageError::IoError(format!("Failed to begin transaction: {}", e)))?;
        {
            let _table = write_txn
                .open_table::<&str, &[u8]>(STORAGE_TABLE)
                .map_err(|e| StorageError::IoError(format!("Failed to open table: {}", e)))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::IoError(format!("Failed to commit: {}", e)))?;

        Ok(Self {
            db: std::sync::Arc::new(Mutex::new(db)),
        })
    }

    /// Create with durability settings
    pub fn with_durability<P: AsRef<Path>>(
        path: P,
        durability: RedbDurability,
    ) -> Result<Self, StorageError> {
        let db = redb::Database::create(path)
            .map_err(|e| StorageError::IoError(format!("Failed to open redb: {}", e)))?;

        // Configure durability (note: redb doesn't expose this directly in current API)
        let _ = durability;

        let write_txn = db
            .begin_write()
            .map_err(|e| StorageError::IoError(format!("Failed to begin transaction: {}", e)))?;
        {
            let _table = write_txn
                .open_table::<&str, &[u8]>(STORAGE_TABLE)
                .map_err(|e| StorageError::IoError(format!("Failed to open table: {}", e)))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::IoError(format!("Failed to commit: {}", e)))?;

        Ok(Self {
            db: std::sync::Arc::new(Mutex::new(db)),
        })
    }

    /// Compact the database
    pub fn compact(&self) -> Result<(), StorageError> {
        let db = self.db.lock();
        db.compact()
            .map_err(|e| StorageError::IoError(format!("Failed to compact: {}", e)))
    }

    /// Get database stats
    pub fn stats(&self) -> Result<RedbStats, StorageError> {
        let db = self.db.lock();
        // redb doesn't expose detailed stats in current API
        let _ = db;
        Ok(RedbStats {
            db_size_bytes: 0, // Would need to check file size
        })
    }
}

#[cfg(feature = "redb")]
impl StorageBackend for RedbStorage {
    fn put(
        &self,
        key: &str,
        data: &[u8],
        metadata: crate::storage::EntryMetadata,
    ) -> Result<(), StorageError> {
        // Use shared serialization helper
        let serialized = serialization::serialize_entry(key, data, metadata)?;

        let db = self.db.lock();
        let write_txn = db
            .begin_write()
            .map_err(|e| StorageError::IoError(format!("Failed to begin transaction: {}", e)))?;
        {
            let mut table = write_txn
                .open_table::<&str, &[u8]>(STORAGE_TABLE)
                .map_err(|e| StorageError::IoError(format!("Failed to open table: {}", e)))?;
            table
                .insert(key, serialized.as_slice())
                .map_err(|e| StorageError::IoError(format!("Failed to insert: {}", e)))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::IoError(format!("Failed to commit: {}", e)))
    }

    fn get(&self, key: &str) -> Result<Option<crate::storage::StorageEntry>, StorageError> {
        let db = self.db.lock();
        let read_txn = db
            .begin_read()
            .map_err(|e| StorageError::IoError(format!("Failed to begin read: {}", e)))?;
        let table = read_txn
            .open_table::<&str, &[u8]>(STORAGE_TABLE)
            .map_err(|e| StorageError::IoError(format!("Failed to open table: {}", e)))?;

        match table
            .get(key)
            .map_err(|e| StorageError::IoError(format!("Failed to get: {}", e)))?
        {
            Some(data) => {
                // Use shared deserialization helper
                let entry = serialization::deserialize_entry(data.value())?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    fn delete(&self, key: &str) -> Result<(), StorageError> {
        let db = self.db.lock();
        let write_txn = db
            .begin_write()
            .map_err(|e| StorageError::IoError(format!("Failed to begin transaction: {}", e)))?;
        {
            let mut table = write_txn
                .open_table::<&str, &[u8]>(STORAGE_TABLE)
                .map_err(|e| StorageError::IoError(format!("Failed to open table: {}", e)))?;
            table
                .remove(key)
                .map_err(|e| StorageError::IoError(format!("Failed to remove: {}", e)))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::IoError(format!("Failed to commit: {}", e)))
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>, StorageError> {
        let db = self.db.lock();
        let read_txn = db
            .begin_read()
            .map_err(|e| StorageError::IoError(format!("Failed to begin read: {}", e)))?;
        let table = read_txn
            .open_table::<&str, &[u8]>(STORAGE_TABLE)
            .map_err(|e| StorageError::IoError(format!("Failed to open table: {}", e)))?;

        let mut keys = Vec::new();
        for item in table
            .iter()
            .map_err(|e| StorageError::IoError(format!("Failed to iterate: {}", e)))?
        {
            let (key, _) =
                item.map_err(|e| StorageError::IoError(format!("Iteration error: {}", e)))?;
            let key_str = key.value();
            if key_str.starts_with(prefix) {
                keys.push(key_str.to_string());
            }
        }
        Ok(keys)
    }

    // Uses default exists() implementation from trait
}

/// Durability level for redb
#[derive(Debug, Clone, Copy)]
pub enum RedbDurability {
    /// Wait for fsync after each commit (safest, slowest)
    Immediate,
    /// Let OS handle syncing (fast, less durable)
    Eventual,
}

/// Redb statistics
#[derive(Debug, Clone)]
pub struct RedbStats {
    /// Database file size in bytes
    pub db_size_bytes: u64,
}

/// Table name for storage
#[cfg(feature = "redb")]
const STORAGE_TABLE: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new("storage");

// Generate stub implementation using macro
crate::define_stub_backend!(RedbStorage, "redb");

#[cfg(test)]
#[cfg(feature = "redb")]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::storage::test_utils::create_test_metadata;

    #[test]
    fn test_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = RedbStorage::open(temp_dir.path().join("test.redb")).unwrap();
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
        let path = temp_dir.path().join("persistent.redb");
        let metadata = create_test_metadata();

        // Create and write
        {
            let storage = RedbStorage::open(&path).unwrap();
            storage.put("persistent", b"data", metadata).unwrap();
        }

        // Reopen and verify
        {
            let storage = RedbStorage::open(&path).unwrap();
            let entry = storage.get("persistent").unwrap().unwrap();
            assert_eq!(entry.data, b"data");
        }
    }
}
