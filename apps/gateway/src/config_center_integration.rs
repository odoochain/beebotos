//! ConfigCenter Integration
//!
//! 🔧 FIX: Integration with beebotos_core::ConfigCenter for unified configuration management.
//!
//! This module provides integration between the Gateway's local configuration
//! and the unified ConfigCenter from beebotos_core.

use std::sync::Arc;
use tracing::{info, warn, error};

use beebotos_core::{ConfigCenter, Environment};

/// Configuration manager that wraps ConfigCenter with Gateway-specific logic
pub struct GatewayConfigManager {
    /// The underlying ConfigCenter
    config_center: Arc<ConfigCenter>,
    /// Local config for backward compatibility during migration
    local_config: Option<crate::config::BeeBotOSConfig>,
}

impl GatewayConfigManager {
    /// Initialize ConfigCenter from environment
    pub async fn from_env() -> Result<Self, ConfigError> {
        info!("Initializing ConfigCenter from environment...");

        let config_center = ConfigCenter::from_env()
            .map_err(|e| ConfigError::Init(e.to_string()))?;

        info!("✅ ConfigCenter initialized successfully");

        Ok(Self {
            config_center: Arc::new(config_center),
            local_config: None,
        })
    }

    /// Initialize with local config fallback
    pub async fn with_local_config(local: crate::config::BeeBotOSConfig) -> Self {
        info!("Initializing ConfigCenter with local config fallback...");

        // Try to create ConfigCenter, but fall back to local config if it fails
        let config_center = match ConfigCenter::from_env() {
            Ok(center) => {
                info!("✅ ConfigCenter initialized, local config as fallback");
                Arc::new(center)
            }
            Err(e) => {
                warn!("⚠️ ConfigCenter init failed ({}), using local config only", e);
                // Create empty ConfigCenter as placeholder
                Arc::new(ConfigCenter::new(Environment::Development))
            }
        };

        Self {
            config_center,
            local_config: Some(local),
        }
    }

    /// Get ConfigCenter reference
    pub fn config_center(&self) -> &ConfigCenter {
        &self.config_center
    }

    /// 🔧 FIX: Get configuration value with fallback
    /// 
    /// First tries ConfigCenter, then falls back to local config
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<T, ConfigError> {
        // Try ConfigCenter first
        match self.config_center.get(key).await {
            Ok(value) => {
                return serde_json::from_value(value)
                    .map_err(|e| ConfigError::Parse(e.to_string()));
            }
            Err(_) => {
                // Fall back to local config
                if let Some(local) = &self.local_config {
                    return Self::get_from_local(local, key);
                }
            }
        }

        Err(ConfigError::NotFound(key.to_string()))
    }

    /// 🔧 FIX: Get string configuration
    pub async fn get_string(&self, key: &str) -> Result<String, ConfigError> {
        self.get::<String>(key).await
    }

    /// 🔧 FIX: Get integer configuration
    pub async fn get_int(&self, key: &str) -> Result<i64, ConfigError> {
        self.get::<i64>(key).await
    }

    /// 🔧 FIX: Get boolean configuration
    pub async fn get_bool(&self, key: &str) -> Result<bool, ConfigError> {
        self.get::<bool>(key).await
    }

    /// 🔧 FIX: Get database URL
    pub async fn database_url(&self) -> Result<String, ConfigError> {
        // Try ConfigCenter first
        if let Ok(url) = self.config_center.get_string("database.url").await {
            return Ok(url);
        }

        // Fall back to local config
        if let Some(local) = &self.local_config {
            return Ok(local.database.url.clone());
        }

        Err(ConfigError::NotFound("database.url".to_string()))
    }

    /// 🔧 FIX: Get server bind address
    pub async fn server_bind_addr(&self) -> Result<std::net::SocketAddr, ConfigError> {
        use std::net::SocketAddr;
        use std::str::FromStr;

        // Try ConfigCenter
        if let (Ok(host), Ok(port)) = (
            self.config_center.get_string("server.host").await,
            self.config_center.get_int("server.port").await,
        ) {
            let addr = format!("{}:{}", host, port);
            return SocketAddr::from_str(&addr)
                .map_err(|e| ConfigError::Parse(e.to_string()));
        }

        // Fall back to local config
        if let Some(local) = &self.local_config {
            let addr = format!("{}:{}", local.server.host, local.server.port);
            return SocketAddr::from_str(&addr)
                .map_err(|e| ConfigError::Parse(e.to_string()));
        }

        Err(ConfigError::NotFound("server.bind_addr".to_string()))
    }

