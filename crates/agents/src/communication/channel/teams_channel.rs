//! Microsoft Teams Channel Implementation
//!
//! Unified Channel trait implementation for Microsoft Teams.
//! Supports Bot Framework API (Webhook/Polling mode).

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio::time::Duration;
use tracing::{debug, error, info, warn};

use super::r#trait::{BaseChannelConfig, ConnectionMode, ContentType};
use super::{Channel, ChannelConfig, ChannelEvent, ChannelInfo, MemberInfo};
use crate::communication::{Message, MessageType, PlatformType};
use crate::error::{AgentError, Result};

/// Bot Framework API base URL
#[allow(dead_code)]
const BOT_FRAMEWORK_API_BASE: &str = "https://api.botframework.com";

/// Teams Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsChannelConfig {
    /// Microsoft App ID
    pub app_id: String,
    /// Microsoft App Password
    pub app_password: String,
    /// Webhook endpoint path (default: "/api/messages")
    #[serde(default = "default_webhook_path")]
    pub webhook_path: String,
    /// Tenant ID (optional, for single-tenant bots)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    /// Base channel configuration
    #[serde(flatten)]
    pub base: BaseChannelConfig,
}

fn default_webhook_path() -> String {
    "/api/messages".to_string()
}

impl Default for TeamsChannelConfig {
    fn default() -> Self {
        Self {
            app_id: String::new(),
            app_password: String::new(),
            webhook_path: default_webhook_path(),
            tenant_id: None,
            base: BaseChannelConfig {
                connection_mode: ConnectionMode::Webhook,
                webhook_port: 3978,
                ..Default::default()
            },
        }
    }
}

impl ChannelConfig for TeamsChannelConfig {
    fn from_env() -> Option<Self>
    where
        Self: Sized,
    {
        let app_id = std::env::var("TEAMS_APP_ID").ok()?;
        let app_password = std::env::var("TEAMS_APP_PASSWORD").ok()?;

        let webhook_path =
            std::env::var("TEAMS_WEBHOOK_PATH").unwrap_or_else(|_| default_webhook_path());

        let tenant_id = std::env::var("TEAMS_TENANT_ID").ok();

        let mut base = BaseChannelConfig::from_env("TEAMS").unwrap_or_default();
        // Teams default webhook port is 3978
        if std::env::var("TEAMS_WEBHOOK_PORT").is_err() {
            base.webhook_port = 3978;
        }
        // Teams default is Webhook
        if std::env::var("TEAMS_CONNECTION_MODE").is_err() {
            base.connection_mode = ConnectionMode::Webhook;
        }

        Some(Self {
            app_id,
            app_password,
            webhook_path,
            tenant_id,
            base,
        })
    }

    fn is_valid(&self) -> bool {
        !self.app_id.is_empty() && !self.app_password.is_empty()
    }

    fn allowlist(&self) -> Vec<String> {
        vec![]
    }

    fn connection_mode(&self) -> ConnectionMode {
        self.base.connection_mode
    }

    fn auto_reconnect(&self) -> bool {
        self.base.auto_reconnect
    }

    fn max_reconnect_attempts(&self) -> u32 {
        self.base.max_reconnect_attempts
    }
}

/// Bot Framework token response
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    #[serde(rename = "token_type")]
    pub token_type: String,
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "expires_in")]
    pub expires_in: i64,
}

/// Teams activity (incoming message)
#[derive(Debug, Clone, Deserialize)]
pub struct TeamsActivity {
    #[serde(rename = "type")]
    pub activity_type: String,
    pub id: Option<String>,
    pub timestamp: Option<String>,
    #[serde(rename = "localTimestamp")]
    pub local_timestamp: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: String,
    #[serde(rename = "from")]
    pub from_user: TeamsChannelAccount,
    pub conversation: TeamsConversation,
    #[serde(rename = "recipient")]
    pub recipient: TeamsChannelAccount,
    #[serde(rename = "text")]
    pub text: Option<String>,
    #[serde(rename = "textFormat")]
    pub text_format: Option<String>,
    pub attachments: Option<Vec<TeamsAttachment>>,
    pub entities: Option<Vec<serde_json::Value>>,
}

