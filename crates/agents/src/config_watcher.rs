//! Configuration Watcher
//!
//! 🟡 MEDIUM FIX: Hot reload configuration from file changes

use std::path::Path;
use std::sync::Arc;

use ::tracing::{info, warn};
use anyhow::anyhow;
use tokio::sync::RwLock;

use crate::AgentConfig;

/// Configuration manager with hot reload
pub struct ConfigManager {
    config: Arc<RwLock<AgentConfig>>,
    path: Option<std::path::PathBuf>,
}

impl ConfigManager {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            path: None,
        }
    }

    pub fn with_path(mut self, path: impl AsRef<Path>) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Get current config
    pub async fn get(&self) -> AgentConfig {
        self.config.read().await.clone()
    }

    /// Update config
    pub async fn update(&self, config: AgentConfig) {
        *self.config.write().await = config;
    }

    /// 🟡 MEDIUM FIX: Watch for file changes and reload
    pub async fn watch_and_reload(&self) -> anyhow::Result<()> {
        let path = match &self.path {
            Some(p) => p,
            None => {
                warn!("No config path set, skipping watch");
                return Ok(());
            }
        };

        info!("Starting config watcher for: {}", path.display());

        // Simple polling-based watcher (production should use notify crate)
        let mut last_modified = std::fs::metadata(path)?.modified()?;
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;

            match std::fs::metadata(path) {
                Ok(metadata) => {
                    if let Ok(modified) = metadata.modified() {
                        if modified > last_modified {
                            info!("Config file changed, reloading...");

                            match self.reload().await {
                                Ok(_) => {
                                    info!("Config reloaded successfully");
                                    last_modified = modified;
                                }
                                Err(e) => {
                                    warn!("Failed to reload config: {}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read config file: {}", e);
                }
            }
        }
    }

    /// Reload config from file
    async fn reload(&self) -> anyhow::Result<()> {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| anyhow!("No config path"))?;

        let content = tokio::fs::read_to_string(path).await?;
        let config: AgentConfig = serde_yaml::from_str(&content)?;

        self.update(config).await;
        Ok(())
    }
}
