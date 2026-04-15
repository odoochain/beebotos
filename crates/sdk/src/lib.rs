//! BeeBotOS SDK
//!
//! Software Development Kit for building agents and applications
//! on the BeeBotOS platform.

pub mod client;
pub mod context;
pub mod error;
pub mod types;

pub use client::BeeBotOSClient;
pub use context::AgentContext;
pub use error::{Result, SdkError};
pub use types::{AgentId, SessionId, TaskId};

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// SDK configuration
#[derive(Debug, Clone)]
pub struct SdkConfig {
    /// Gateway endpoint URL
    pub gateway_url: String,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Enable automatic retries
    pub enable_retries: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            gateway_url: "http://localhost:8080".to_string(),
            api_key: None,
            timeout_secs: 30,
            enable_retries: true,
            max_retries: 3,
        }
    }
}

impl SdkConfig {
    /// Create a new SDK configuration with the given gateway URL
    pub fn new(gateway_url: impl Into<String>) -> Self {
        Self {
            gateway_url: gateway_url.into(),
            ..Default::default()
        }
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

/// Initialize the SDK with the given configuration
pub async fn init(config: SdkConfig) -> Result<BeeBotOSClient> {
    BeeBotOSClient::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_config_default() {
        let config = SdkConfig::default();
        assert_eq!(config.gateway_url, "http://localhost:8080");
        assert_eq!(config.timeout_secs, 30);
        assert!(config.enable_retries);
    }

    #[test]
    fn test_sdk_config_builder() {
        let config = SdkConfig::new("https://api.beebotos.dev")
            .with_api_key("test-key")
            .with_timeout(60);

        assert_eq!(config.gateway_url, "https://api.beebotos.dev");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