/// Teams channel account
#[derive(Debug, Clone, Deserialize)]
pub struct TeamsChannelAccount {
    pub id: String,
    pub name: String,
    #[serde(rename = "aadObjectId")]
    pub aad_object_id: Option<String>,
}

/// Teams conversation
#[derive(Debug, Clone, Deserialize)]
pub struct TeamsConversation {
    pub id: String,
    #[serde(rename = "conversationType")]
    pub conversation_type: Option<String>,
    #[serde(rename = "isGroup")]
    pub is_group: Option<bool>,
    pub name: Option<String>,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<String>,
}

/// Teams attachment
#[derive(Debug, Clone, Deserialize)]
pub struct TeamsAttachment {
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "contentUrl")]
    pub content_url: Option<String>,
    pub name: Option<String>,
    pub content: Option<serde_json::Value>,
}

/// Teams Channel implementation
pub struct TeamsChannel {
    config: TeamsChannelConfig,
    http_client: reqwest::Client,
    access_token: Arc<RwLock<Option<(String, chrono::DateTime<chrono::Utc>)>>>,
    connected: Arc<RwLock<bool>>,
    listener_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl TeamsChannel {
    /// Create a new Teams channel
    pub fn new(config: TeamsChannelConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
            access_token: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(false)),
            listener_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let config = TeamsChannelConfig::from_env().ok_or_else(|| {
            AgentError::configuration("TEAMS_APP_ID or TEAMS_APP_PASSWORD not set")
        })?;
        Ok(Self::new(config))
    }

