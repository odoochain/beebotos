//! Indexing for storage queries

// use serde::{Deserialize, Serialize}; // Currently unused
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::error::{KernelError, Result};

/// Index type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexType {
    /// Unique index
    Unique,
    /// Non-unique index
    NonUnique,
    /// Full-text index
    FullText,
}

/// Index entry
#[derive(Debug, Clone)]
pub struct Index {
    /// Index type
    index_type: IndexType,
    /// For unique/non-unique: value -> ids
    value_index: HashMap<String, HashSet<String>>,
    /// For full-text: token -> ids
    token_index: HashMap<String, HashSet<String>>,
}

impl Index {
    /// Create new index
    pub fn new(index_type: IndexType) -> Self {
        Self {
            index_type,
            value_index: HashMap::new(),
            token_index: HashMap::new(),
        }
    }

    /// Insert value into index
    pub fn insert(&mut self, id: impl Into<String>, value: impl AsRef<str>) -> Result<()> {
        let id = id.into();
        let value = value.as_ref();

        match self.index_type {
            IndexType::Unique => {
                if self.value_index.contains_key(value) {
                    return Err(KernelError::invalid_argument("Unique constraint violation"));
                }
                let mut set = HashSet::new();
                set.insert(id);
                self.value_index.insert(value.to_string(), set);
            }
            IndexType::NonUnique => {
                self.value_index
                    .entry(value.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(id);
            }
            IndexType::FullText => {
                for token in value.split_whitespace() {
                    self.token_index
                        .entry(token.to_lowercase())
                        .or_insert_with(HashSet::new)
                        .insert(id.clone());
                }
            }
        }
        Ok(())
    }

    /// Remove value from index
    pub fn remove(&mut self, id: impl AsRef<str>, value: impl AsRef<str>) {
        let id = id.as_ref();
        let value = value.as_ref();

        match self.index_type {
            IndexType::Unique | IndexType::NonUnique => {
                if let Some(set) = self.value_index.get_mut(value) {
                    set.remove(id);
                    if set.is_empty() {
                        self.value_index.remove(value);
                    }
                }
            }
            IndexType::FullText => {
                for token in value.split_whitespace() {
                    if let Some(set) = self.token_index.get_mut(token) {
                        set.remove(id);
                        if set.is_empty() {
                            self.token_index.remove(token);
                        }
                    }
                }
            }
        }
    }

    /// Lookup by exact value
    pub fn lookup(&self, value: impl AsRef<str>) -> HashSet<String> {
        let value = value.as_ref();
        match self.index_type {
            IndexType::Unique | IndexType::NonUnique => {
                self.value_index.get(value).cloned().unwrap_or_default()
            }
            IndexType::FullText => self.token_index.get(value).cloned().unwrap_or_default(),
        }
    }

    /// Search for values containing prefix
    pub fn prefix_search(&self, prefix: impl AsRef<str>) -> HashSet<String> {
        let prefix = prefix.as_ref();
        let mut result = HashSet::new();

        for (value, ids) in &self.value_index {
            if value.starts_with(prefix) {
                result.extend(ids.iter().cloned());
            }
        }
        result
    }
}

/// Index manager for multiple indices
#[derive(Debug, Clone, Default)]
pub struct IndexManager {
    indices: Arc<RwLock<HashMap<String, Index>>>,
}

impl IndexManager {
    /// Create new index manager
    pub fn new() -> Self {
        Self {
            indices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create new index
    pub fn create_index(&self, name: impl Into<String>, index_type: IndexType) -> Result<()> {
        let mut indices = self
            .indices
            .write()
            .map_err(|_| KernelError::internal("Failed to acquire write lock"))?;
        indices.insert(name.into(), Index::new(index_type));
        Ok(())
    }

    /// Get index
    pub fn get_index(&self, name: &str) -> Result<Option<Index>> {
        let indices = self
            .indices
            .read()
            .map_err(|_| KernelError::internal("Failed to acquire read lock"))?;
        Ok(indices.get(name).cloned())
    }

    /// Insert into index
    pub fn insert(
        &self,
        index_name: &str,
        id: impl Into<String>,
        value: impl AsRef<str>,
    ) -> Result<()> {
        let mut indices = self
            .indices
            .write()
            .map_err(|_| KernelError::internal("Failed to acquire write lock"))?;
        match indices.get_mut(index_name) {
            Some(index) => index.insert(id, value),
            None => Err(KernelError::invalid_argument(format!(
                "Index not found: {}",
                index_name
            ))),
        }
    }

    /// Lookup in index
    pub fn lookup(&self, index_name: &str, value: impl AsRef<str>) -> Result<HashSet<String>> {
        let indices = self
            .indices
            .read()
            .map_err(|_| KernelError::internal("Failed to acquire read lock"))?;
        match indices.get(index_name) {
            Some(index) => Ok(index.lookup(value)),
            None => Err(KernelError::invalid_argument(format!(
                "Index not found: {}",
                index_name
            ))),
        }
    }

    /// Drop index
    pub fn drop_index(&self, name: &str) -> Result<bool> {
        let mut indices = self
            .indices
            .write()
            .map_err(|_| KernelError::internal("Failed to acquire write lock"))?;
        Ok(indices.remove(name).is_some())
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new(IndexType::NonUnique)
    }
}

impl Default for IndexType {
    fn default() -> Self {
        IndexType::NonUnique
    }
}