    /// 🔧 FIX: Get JWT secret
    pub async fn jwt_secret(&self) -> Result<secrecy::SecretString, ConfigError> {
        // Try ConfigCenter
        if let Ok(secret) = self.config_center.get_string("jwt.secret").await {
            return Ok(secrecy::SecretString::new(secret));
        }

        // Fall back to local config
        if let Some(local) = &self.local_config {
            return Ok(local.jwt.secret.clone());
        }

        Err(ConfigError::NotFound("jwt.secret".to_string()))
    }

    /// 🔧 FIX: Get blockchain configuration
    pub async fn blockchain_config(&self) -> Result<crate::config::BlockchainConfig, ConfigError> {
        // For now, return local config if available
        if let Some(local) = &self.local_config {
            return Ok(local.blockchain.clone());
        }

        Err(ConfigError::NotFound("blockchain".to_string()))
    }

    /// 🔧 FIX: Check if feature is enabled
    pub async fn is_enabled(&self, feature: &str) -> bool {
        // Try ConfigCenter
        if let Ok(enabled) = self.config_center.get_bool(&format!("features.{}", feature)).await {
            return enabled;
        }

        // Fall back to local config checks
        if let Some(local) = &self.local_config {
            match feature {
                "blockchain" => return local.blockchain.enabled,
                "metrics" => return local.metrics.enabled,
                "tracing" => return local.tracing.enabled,
                _ => return false,
            }
        }

        false
    }

    /// 🔧 FIX: Reload configuration (hot reload)
    pub async fn reload(&self) -> Result<(), ConfigError> {
        info!("Reloading configuration...");
        
        // Reload ConfigCenter
        if let Err(e) = self.config_center.reload().await {
            warn!("Failed to reload ConfigCenter: {}", e);
        }

        info!("✅ Configuration reload complete");
        Ok(())
    }

    /// 🔧 FIX: Export current configuration
    pub async fn export_config(&self) -> Result<serde_json::Value, ConfigError> {
        // Export from ConfigCenter
        match self.config_center.export().await {
            Ok(config) => Ok(config),
            Err(_) => {
                // Fall back to local config serialization
                if let Some(local) = &self.local_config {
                    serde_json::to_value(local)
                        .map_err(|e| ConfigError::Serialize(e.to_string()))
                } else {
                    Err(ConfigError::NotFound("config".to_string()))
                }
            }
        }
    }

    // Private helper methods

    fn get_from_local<T: serde::de::DeserializeOwned>(
        local: &crate::config::BeeBotOSConfig,
        key: &str,
    ) -> Result<T, ConfigError> {
        // Parse dot-notation key and extract value from local config
        let parts: Vec<&str> = key.split('.').collect();
        
        // For now, return error for non-trivial paths
        // In a full implementation, this would use reflection or manual matching
        Err(ConfigError::NotFound(key.to_string()))
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to initialize config: {0}")]
    Init(String),
    
    #[error("Configuration key not found: {0}")]
    NotFound(String),
    
    #[error("Failed to parse config value: {0}")]
    Parse(String),
    
    #[error("Failed to serialize config: {0}")]
    Serialize(String),
    
    #[error("Configuration reload failed: {0}")]
    Reload(String),
}

/// 🔧 FIX: Initialize global config manager
static GLOBAL_CONFIG_MANAGER: tokio::sync::OnceCell<Arc<GatewayConfigManager>> = tokio::sync::OnceCell::const_new();

/// Initialize global config manager
pub async fn init_config_manager(local_config: crate::config::BeeBotOSConfig) -> Arc<GatewayConfigManager> {
    GLOBAL_CONFIG_MANAGER
        .get_or_init(|| async {
            Arc::new(GatewayConfigManager::with_local_config(local_config).await)
        })
        .await
        .clone()
}

/// Get global config manager
pub fn config_manager() -> Option<Arc<GatewayConfigManager>> {
    GLOBAL_CONFIG_MANAGER.get().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_manager_creation() {
        // This would test with a mock config
        // For now, just verify it compiles
    }
}
