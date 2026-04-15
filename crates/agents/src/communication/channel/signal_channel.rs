//! Signal Channel Implementation
//!
//! Unified Channel trait implementation for Signal Messenger.
//! Uses signal-cli via JSON-RPC over Unix socket or DBus.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(not(unix))]
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixStream;
use tokio::sync::{mpsc, RwLock};
use tokio::time::Duration;
use tracing::{error, info};

use super::r#trait::{BaseChannelConfig, ConnectionMode, ContentType};
use super::{Channel, ChannelConfig, ChannelEvent, ChannelInfo, MemberInfo};
use crate::communication::{Message, MessageType, PlatformType};
use crate::error::{AgentError, Result};

/// Signal Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalChannelConfig {
    /// Phone number of the Signal account
    pub phone_number: String,
    /// Path to signal-cli executable
    #[serde(default = "default_signal_cli_path")]
    pub signal_cli_path: String,
    /// Path to signal-cli data directory
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    /// Unix socket path for JSON-RPC
    #[serde(default = "default_socket_path")]
    pub socket_path: String,
    #[serde(flatten)]
    pub base: BaseChannelConfig,
}

fn default_signal_cli_path() -> String {
    "signal-cli".to_string()
}

fn default_data_dir() -> String {
    "~/.local/share/signal-cli".to_string()
}

fn default_socket_path() -> String {
    "data/signal-cli.sock".to_string()
}

impl Default for SignalChannelConfig {
    fn default() -> Self {
        Self {
            phone_number: String::new(),
            signal_cli_path: default_signal_cli_path(),
            data_dir: default_data_dir(),
            socket_path: default_socket_path(),
            base: BaseChannelConfig::default(),
        }
    }
}

impl ChannelConfig for SignalChannelConfig {
    fn from_env() -> Option<Self>
    where
        Self: Sized,
    {
        let phone_number = std::env::var("SIGNAL_PHONE_NUMBER").ok()?;

        let signal_cli_path =
            std::env::var("SIGNAL_CLI_PATH").unwrap_or_else(|_| default_signal_cli_path());

        let data_dir = std::env::var("SIGNAL_DATA_DIR").unwrap_or_else(|_| default_data_dir());

        let socket_path =
            std::env::var("SIGNAL_SOCKET_PATH").unwrap_or_else(|_| default_socket_path());

        let mut base = BaseChannelConfig::from_env("SIGNAL")?;
        // Signal uses Polling for DBus mode
        if let Ok(mode) = std::env::var("SIGNAL_CONNECTION_MODE") {
            if mode == "dbus" {
                base.connection_mode = ConnectionMode::Polling;
            }
        }

        Some(Self {
            phone_number,
            signal_cli_path,
            data_dir,
            socket_path,
            base,
        })
    }

