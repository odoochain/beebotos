//! Chrome DevTools Protocol

use crate::error::Result;

/// CDP client
pub struct CdpClient {
    ws_url: String,
}

impl CdpClient {
    pub fn new(ws_url: impl Into<String>) -> Self {
        Self {
            ws_url: ws_url.into(),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        tracing::info!("Connecting to CDP at {}", self.ws_url);
        // TODO: Implement CDP connection
        Ok(())
    }

    pub async fn send_command(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        tracing::info!("CDP command: {} with params {:?}", method, params);
        // TODO: Implement actual CDP commands
        Ok(serde_json::Value::Null)
    }
}
