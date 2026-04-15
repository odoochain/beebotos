//! SDK Client

use crate::error::Result;
use crate::SdkConfig;

/// BeeBotOS SDK client
#[allow(dead_code)]
pub struct BeeBotOSClient {
    config: SdkConfig,
}

impl BeeBotOSClient {
    pub async fn new(config: SdkConfig) -> Result<Self> {
        tracing::info!("Creating SDK client for {}", config.gateway_url);
        Ok(Self { config })
    }

    pub async fn health(&self) -> Result<bool> {
        Ok(true)
    }
}
