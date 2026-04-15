//! WhatsApp Webhook Handler
//!
//! Handles incoming webhooks from WhatsApp via Baileys Node.js bridge.
//! Supports message events, connection status, and QR code authentication.

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::communication::webhook::{
    SignatureVerification, WebhookConfig, WebhookEvent, WebhookEventType, WebhookHandler,
};
use crate::communication::{Message, MessageType, PlatformType};
use crate::error::{AgentError, Result};

/// WhatsApp webhook payload from Baileys bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppWebhookPayload {
    /// Event type
    #[serde(rename = "type")]
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
    /// Timestamp
    pub timestamp: Option<i64>,
    /// Connection ID
    pub connection_id: Option<String>,
}

/// WhatsApp message data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppMessageData {
    /// Message ID
    pub id: String,
    /// Remote JID (phone number)
    pub remote_jid: String,
    /// From Me flag
    pub from_me: bool,
    /// Message content
    pub content: String,
    /// Message type
    pub message_type: String,
    /// Timestamp
    pub timestamp: i64,
    /// Participant (for group messages)
    pub participant: Option<String>,
    /// Media URL (if media message)
    pub media_url: Option<String>,
    /// Media type
    pub media_type: Option<String>,
    /// Caption (for media messages)
    pub caption: Option<String>,
    /// Quoted message ID (for replies)
    pub quoted_message_id: Option<String>,
}

/// WhatsApp connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConnectionStatus {
    /// Connection state
    pub state: String,
    /// QR code (for pairing)
    pub qr_code: Option<String>,
    /// Error message
    pub error: Option<String>,
}

/// WhatsApp webhook handler
pub struct WhatsAppWebhookHandler {
    config: WebhookConfig,
    /// Secret for webhook verification (optional)
    secret: Option<String>,
}

impl WhatsAppWebhookHandler {
    /// Create a new WhatsApp webhook handler
    pub fn new(config: WebhookConfig) -> Self {
        let secret = config.secret.clone();
        Self { config, secret }
    }

    /// Parse message event
    fn parse_message(&self, data: &serde_json::Value) -> Result<WebhookEvent> {
        let msg: WhatsAppMessageData = serde_json::from_value(data.clone()).map_err(|e| {
            AgentError::platform(format!("Failed to parse WhatsApp message: {}", e))
        })?;

        let message_type = match msg.message_type.as_str() {
            "text" => MessageType::Text,
            "image" => MessageType::Image,
            "video" => MessageType::Video,
            "audio" => MessageType::Voice,
            "document" => MessageType::File,
            "location" => MessageType::Text,
            "sticker" => MessageType::Image,
            _ => MessageType::System,
        };

        let mut metadata = HashMap::new();
        metadata.insert("remote_jid".to_string(), msg.remote_jid.clone());
        metadata.insert("from_me".to_string(), msg.from_me.to_string());
        if let Some(participant) = &msg.participant {
            metadata.insert("participant".to_string(), participant.clone());
        }
        if let Some(media_url) = &msg.media_url {
            metadata.insert("media_url".to_string(), media_url.clone());
        }
        if let Some(media_type) = &msg.media_type {
            metadata.insert("media_type".to_string(), media_type.clone());
        }
        if let Some(quoted_id) = &msg.quoted_message_id {
            metadata.insert("quoted_message_id".to_string(), quoted_id.clone());
        }

        let message = Message {
            id: uuid::Uuid::parse_str(&msg.id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
            thread_id: uuid::Uuid::new_v4(),
            platform: PlatformType::WhatsApp,
            message_type,
            content: msg.content.clone(),
            metadata,
            timestamp: chrono::DateTime::from_timestamp(msg.timestamp, 0)
                .unwrap_or_else(chrono::Utc::now),
        };

        Ok(WebhookEvent {
            event_type: WebhookEventType::MessageReceived,
            platform: PlatformType::WhatsApp,
            event_id: msg.id,
            timestamp: chrono::Utc::now(),
            payload: data.clone(),
            message: Some(message),
            metadata: HashMap::new(),
        })
    }

    /// Parse connection status event
    fn parse_connection_status(&self, data: &serde_json::Value) -> Result<WebhookEvent> {
        let status: WhatsAppConnectionStatus =
            serde_json::from_value(data.clone()).map_err(|e| {
                AgentError::platform(format!("Failed to parse connection status: {}", e))
            })?;

        let event_type = match status.state.as_str() {
            "connected" => WebhookEventType::UserJoined,
            "disconnected" => WebhookEventType::UserLeft,
            "qr" => WebhookEventType::System,
            _ => WebhookEventType::System,
        };

        let mut metadata = HashMap::new();
        metadata.insert("connection_state".to_string(), status.state);
        if let Some(qr) = status.qr_code {
            metadata.insert("qr_code".to_string(), qr);
        }
        if let Some(error) = status.error {
            metadata.insert("error".to_string(), error);
        }

        Ok(WebhookEvent {
            event_type,
            platform: PlatformType::WhatsApp,
            event_id: format!("conn_{}", chrono::Utc::now().timestamp()),
            timestamp: chrono::Utc::now(),
            payload: data.clone(),
            message: None,
            metadata,
        })
    }
}

#[async_trait]
impl WebhookHandler for WhatsAppWebhookHandler {
    fn platform_type(&self) -> PlatformType {
        PlatformType::WhatsApp
    }

