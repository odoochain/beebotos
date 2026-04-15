//! WhatsApp Channel Implementation
//!
//! Unified Channel trait implementation for WhatsApp using Baileys Bridge.
//! Uses WebSocket-like connection via Node.js bridge process.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Child;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::r#trait::{BaseChannelConfig, ConnectionMode, ContentType};
use super::{Channel, ChannelConfig, ChannelEvent, ChannelInfo, MemberInfo};
use crate::communication::{Message, MessageType, PlatformType};
use crate::error::{AgentError, Result};

/// WhatsApp Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppChannelConfig {
    /// Path to Node.js executable
    #[serde(default = "default_node_path")]
    pub node_path: String,
    /// Path to Baileys bridge script
    #[serde(default = "default_bridge_path")]
    pub bridge_path: String,
    /// Authentication directory
    #[serde(default = "default_auth_dir")]
    pub auth_dir: String,
    /// Media download directory
    #[serde(default = "default_media_dir")]
    pub media_dir: String,
    /// Reconnect interval in milliseconds
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval_ms: u64,
    /// Whether to print QR code to console
    #[serde(default = "default_print_qr")]
    pub print_qr: bool,
    /// Log level for bridge
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Base channel configuration
    #[serde(flatten)]
    pub base: BaseChannelConfig,
}

fn default_node_path() -> String {
    "node".to_string()
}

fn default_bridge_path() -> String {
    "./tools/whatsapp-baileys-bridge/whatsapp-baileys-bridge.js".to_string()
}

fn default_auth_dir() -> String {
    "./data/whatsapp/auth".to_string()
}

fn default_media_dir() -> String {
    "./data/whatsapp/media".to_string()
}

fn default_reconnect_interval() -> u64 {
    5000
}

