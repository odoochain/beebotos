//! Storage Module
//!
//! Local storage for agents.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::{Result, SdkError};

/// Storage configuration
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub path: PathBuf,
    pub max_size: usize,
    pub encryption: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./data"),
            max_size: 100 * 1024 * 1024, // 100MB
            encryption: false,
        }
    }
}

/// Agent storage
pub struct Storage {
    path: PathBuf,
}

impl Storage {
    /// Create/open storage
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        tokio::fs::create_dir_all(&path).await?;
        Ok(Self { path })
    }

    /// Store value
    pub async fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let path = self.path.join(sanitize_key(key));
        
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(&path, value).await?;
        Ok(())
    }

    /// Retrieve value
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.path.join(sanitize_key(key));
        
        match tokio::fs::read(&path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete value
    pub async fn delete(&self, key: &str) -> Result<()> {
        let path = self.path.join(sanitize_key(key));
        
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.path.join(sanitize_key(key));
        Ok(path.exists())
    }

    /// List keys with prefix
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let mut keys = vec![];
        let mut entries = tokio::fs::read_dir(&self.path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().into_string().unwrap_or_default();
            
            if let Some(prefix) = prefix {
                if name.starts_with(prefix) {
                    keys.push(name);
                }
            } else {
                keys.push(name);
            }
        }
        
        Ok(keys)
    }

    /// Store JSON value
    pub async fn put_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let data = serde_json::to_vec(value)?;
        self.put(key, &data).await
    }

    /// Retrieve JSON value
    pub async fn get_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        match self.get(key).await? {
            Some(data) => {
                let value = serde_json::from_slice(&data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Get storage path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Sanitize key for filesystem
fn sanitize_key(key: &str) -> String {
    key.replace('/', "_")
        .replace('\\', "_")
        .replace('..', "_")
}

/// In-memory storage for testing
#[derive(Debug, Default)]
pub struct MemoryStorage {
    data: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self.data.lock().unwrap();
        Ok(data.get(key).cloned())
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.remove(key);
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.clear();
        Ok(())
    }
}
