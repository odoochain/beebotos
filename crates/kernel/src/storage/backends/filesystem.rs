//! Filesystem Storage Backend
//!
//! Simple file-based storage backend that stores each key as a separate file.
//! Good for small datasets and debugging, not recommended for high-throughput
//! scenarios.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::storage::{EntryMetadata, StorageBackend, StorageEntry, StorageError};

/// Filesystem storage backend
#[derive(Debug)]
pub struct FilesystemStorage {
    base_path: PathBuf,
}

impl FilesystemStorage {
    /// Create new filesystem storage at given path
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self, StorageError> {
        let base_path = base_path.as_ref().to_path_buf();

        // Create base directory if it doesn't exist
        fs::create_dir_all(&base_path)
            .map_err(|e| StorageError::IoError(format!("Failed to create directory: {}", e)))?;

        Ok(Self { base_path })
    }

    /// Get file path for a key
    fn key_to_path(&self, key: &str) -> PathBuf {
        // Sanitize key to be safe for filesystem
        let sanitized = key.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        self.base_path.join(format!("{}.bin", sanitized))
    }

    /// Get metadata path for a key
    fn metadata_path(&self, key: &str) -> PathBuf {
        let sanitized = key.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        self.base_path.join(format!("{}.meta", sanitized))
    }

    /// List all keys in storage
    pub fn list_all(&self) -> Result<Vec<String>, StorageError> {
        let mut keys = Vec::new();

        let entries = fs::read_dir(&self.base_path)
            .map_err(|e| StorageError::IoError(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| StorageError::IoError(format!("Failed to read entry: {}", e)))?;

            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "bin" {
                    if let Some(stem) = path.file_stem() {
                        keys.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(keys)
    }

    /// Get storage directory size
    pub fn total_size(&self) -> Result<u64, StorageError> {
        let mut total = 0u64;

        let entries = fs::read_dir(&self.base_path)
            .map_err(|e| StorageError::IoError(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| StorageError::IoError(format!("Failed to read entry: {}", e)))?;

            let metadata = entry
                .metadata()
                .map_err(|e| StorageError::IoError(format!("Failed to get metadata: {}", e)))?;

            if metadata.is_file() {
                total += metadata.len();
            }
        }

        Ok(total)
    }

    /// Clean up old files based on age
    pub fn cleanup_old(&self, max_age_secs: u64) -> Result<usize, StorageError> {
        let mut removed = 0;
        let now = std::time::SystemTime::now();

        let entries = fs::read_dir(&self.base_path)
            .map_err(|e| StorageError::IoError(format!("Failed to read directory: {}", e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| StorageError::IoError(format!("Failed to read entry: {}", e)))?;

            let metadata = entry
                .metadata()
                .map_err(|e| StorageError::IoError(format!("Failed to get metadata: {}", e)))?;

            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age.as_secs() > max_age_secs {
                        fs::remove_file(entry.path()).map_err(|e| {
                            StorageError::IoError(format!("Failed to remove file: {}", e))
                        })?;
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }
}

impl StorageBackend for FilesystemStorage {
    fn put(
        &self,
        key: &str,
        data: &[u8],
        metadata: EntryMetadata,
    ) -> std::result::Result<(), StorageError> {
        let path = self.key_to_path(key);
        let meta_path = self.metadata_path(key);

        // Write data file
        let mut file = fs::File::create(&path)
            .map_err(|e| StorageError::IoError(format!("Failed to create file: {}", e)))?;

        file.write_all(data)
            .map_err(|e| StorageError::IoError(format!("Failed to write data: {}", e)))?;

        file.sync_all()
            .map_err(|e| StorageError::IoError(format!("Failed to sync file: {}", e)))?;

        // Write metadata file
        let meta_json = serde_json::to_string(&metadata)
            .map_err(|e| StorageError::IoError(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(&meta_path, meta_json)
            .map_err(|e| StorageError::IoError(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    fn get(&self, key: &str) -> std::result::Result<Option<StorageEntry>, StorageError> {
        let path = self.key_to_path(key);
        let meta_path = self.metadata_path(key);

        // Check if file exists
        if !path.exists() {
            return Ok(None);
        }

        // Read data
        let data = fs::read(&path)
            .map_err(|e| StorageError::IoError(format!("Failed to read file: {}", e)))?;

        // Read metadata (fallback to default if missing)
        let metadata = if meta_path.exists() {
            let meta_json = fs::read_to_string(&meta_path)
                .map_err(|e| StorageError::IoError(format!("Failed to read metadata: {}", e)))?;
            serde_json::from_str(&meta_json)
                .map_err(|e| StorageError::IoError(format!("Failed to parse metadata: {}", e)))?
        } else {
            EntryMetadata {
                created_at: 0,
                modified_at: 0,
                size_bytes: data.len() as u64,
                content_type: "application/octet-stream".to_string(),
                checksum: "".to_string(),
                tags: vec![],
            }
        };

        Ok(Some(StorageEntry {
            key: key.to_string(),
            data,
            metadata,
        }))
    }

    fn delete(&self, key: &str) -> std::result::Result<(), StorageError> {
        let path = self.key_to_path(key);
        let meta_path = self.metadata_path(key);

        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| StorageError::IoError(format!("Failed to delete file: {}", e)))?;
        }

        if meta_path.exists() {
            let _ = fs::remove_file(&meta_path);
        }

        Ok(())
    }

    fn list(&self, prefix: &str) -> std::result::Result<Vec<String>, StorageError> {
        let all_keys = self.list_all()?;
        // Sanitize prefix the same way keys are sanitized
        let sanitized_prefix = prefix.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        Ok(all_keys
            .into_iter()
            .filter(|k| k.starts_with(&sanitized_prefix))
            .collect())
    }

    fn exists(&self, key: &str) -> std::result::Result<bool, StorageError> {
        let path = self.key_to_path(key);
        Ok(path.exists())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::storage::test_utils::create_test_metadata;

    #[test]
    fn test_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FilesystemStorage::new(temp_dir.path()).unwrap();
        let metadata = create_test_metadata();

        // Test put and get
        storage.put("key1", b"value1", metadata.clone()).unwrap();
        let entry = storage.get("key1").unwrap().unwrap();
        assert_eq!(entry.data, b"value1");

        // Test exists
        assert!(storage.exists("key1").unwrap());
        assert!(!storage.exists("nonexistent").unwrap());

        // Test delete
        storage.delete("key1").unwrap();
        assert!(!storage.exists("key1").unwrap());
    }

    #[test]
    fn test_list_with_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FilesystemStorage::new(temp_dir.path()).unwrap();
        let metadata = create_test_metadata();

        storage.put("prefix:1", b"v1", metadata.clone()).unwrap();
        storage.put("prefix:2", b"v2", metadata.clone()).unwrap();
        storage.put("other", b"v3", metadata.clone()).unwrap();

        let keys = storage.list("prefix:").unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_key_sanitization() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FilesystemStorage::new(temp_dir.path()).unwrap();
        let metadata = create_test_metadata();

        // Key with special characters
        storage.put("key/with/slashes", b"value", metadata).unwrap();
        let entry = storage.get("key/with/slashes").unwrap().unwrap();
        assert_eq!(entry.data, b"value");
    }

    #[test]
    fn test_total_size() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FilesystemStorage::new(temp_dir.path()).unwrap();
        let metadata = create_test_metadata();

        storage.put("key1", b"value1", metadata.clone()).unwrap();
        storage.put("key2", b"value2", metadata).unwrap();

        let size = storage.total_size().unwrap();
        assert!(size > 0);
    }
}