    /// Get access token for Bot Framework
    async fn get_access_token(&self) -> Result<String> {
        // Check if we have a valid cached token
        {
            let cache = self.access_token.read().await;
            if let Some((token, expires_at)) = cache.as_ref() {
                if *expires_at > chrono::Utc::now() + chrono::Duration::minutes(5) {
                    debug!("Using cached Bot Framework access token");
                    return Ok(token.clone());
                }
            }
        }

        // Fetch new token using client credentials flow
        let url = "https://login.microsoftonline.com/botframework.com/oauth2/v2.0/token";

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.app_id),
            ("client_secret", &self.config.app_password),
            ("scope", "https://api.botframework.com/.default"),
        ];

        let response = self
            .http_client
            .post(url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AgentError::platform(format!("Failed to get access token: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AgentError::authentication(format!(
                "Failed to get access token: {}",
                error_text
            ))
            .into());
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AgentError::platform(format!("Failed to parse token response: {}", e)))?;

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in);

        // Cache the token
        let mut cache = self.access_token.write().await;
        *cache = Some((token_response.access_token.clone(), expires_at));

        info!("Bot Framework access token refreshed successfully");
        Ok(token_response.access_token)
    }

    /// Send activity to conversation
    pub async fn send_activity(
        &self,
        service_url: &str,
        conversation_id: &str,
        text: &str,
    ) -> Result<String> {
        let token = self.get_access_token().await?;
        let url = format!(
            "{}/v3/conversations/{}/activities",
            service_url, conversation_id
        );

        let body = serde_json::json!({
            "type": "message",
            "text": text,
            "textFormat": "markdown",
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AgentError::platform(format!("Failed to send activity: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(
                AgentError::platform(format!("Bot Framework API error: {}", error_text)).into(),
            );
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AgentError::platform(format!("Failed to parse response: {}", e)))?;

        let activity_id = result
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(activity_id)
    }

    /// Convert Teams activity to internal Message
    #[allow(dead_code)]
    fn convert_activity(&self, activity: &TeamsActivity) -> Option<Message> {
        let content = activity.text.clone().unwrap_or_default();

        let mut metadata = std::collections::HashMap::new();
        metadata.insert(
            "activity_id".to_string(),
            activity.id.clone().unwrap_or_default(),
        );
        metadata.insert("channel_id".to_string(), activity.channel_id.clone());
        metadata.insert(
            "conversation_id".to_string(),
            activity.conversation.id.clone(),
        );
        metadata.insert("user_id".to_string(), activity.from_user.id.clone());
        metadata.insert("user_name".to_string(), activity.from_user.name.clone());
        metadata.insert(
            "service_url".to_string(),
            "https://smba.trafficmanager.net/amer".to_string(),
        );

        if let Some(ref tenant_id) = activity.conversation.tenant_id {
            metadata.insert("tenant_id".to_string(), tenant_id.clone());
        }

        if let Some(ref conversation_type) = activity.conversation.conversation_type {
            metadata.insert("conversation_type".to_string(), conversation_type.clone());
        }

        let timestamp = activity
            .timestamp
            .as_ref()
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .map(|t| t.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        Some(Message {
            id: uuid::Uuid::new_v4(),
            thread_id: uuid::Uuid::new_v4(),
            platform: PlatformType::Teams,
            message_type: MessageType::Text,
            content,
            metadata,
            timestamp,
        })
    }

    /// Run webhook listener
    async fn run_webhook_listener(&self, _event_bus: mpsc::Sender<ChannelEvent>) -> Result<()> {
        info!(
            "Teams webhook listener started on port {}{}",
            self.config.base.webhook_port, self.config.webhook_path
        );
        // TODO: Implement HTTP server for Bot Framework webhook
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    /// Run polling listener (not commonly used for Teams)
    #[allow(dead_code)]
    async fn run_polling_listener(&self, event_bus: mpsc::Sender<ChannelEvent>) -> Result<()> {
        warn!("Teams does not support polling mode, using webhook instead");
        self.run_webhook_listener(event_bus).await
    }
}

#[async_trait]
impl Channel for TeamsChannel {
    fn name(&self) -> &str {
        "teams"
    }

    fn platform(&self) -> PlatformType {
        PlatformType::Teams
    }

    fn is_connected(&self) -> bool {
        if let Ok(connected) = self.connected.try_read() {
            *connected
        } else {
            false
        }
    }

    async fn connect(&mut self) -> Result<()> {
        // Verify credentials by getting access token
        self.get_access_token().await?;
        *self.connected.write().await = true;
        info!("Teams channel connected successfully");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.stop_listener().await?;
        *self.access_token.write().await = None;
        *self.connected.write().await = false;
        info!("Disconnected from Teams");
        Ok(())
    }

    async fn send(&self, channel_id: &str, message: &Message) -> Result<()> {
        // For Teams, channel_id should be in format "service_url|conversation_id"
        let parts: Vec<&str> = channel_id.split('|').collect();
        if parts.len() != 2 {
            return Err(AgentError::platform(
                "Invalid channel_id format, expected 'service_url|conversation_id'",
            ));
        }

        let service_url = parts[0];
        let conversation_id = parts[1];

        self.send_activity(service_url, conversation_id, &message.content)
            .await?;
        Ok(())
    }

    async fn start_listener(&self, event_bus: mpsc::Sender<ChannelEvent>) -> Result<()> {
        self.stop_listener().await?;

        match self.config.base.connection_mode {
            ConnectionMode::Webhook | ConnectionMode::Polling => {
                let channel = self.clone();
                let handle = tokio::spawn(async move {
                    if let Err(e) = channel.run_webhook_listener(event_bus).await {
                        error!("Webhook listener error: {}", e);
                    }
                });
                *self.listener_handle.write().await = Some(handle);
            }
            _ => {
                return Err(AgentError::platform(
                    "Teams does not support WebSocket mode",
                ));
            }
        }

        Ok(())
    }

    async fn stop_listener(&self) -> Result<()> {
        if let Some(handle) = self.listener_handle.write().await.take() {
            handle.abort();
        }
        Ok(())
    }

    fn supported_content_types(&self) -> Vec<ContentType> {
        vec![
            ContentType::Text,
            ContentType::Image,
            ContentType::File,
            ContentType::Audio,
            ContentType::Video,
            ContentType::Card,
            ContentType::Rich,
        ]
    }

    async fn list_channels(&self) -> Result<Vec<ChannelInfo>> {
        // Teams conversations are discovered through incoming activities
        Ok(vec![])
    }

    async fn list_members(&self, _channel_id: &str) -> Result<Vec<MemberInfo>> {
        // Bot Framework has API for this
        Ok(vec![])
    }

    fn connection_mode(&self) -> ConnectionMode {
        self.config.base.connection_mode
    }
}

impl Clone for TeamsChannel {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
            access_token: self.access_token.clone(),
            connected: self.connected.clone(),
            listener_handle: Arc::new(RwLock::new(None)),
        }
    }
}
