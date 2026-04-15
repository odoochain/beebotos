//! Agent execution context

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::AgentId;

/// Agent execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub agent_id: AgentId,
    pub task_id: String,
    pub inputs: HashMap<String, serde_json::Value>,
    pub outputs: HashMap<String, serde_json::Value>,
    pub metadata: ExecutionMetadata,
}

impl ExecutionContext {
    /// Create new context
    pub fn new(agent_id: AgentId, task_id: impl Into<String>) -> Self {
        Self {
            agent_id,
            task_id: task_id.into(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            metadata: ExecutionMetadata::default(),
        }
    }

    /// Set input
    pub fn set_input(
        &mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<(), serde_json::Error> {
        self.inputs.insert(key.into(), serde_json::to_value(value)?);
        Ok(())
    }

    /// Get input
    pub fn get_input<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.inputs
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set output
    pub fn set_output(
        &mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<(), serde_json::Error> {
        self.outputs
            .insert(key.into(), serde_json::to_value(value)?);
        Ok(())
    }

    /// Get output
    pub fn get_output<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.outputs
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Execution metadata for runtime context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub started_at: u64,
    pub completed_at: Option<u64>,
    pub memory_used: usize,
    pub cpu_time_ms: u64,
}

/// Shared context between agents
#[derive(Debug, Clone)]
pub struct SharedContext {
    data: std::sync::Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
}

impl SharedContext {
    /// Create new shared context
    pub fn new() -> Self {
        Self {
            data: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Get value
    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.data.read().await.get(key).cloned()
    }

    /// Set value
    pub async fn set(
        &self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> Result<(), serde_json::Error> {
        let value = serde_json::to_value(value)?;
        self.data.write().await.insert(key.into(), value);
        Ok(())
    }

    /// Remove value
    pub async fn remove(&self, key: &str) -> Option<serde_json::Value> {
        self.data.write().await.remove(key)
    }
}

impl Default for SharedContext {
    fn default() -> Self {
        Self::new()
    }
}
