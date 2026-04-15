//! Webhook Handler
//!
//! Handles incoming webhooks to trigger agent actions.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
    pub signature: Option<String>,
}

/// Webhook response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub success: bool,
    pub message: String,
    pub run_id: Option<String>,
}

/// HTTP headers type
pub type HeaderMap = HashMap<String, String>;

/// Webhook handler trait
#[async_trait::async_trait]
pub trait WebhookHandler: Send + Sync {
    async fn handle(
        &self,
        payload: WebhookPayload,
        headers: HeaderMap,
    ) -> Result<String, WebhookError>;
}

/// Webhook manager
pub struct WebhookManager {
    handlers: HashMap<String, Box<dyn WebhookHandler>>,
    #[allow(dead_code)]
    secret: String,
}

impl WebhookManager {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            handlers: HashMap::new(),
            secret: secret.into(),
        }
    }

    pub fn register(&mut self, path: impl Into<String>, handler: Box<dyn WebhookHandler>) {
        self.handlers.insert(path.into(), handler);
    }

    pub async fn handle(
        &self,
        path: &str,
        payload: WebhookPayload,
        headers: HeaderMap,
    ) -> Result<String, WebhookError> {
        // Verify signature if present
        if let Some(sig) = &payload.signature {
            if !self.verify_signature(sig, &payload) {
                return Err(WebhookError::InvalidSignature);
            }
        }

        let handler = self
            .handlers
            .get(path)
            .ok_or(WebhookError::HandlerNotFound)?;

        handler.handle(payload, headers).await
    }

    fn verify_signature(&self, signature: &str, _payload: &WebhookPayload) -> bool {
        // Simplified - real impl would use HMAC
        !signature.is_empty()
    }
}

/// Webhook errors
#[derive(Debug, Clone)]
pub enum WebhookError {
    HandlerNotFound,
    InvalidSignature,
    HandlerError(String),
}

impl std::fmt::Display for WebhookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookError::HandlerNotFound => write!(f, "Handler not found"),
            WebhookError::InvalidSignature => write!(f, "Invalid signature"),
            WebhookError::HandlerError(s) => write!(f, "Handler error: {}", s),
        }
    }
}

impl std::error::Error for WebhookError {}
