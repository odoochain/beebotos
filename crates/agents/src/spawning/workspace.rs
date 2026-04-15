//! Spawn Workspace

use std::collections::HashMap;
use std::path::PathBuf;

use uuid::Uuid;

use crate::AgentError;

/// Workspace for spawned tasks
#[derive(Debug)]
pub struct SpawnWorkspace {
    pub id: Uuid,
    pub path: PathBuf,
    pub env_vars: HashMap<String, String>,
    pub isolated: bool,
}

impl SpawnWorkspace {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        let id = Uuid::new_v4();
        let mut path: PathBuf = base_path.into();
        path.push(id.to_string());

        Self {
            id,
            path,
            env_vars: HashMap::new(),
            isolated: true,
        }
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    pub fn setup(&self) -> Result<(), AgentError> {
        std::fs::create_dir_all(&self.path)
            .map_err(|e| AgentError::platform(format!("Workspace setup failed: {}", e)))?;
        Ok(())
    }

    pub fn cleanup(&self) -> Result<(), AgentError> {
        if self.path.exists() {
            std::fs::remove_dir_all(&self.path)
                .map_err(|e| AgentError::platform(format!("Workspace setup failed: {}", e)))?;
        }
        Ok(())
    }
}