fn default_print_qr() -> bool {
    true
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for WhatsAppChannelConfig {
    fn default() -> Self {
        Self {
            node_path: default_node_path(),
            bridge_path: default_bridge_path(),
            auth_dir: default_auth_dir(),
            media_dir: default_media_dir(),
            reconnect_interval_ms: default_reconnect_interval(),
            print_qr: default_print_qr(),
            log_level: default_log_level(),
            base: BaseChannelConfig::default(),
        }
    }
}

impl ChannelConfig for WhatsAppChannelConfig {
    fn from_env() -> Option<Self>
    where
        Self: Sized,
    {
        let base = BaseChannelConfig::from_env("WHATSAPP").unwrap_or_default();

        Some(Self {
            node_path: std::env::var("WHATSAPP_NODE_PATH").unwrap_or_else(|_| default_node_path()),
            bridge_path: std::env::var("WHATSAPP_BRIDGE_PATH")
                .unwrap_or_else(|_| default_bridge_path()),
            auth_dir: std::env::var("WHATSAPP_AUTH_DIR").unwrap_or_else(|_| default_auth_dir()),
            media_dir: std::env::var("WHATSAPP_MEDIA_DIR").unwrap_or_else(|_| default_media_dir()),
            reconnect_interval_ms: std::env::var("WHATSAPP_RECONNECT_INTERVAL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default_reconnect_interval()),
            print_qr: std::env::var("WHATSAPP_PRINT_QR")
                .map(|s| s != "false")
                .unwrap_or(default_print_qr()),
            log_level: std::env::var("WHATSAPP_LOG_LEVEL").unwrap_or_else(|_| default_log_level()),
            base,
        })
    }

    fn is_valid(&self) -> bool {
        !self.node_path.is_empty() && !self.bridge_path.is_empty()
    }

    fn allowlist(&self) -> Vec<String> {
        vec![]
    }

    fn connection_mode(&self) -> ConnectionMode {
        // WhatsApp Baileys uses WebSocket-like connection
        ConnectionMode::WebSocket
    }

    fn auto_reconnect(&self) -> bool {
        self.base.auto_reconnect
    }

    fn max_reconnect_attempts(&self) -> u32 {
        self.base.max_reconnect_attempts
    }
}

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatsAppConnectionState {
    Disconnected,
    Connecting,
    QrCode,
    Connected,
    Reconnecting,
}

/// Bridge command
#[derive(Debug, Clone, Serialize)]
struct BridgeCommand {
    #[serde(rename = "type")]
    cmd_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    data: serde_json::Value,
}

/// Bridge event
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum BridgeEvent {
    #[serde(rename = "connected")]
    Connected { data: ConnectedData },
    #[serde(rename = "disconnected")]
    Disconnected { data: DisconnectedData },
    #[serde(rename = "qr")]
    Qr { data: QrData },
    #[serde(rename = "message")]
    Message { data: WhatsAppIncomingMessage },
    #[serde(rename = "send_result")]
    SendResult {
        #[allow(dead_code)]
        id: Option<String>,
        #[allow(dead_code)]
        success: bool,
        #[allow(dead_code)]
        data: serde_json::Value,
    },
    #[serde(rename = "error")]
    Error {
        #[allow(dead_code)]
        id: Option<String>,
        data: ErrorData,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct ConnectedData {
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "userName")]
    user_name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DisconnectedData {
    reason: String,
    #[allow(dead_code)]
    #[serde(rename = "willReconnect")]
    will_reconnect: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct QrData {
    qr: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ErrorData {
    message: String,
}

/// Incoming WhatsApp message
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppIncomingMessage {
    pub id: String,
    #[serde(rename = "remoteJid")]
    pub remote_jid: String,
    #[serde(rename = "fromMe")]
    pub from_me: bool,
    pub content: String,
    #[serde(rename = "messageType")]
    pub message_type: String,
    pub timestamp: i64,
    #[serde(rename = "pushName")]
    pub push_name: Option<String>,
    #[serde(rename = "isGroup")]
    pub is_group: bool,
}

/// WhatsApp Channel implementation
pub struct WhatsAppChannel {
    config: WhatsAppChannelConfig,
    state: Arc<RwLock<WhatsAppConnectionState>>,
    child: Arc<Mutex<Option<Child>>>,
    command_tx: Arc<Mutex<Option<mpsc::UnboundedSender<BridgeCommand>>>>,
    message_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<WhatsAppIncomingMessage>>>>,
    qr_code: Arc<RwLock<Option<String>>>,
    user: Arc<RwLock<Option<(String, String)>>>, // (user_id, user_name)
    listener_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl WhatsAppChannel {
    /// Create a new WhatsApp channel
    pub fn new(config: WhatsAppChannelConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(WhatsAppConnectionState::Disconnected)),
            child: Arc::new(Mutex::new(None)),
            command_tx: Arc::new(Mutex::new(None)),
            message_rx: Arc::new(Mutex::new(None)),
            qr_code: Arc::new(RwLock::new(None)),
            user: Arc::new(RwLock::new(None)),
            listener_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let config = WhatsAppChannelConfig::from_env()
            .ok_or_else(|| AgentError::configuration("Failed to load WhatsApp configuration"))?;
        Ok(Self::new(config))
    }

    /// Get current QR code
    pub async fn qr_code(&self) -> Option<String> {
        self.qr_code.read().await.clone()
    }

    /// Get current user
    pub async fn user(&self) -> Option<(String, String)> {
        self.user.read().await.clone()
    }

    /// Start the bridge process
    async fn start_bridge(&self) -> Result<()> {
        let mut child_lock = self.child.lock().await;

        if child_lock.is_some() {
            return Ok(());
        }

        // Ensure directories exist
        tokio::fs::create_dir_all(&self.config.auth_dir)
            .await
            .map_err(|e| AgentError::platform(format!("Failed to create auth dir: {}", e)))?;
        tokio::fs::create_dir_all(&self.config.media_dir)
            .await
            .map_err(|e| AgentError::platform(format!("Failed to create media dir: {}", e)))?;

        // Set environment variables
        let mut envs = HashMap::new();
        envs.insert(
            "WHATSAPP_AUTH_DIR".to_string(),
            self.config.auth_dir.clone(),
        );
        envs.insert(
            "WHATSAPP_MEDIA_DIR".to_string(),
            self.config.media_dir.clone(),
        );
        envs.insert(
            "WHATSAPP_RECONNECT_INTERVAL".to_string(),
            self.config.reconnect_interval_ms.to_string(),
        );
        envs.insert(
            "WHATSAPP_MAX_RECONNECT_ATTEMPTS".to_string(),
            self.config.base.max_reconnect_attempts.to_string(),
        );
        envs.insert(
            "WHATSAPP_LOG_LEVEL".to_string(),
            self.config.log_level.clone(),
        );
        envs.insert(
            "WHATSAPP_PRINT_QR".to_string(),
            self.config.print_qr.to_string(),
        );

        // Spawn bridge process
        let mut child = tokio::process::Command::new(&self.config.node_path)
            .arg(&self.config.bridge_path)
            .envs(&envs)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AgentError::platform(format!("Failed to start bridge: {}", e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AgentError::platform("Failed to get bridge stdin"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgentError::platform("Failed to get bridge stdout"))?;

        // Create channels for communication
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<BridgeCommand>();
        let (msg_tx, msg_rx) = mpsc::unbounded_channel::<WhatsAppIncomingMessage>();

        *self.command_tx.lock().await = Some(cmd_tx);
        *self.message_rx.lock().await = Some(msg_rx);

        // Spawn stdin writer task
        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(cmd) = cmd_rx.recv().await {
                let json = match serde_json::to_string(&cmd) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("Failed to serialize command: {}", e);
                        continue;
                    }
                };

                if let Err(e) = stdin.write_all(json.as_bytes()).await {
                    error!("Failed to write to bridge stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.write_all(b"\n").await {
                    error!("Failed to write newline to bridge stdin: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    error!("Failed to flush bridge stdin: {}", e);
                    break;
                }
            }
        });

        // Spawn stdout reader task
        let state = self.state.clone();
        let qr_code = self.qr_code.clone();
        let user = self.user.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                debug!("Bridge output: {}", line);

                match serde_json::from_str::<BridgeEvent>(&line) {
                    Ok(event) => match event {
                        BridgeEvent::Connected { data } => {
                            info!(
                                "WhatsApp connected as {} ({})",
                                data.user_name, data.user_id
                            );
                            *state.write().await = WhatsAppConnectionState::Connected;
                            *user.write().await = Some((data.user_id, data.user_name));
                            *qr_code.write().await = None;
                        }
                        BridgeEvent::Disconnected { data } => {
                            warn!("WhatsApp disconnected: {}", data.reason);
                            *state.write().await = WhatsAppConnectionState::Disconnected;
                        }
                        BridgeEvent::Qr { data } => {
                            info!("WhatsApp QR code received");
                            *state.write().await = WhatsAppConnectionState::QrCode;
                            *qr_code.write().await = Some(data.qr.clone());
                        }
                        BridgeEvent::Message { data } => {
                            let _ = msg_tx.send(data);
                        }
                        BridgeEvent::Error { data, .. } => {
                            error!("Bridge error: {}", data.message);
                        }
                        _ => {}
                    },
                    Err(e) => {
                        error!("Failed to parse bridge event: {}", e);
                    }
                }
            }
        });

        *child_lock = Some(child);
        info!("WhatsApp bridge started");

        Ok(())
    }

    /// Stop the bridge
    async fn stop_bridge(&self) -> Result<()> {
        // Send disconnect command
        let _ = self
            .send_command(BridgeCommand {
                cmd_type: "disconnect".to_string(),
                id: None,
                data: serde_json::json!({}),
            })
            .await;

        // Kill the process
        let mut child_lock = self.child.lock().await;
        if let Some(mut child) = child_lock.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }

        *self.state.write().await = WhatsAppConnectionState::Disconnected;
        info!("WhatsApp bridge stopped");

        Ok(())
    }

    /// Send command to bridge
    async fn send_command(&self, cmd: BridgeCommand) -> Result<()> {
        let command_tx = self.command_tx.lock().await;
        if let Some(tx) = command_tx.as_ref() {
            tx.send(cmd)
                .map_err(|e| AgentError::platform(format!("Failed to send command: {}", e)))?;
            Ok(())
        } else {
            Err(AgentError::platform("Bridge not running").into())
        }
    }

    /// Send text message
    pub async fn send_text_message(&self, to: &str, text: &str) -> Result<String> {
        let cmd_id = Uuid::new_v4().to_string();

        self.send_command(BridgeCommand {
            cmd_type: "send_text".to_string(),
            id: Some(cmd_id.clone()),
            data: serde_json::json!({
                "to": to,
                "text": text,
            }),
        })
        .await?;

        // For simplicity, return the command ID
        // In a full implementation, we'd wait for the send_result event
        Ok(cmd_id)
    }

    /// Convert WhatsApp message to internal Message
    fn convert_message(&self, wa_msg: &WhatsAppIncomingMessage) -> Option<Message> {
        let message_type = match wa_msg.message_type.as_str() {
            "text" => MessageType::Text,
            "image" => MessageType::Image,
            "video" => MessageType::Video,
            "audio" | "voice" => MessageType::Voice,
            "document" => MessageType::File,
            _ => MessageType::System,
        };

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("remote_jid".to_string(), wa_msg.remote_jid.clone());
        metadata.insert("message_id".to_string(), wa_msg.id.clone());
        metadata.insert("from_me".to_string(), wa_msg.from_me.to_string());
        metadata.insert("is_group".to_string(), wa_msg.is_group.to_string());

        if let Some(ref push_name) = wa_msg.push_name {
            metadata.insert("sender_name".to_string(), push_name.clone());
        }

        let timestamp = chrono::DateTime::from_timestamp(wa_msg.timestamp, 0)?;

        Some(Message {
            id: uuid::Uuid::new_v4(),
            thread_id: uuid::Uuid::new_v4(),
            platform: PlatformType::WhatsApp,
            message_type,
            content: wa_msg.content.clone(),
            metadata,
            timestamp,
        })
    }

    /// Run listener
    async fn run_listener(&self, event_bus: mpsc::Sender<ChannelEvent>) -> Result<()> {
        let mut interval = interval(Duration::from_millis(100));

        loop {
            interval.tick().await;

            let mut rx = self.message_rx.lock().await;
            if let Some(ref mut rx) = *rx {
                // Drain all available messages
                while let Ok(wa_msg) = rx.try_recv() {
                    if let Some(message) = self.convert_message(&wa_msg) {
                        let event = ChannelEvent::MessageReceived {
                            platform: PlatformType::WhatsApp,
                            channel_id: wa_msg.remote_jid.clone(),
                            message,
                        };
                        let _ = event_bus.send(event).await;
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Channel for WhatsAppChannel {
    fn name(&self) -> &str {
        "whatsapp"
    }

    fn platform(&self) -> PlatformType {
        PlatformType::WhatsApp
    }

    fn is_connected(&self) -> bool {
        if let Ok(state) = self.state.try_read() {
            *state == WhatsAppConnectionState::Connected
        } else {
            false
        }
    }

    async fn connect(&mut self) -> Result<()> {
        self.start_bridge().await?;

        // Wait for connection (with timeout)
        let timeout = Duration::from_secs(120);
        let result = tokio::time::timeout(timeout, async {
            loop {
                let state = self.state.read().await.clone();
                if state == WhatsAppConnectionState::Connected {
                    return Ok(());
                }
                if state == WhatsAppConnectionState::Disconnected {
                    return Err::<(), AgentError>(AgentError::platform("Connection failed"));
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(AgentError::platform("Connection timeout").into()),
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.stop_listener().await?;
        self.stop_bridge().await
    }

    async fn send(&self, channel_id: &str, message: &Message) -> Result<()> {
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

        let channel = self.clone();
        let handle = tokio::spawn(async move {
            if let Err(e) = channel.run_listener(event_bus).await {
                error!("WhatsApp listener error: {}", e);
            }
        });
        *self.listener_handle.write().await = Some(handle);

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
            ContentType::Location,
            ContentType::Sticker,
        ]
    }

    async fn list_channels(&self) -> Result<Vec<ChannelInfo>> {
        // WhatsApp doesn't have an API to list all chats
        Ok(vec![])
    }

    async fn list_members(&self, _channel_id: &str) -> Result<Vec<MemberInfo>> {
        // WhatsApp groups have member info
        Ok(vec![])
    }

    fn connection_mode(&self) -> ConnectionMode {
        ConnectionMode::WebSocket
    }
}

impl Clone for WhatsAppChannel {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
            child: Arc::new(Mutex::new(None)),
            command_tx: Arc::new(Mutex::new(None)),
            message_rx: Arc::new(Mutex::new(None)),
            qr_code: self.qr_code.clone(),
            user: self.user.clone(),
            listener_handle: Arc::new(RwLock::new(None)),
        }
    }
}