    async fn verify_signature(
        &self,
        _body: &[u8],
        _signature: Option<&str>,
        _timestamp: Option<&str>,
    ) -> Result<SignatureVerification> {
        // WhatsApp Baileys bridge doesn't use signature verification by default
        // If secret is configured, implement HMAC verification here
        if self.secret.is_some() {
            // TODO: Implement HMAC verification if needed
            warn!("WhatsApp webhook signature verification not implemented");
        }
        Ok(SignatureVerification::Skipped)
    }

    async fn parse_payload(&self, body: &[u8]) -> Result<Vec<WebhookEvent>> {
        let payload: WhatsAppWebhookPayload = serde_json::from_slice(body).map_err(|e| {
            AgentError::platform(format!("Failed to parse WhatsApp webhook: {}", e))
        })?;

        debug!("Received WhatsApp webhook: {:?}", payload.event_type);

        let event = match payload.event_type.as_str() {
            "message" => self.parse_message(&payload.data)?,
            "message_update" => {
                let mut event = self.parse_message(&payload.data)?;
                event.event_type = WebhookEventType::MessageEdited;
                event
            }
            "message_delete" => {
                let mut event = self.parse_message(&payload.data)?;
                event.event_type = WebhookEventType::MessageDeleted;
                event
            }
            "connection" => self.parse_connection_status(&payload.data)?,
            _ => {
                warn!("Unknown WhatsApp event type: {}", payload.event_type);
                WebhookEvent {
                    event_type: WebhookEventType::Unknown,
                    platform: PlatformType::WhatsApp,
                    event_id: format!("unknown_{}", chrono::Utc::now().timestamp()),
                    timestamp: chrono::Utc::now(),
                    payload: serde_json::to_value(&payload).map_err(|e| {
                        AgentError::platform(format!("JSON serialization error: {}", e))
                    })?,
                    message: None,
                    metadata: HashMap::new(),
                }
            }
        };

        Ok(vec![event])
    }

    async fn handle_event(&self, event: WebhookEvent) -> Result<()> {
        info!(
            "Handling WhatsApp event: {:?} - {}",
            event.event_type, event.event_id
        );

        // Event handling is done by the processor
        // This method can be used for logging or additional processing

        Ok(())
    }

    fn get_config(&self) -> &WebhookConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_message() {
        let config = WebhookConfig {
            platform: PlatformType::WhatsApp,
            endpoint_path: "/webhook/whatsapp".to_string(),
            ..Default::default()
        };
        let handler = WhatsAppWebhookHandler::new(config);

        let data = serde_json::json!({
            "id": "msg123",
            "remote_jid": "1234567890@s.whatsapp.net",
            "from_me": false,
            "content": "Hello, World!",
            "message_type": "text",
            "timestamp": 1234567890,
        });

        let event = handler.parse_message(&data).unwrap();
        assert_eq!(event.event_type, WebhookEventType::MessageReceived);
        assert_eq!(event.platform, PlatformType::WhatsApp);
        assert!(event.message.is_some());

        let message = event.message.unwrap();
        assert_eq!(message.content, "Hello, World!");
        assert_eq!(message.message_type, MessageType::Text);
    }

    #[test]
    fn test_parse_connection_status() {
        let config = WebhookConfig {
            platform: PlatformType::WhatsApp,
            endpoint_path: "/webhook/whatsapp".to_string(),
            ..Default::default()
        };
        let handler = WhatsAppWebhookHandler::new(config);

        let data = serde_json::json!({
            "state": "connected",
            "qr_code": null,
            "error": null,
        });

        let event = handler.parse_connection_status(&data).unwrap();
        assert_eq!(event.event_type, WebhookEventType::UserJoined);
        assert_eq!(event.metadata.get("connection_state").unwrap(), "connected");
    }
}
