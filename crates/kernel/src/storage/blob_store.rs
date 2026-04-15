//! Blob storage for large binary objects

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::error::{KernelError, Result};

/// Blob identifier
pub type BlobId = String;

/// Blob metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobMetadata {
    /// Unique blob identifier
    pub id: BlobId,
    /// Size in bytes
    pub size: u64,
    /// MIME content type (if known)
    pub content_type: Option<String>,
    /// Creation timestamp (Unix seconds)
    pub created_at: u64,
}

/// Blob store for large binary objects
#[derive(Debug, Clone)]
pub struct BlobStore {
    /// Blob data
    blobs: Arc<RwLock<HashMap<BlobId, Vec<u8>>>>,
    /// Blob metadata
    metadata: Arc<RwLock<HashMap<BlobId, BlobMetadata>>>,
    /// Storage path - reserved for persistent storage implementation
    #[allow(dead_code)]
    _storage_path: Option<PathBuf>,
}

impl BlobStore {
    /// Create new in-memory blob store
    pub fn new() -> Self {
        Self {
            blobs: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            _storage_path: None,
        }
    }

    /// Create blob store with persistence
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            blobs: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            _storage_path: Some(path.as_ref().to_path_buf()),
        }
    }

    /// Store a blob
    pub fn put(&self, id: impl Into<String>, data: Vec<u8>) -> Result<BlobMetadata> {
        let id: String = id.into();
        let size = data.len() as u64;

        let mut blobs = self
            .blobs
            .write()
            .map_err(|_| KernelError::internal("Failed to acquire write lock"))?;
        blobs.insert(id.clone(), data);

        let metadata = BlobMetadata {
            id: id.clone(),
            size,
            content_type: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let mut meta = self
            .metadata
            .write()
            .map_err(|_| KernelError::internal("Failed to acquire write lock"))?;
        meta.insert(id, metadata.clone());

        Ok(metadata)
    }

    /// Retrieve a blob
    pub fn get(&self, id: &str) -> Result<Option<Vec<u8>>> {
        let blobs = self
            .blobs
            .read()
            .map_err(|_| KernelError::internal("Failed to acquire read lock"))?;
        Ok(blobs.get(id).cloned())
    }

    /// Get blob metadata
    pub fn get_metadata(&self, id: &str) -> Result<Option<BlobMetadata>> {
        let meta = self
            .metadata
            .read()
            .map_err(|_| KernelError::internal("Failed to acquire read lock"))?;
        Ok(meta.get(id).cloned())
    }

    /// Delete a blob
    pub fn delete(&self, id: &str) -> Result<bool> {
        let mut blobs = self
            .blobs
            .write()
            .map_err(|_| KernelError::internal("Failed to acquire write lock"))?;
        let mut meta = self
            .metadata
            .write()
            .map_err(|_| KernelError::internal("Failed to acquire write lock"))?;

        let removed = blobs.remove(id).is_some();
        meta.remove(id);
        Ok(removed)
    }

    /// List all blob IDs
    pub fn list(&self) -> Result<Vec<BlobId>> {
        let meta = self
            .metadata
            .read()
            .map_err(|_| KernelError::internal("Failed to acquire read lock"))?;
        Ok(meta.keys().cloned().collect())
    }

    /// Get total storage size
    pub fn total_size(&self) -> Result<u64> {
        let meta = self
            .metadata
            .read()
            .map_err(|_| KernelError::internal("Failed to acquire read lock"))?;
        Ok(meta.values().map(|m| m.size).sum())
    }

    /// Clear all blobs
    pub fn clear(&self) -> Result<()> {
        let mut blobs = self
            .blobs
            .write()
            .map_err(|_| KernelError::internal("Failed to acquire write lock"))?;
        let mut meta = self
            .metadata
            .write()
            .map_err(|_| KernelError::internal("Failed to acquire write lock"))?;
        blobs.clear();
        meta.clear();
        Ok(())
    }
}

impl Default for BlobStore {
    fn default() -> Self {
        Self::new()
    }
}