    fn is_valid(&self) -> bool {
        !self.phone_number.is_empty()
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

/// Signal JSON-RPC request
#[derive(Debug, Clone, Serialize)]
pub struct SignalRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

/// Signal JSON-RPC response
#[derive(Debug, Clone, Deserialize)]
pub struct SignalResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<SignalError>,
    pub id: u64,
}

/// Signal error
#[derive(Debug, Clone, Deserialize)]
pub struct SignalError {
    pub code: i32,
    pub message: String,
}

/// Signal message envelope
#[derive(Debug, Clone, Deserialize)]
pub struct SignalEnvelope {
    pub source: String,
    #[serde(rename = "sourceNumber")]
    pub source_number: Option<String>,
    #[serde(rename = "sourceName")]
    pub source_name: Option<String>,
    #[serde(rename = "sourceUuid")]
    pub source_uuid: Option<String>,
    #[serde(rename = "timestamp")]
    pub timestamp: i64,
    #[serde(rename = "dataMessage")]
    pub data_message: Option<SignalDataMessage>,
    #[serde(rename = "syncMessage")]
    pub sync_message: Option<serde_json::Value>,
    #[serde(rename = "callMessage")]
    pub call_message: Option<serde_json::Value>,
    #[serde(rename = "receiptMessage")]
    pub receipt_message: Option<serde_json::Value>,
    #[serde(rename = "typingMessage")]
    pub typing_message: Option<serde_json::Value>,
}

/// Signal data message
#[derive(Debug, Clone, Deserialize)]
pub struct SignalDataMessage {
    pub timestamp: i64,
    pub message: Option<String>,
    #[serde(rename = "groupInfo")]
    pub group_info: Option<SignalGroupInfo>,
    pub attachments: Option<Vec<SignalAttachment>>,
    #[serde(rename = "quote")]
    pub quote: Option<serde_json::Value>,
    #[serde(rename = "mentions")]
    pub mentions: Option<Vec<serde_json::Value>>,
}

/// Signal group info
#[derive(Debug, Clone, Deserialize)]
pub struct SignalGroupInfo {
    #[serde(rename = "groupId")]
    pub group_id: String,
    pub r#type: String,
}

/// Signal attachment
#[derive(Debug, Clone, Deserialize)]
pub struct SignalAttachment {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub filename: String,
    pub id: String,
    pub size: i64,
}

/// Signal Channel implementation
pub struct SignalChannel {
    config: SignalChannelConfig,
    connected: Arc<RwLock<bool>>,
    #[cfg(unix)]
    socket: Arc<RwLock<Option<UnixStream>>>,
    #[cfg(not(unix))]
    socket: Arc<RwLock<Option<TcpStream>>>,
    listener_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    request_id: Arc<RwLock<u64>>,
}

impl SignalChannel {
    /// Create a new Signal channel
    pub fn new(config: SignalChannelConfig) -> Self {
        Self {
            config,
            connected: Arc::new(RwLock::new(false)),
            socket: Arc::new(RwLock::new(None)),
            listener_handle: Arc::new(RwLock::new(None)),
            request_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Check if running on a supported platform
    #[allow(dead_code)]
    fn check_platform_support() -> Result<()> {
        #[cfg(not(unix))]
        {
            return Err(AgentError::platform(
                "Signal channel requires Unix platform. Windows is not supported.",
            )
            .into());
        }
        #[cfg(unix)]
        {
            Ok(())
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let config = SignalChannelConfig::from_env()
            .ok_or_else(|| AgentError::configuration("SIGNAL_PHONE_NUMBER not set"))?;
        Ok(Self::new(config))
    }

    /// Get next request ID
    #[allow(dead_code)]
    async fn next_request_id(&self) -> u64 {
        let mut id = self.request_id.write().await;
        *id += 1;
        *id
    }

    /// Connect to signal-cli via Unix socket
    #[cfg(unix)]
    async fn connect_socket(&self) -> Result<()> {
        let stream = UnixStream::connect(&self.config.socket_path)
            .await
            .map_err(|e| {
                AgentError::platform(format!("Failed to connect to signal-cli socket: {}", e))
            })?;

        *self.socket.write().await = Some(stream);
        *self.connected.write().await = true;

        info!("Connected to signal-cli via Unix socket");
        Ok(())
    }

    /// Connect to signal-cli (Windows - not supported)
    #[cfg(not(unix))]
    async fn connect_socket(&self) -> Result<()> {
        Err(
            AgentError::platform("Signal channel requires Unix platform. Use WSL2 or a Linux VM.")
                .into(),
        )
    }

    /// Send JSON-RPC request
    #[cfg(unix)]
    async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<SignalResponse> {
        let request = SignalRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: self.next_request_id().await,
        };

        let json = serde_json::to_string(&request)?;
        let mut socket = self.socket.write().await;

        if let Some(ref mut stream) = *socket {
            stream.write_all(json.as_bytes()).await?;
            stream.write_all(b"\n").await?;
            stream.flush().await?;

            // Read response
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await?;

            let response: SignalResponse = serde_json::from_str(&line)?;
            Ok(response)
        } else {
            Err(AgentError::platform("Not connected to signal-cli").into())
        }
    }

    /// Send JSON-RPC request (Windows - not supported)
    #[cfg(not(unix))]
    async fn send_request(
        &self,
        _method: &str,
        _params: serde_json::Value,
    ) -> Result<SignalResponse> {
        Err(
            AgentError::platform("Signal channel requires Unix platform. Use WSL2 or a Linux VM.")
                .into(),
        )
    }

    /// Send text message
    pub async fn send_text_message(&self, recipient: &str, text: &str) -> Result<String> {
        let params = serde_json::json!({
            "account": self.config.phone_number,
            "recipient": [recipient],
            "message": text,
        });

        let response = self.send_request("send", params).await?;

        if let Some(error) = response.error {
            return Err(AgentError::platform(format!(
                "signal-cli error: {} (code: {})",
                error.message, error.code
            ))
            .into());
        }

        Ok(response
            .result
            .and_then(|r| r.get("timestamp").and_then(|t| t.as_i64()))
            .map(|t| t.to_string())
            .unwrap_or_default())
    }

    /// Convert Signal envelope to internal Message
    #[allow(dead_code)]
    fn convert_envelope(&self, envelope: &SignalEnvelope) -> Option<Message> {
        let data_message = envelope.data_message.as_ref()?;
        let content = data_message.message.clone().unwrap_or_default();

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("source".to_string(), envelope.source.clone());
        metadata.insert("timestamp".to_string(), envelope.timestamp.to_string());

        if let Some(ref source_name) = envelope.source_name {
            metadata.insert("source_name".to_string(), source_name.clone());
        }

        if let Some(ref group_info) = data_message.group_info {
            metadata.insert("group_id".to_string(), group_info.group_id.clone());
        }

        let timestamp = chrono::DateTime::from_timestamp_millis(envelope.timestamp)?;

        Some(Message {
            id: uuid::Uuid::new_v4(),
            thread_id: uuid::Uuid::new_v4(),
            platform: PlatformType::Signal,
            message_type: MessageType::Text,
            content,
            metadata,
            timestamp,
        })
    }

    /// Run socket listener
    async fn run_socket_listener(&self, _event_bus: mpsc::Sender<ChannelEvent>) -> Result<()> {
        let _socket = self.socket.clone();

        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Check for incoming messages
            // In a real implementation, we'd read from the socket continuously
            // signal-cli sends messages as JSON-RPC notifications
        }
    }

    /// Run DBus listener (alternative to socket)
    async fn run_dbus_listener(&self, _event_bus: mpsc::Sender<ChannelEvent>) -> Result<()> {
        info!("Signal DBus listener started");
        // TODO: Implement DBus listener using zbus crate
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}

#[async_trait]
impl Channel for SignalChannel {
    fn name(&self) -> &str {
        "signal"
    }

    fn platform(&self) -> PlatformType {
        PlatformType::Signal
    }

    fn is_connected(&self) -> bool {
        if let Ok(connected) = self.connected.try_read() {
            *connected
        } else {
            false
        }
    }

    async fn connect(&mut self) -> Result<()> {
        match self.config.base.connection_mode {
            ConnectionMode::WebSocket => self.connect_socket().await,
            ConnectionMode::Polling => {
                // DBus mode
                *self.connected.write().await = true;
                info!("Signal channel connected via DBus");
                Ok(())
            }
            _ => Err(AgentError::platform("Signal does not support Webhook mode")),
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.stop_listener().await?;

        if let Some(socket) = self.socket.write().await.take() {
            drop(socket);
        }

        *self.connected.write().await = false;
        info!("Disconnected from Signal");
        Ok(())
    }

    async fn send(&self, channel_id: &str, message: &Message) -> Result<()> {
        // For Signal, channel_id is the phone number or group ID
        match message.message_type {
            MessageType::Text => {
                self.send_text_message(channel_id, &message.content).await?;
            }
            _ => {
                self.send_text_message(channel_id, &message.content).await?;
            }
        }
        Ok(())
    }

    async fn start_listener(&self, event_bus: mpsc::Sender<ChannelEvent>) -> Result<()> {
        self.stop_listener().await?;

        match self.config.base.connection_mode {
            ConnectionMode::WebSocket => {
                let channel = self.clone();
                let handle = tokio::spawn(async move {
                    if let Err(e) = channel.run_socket_listener(event_bus).await {
                        error!("Socket listener error: {}", e);
                    }
                });
                *self.listener_handle.write().await = Some(handle);
            }
            ConnectionMode::Polling => {
                let channel = self.clone();
                let handle = tokio::spawn(async move {
                    if let Err(e) = channel.run_dbus_listener(event_bus).await {
                        error!("DBus listener error: {}", e);
                    }
                });
                *self.listener_handle.write().await = Some(handle);
            }
            _ => {
                return Err(AgentError::platform("Signal does not support Webhook mode"));
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
        ]
    }

    async fn list_channels(&self) -> Result<Vec<ChannelInfo>> {
        // Signal groups can be listed via signal-cli
        Ok(vec![])
    }

    async fn list_members(&self, _channel_id: &str) -> Result<Vec<MemberInfo>> {
        // Signal group members can be retrieved
        Ok(vec![])
    }

    fn connection_mode(&self) -> ConnectionMode {
        self.config.base.connection_mode
    }
}

impl Clone for SignalChannel {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            connected: self.connected.clone(),
            socket: Arc::new(RwLock::new(None)),
            listener_handle: Arc::new(RwLock::new(None)),
            request_id: self.request_id.clone(),
        }
    }
}
