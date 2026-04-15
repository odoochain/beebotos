//! Channel Manager
//!
//! Unified channel management system for managing multiple communication
//! channels. Provides channel lifecycle management, routing, and monitoring.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tokio::time::Duration;
use tracing::{debug, error, info, warn};

use super::r#trait::ConnectionMode;
use super::{Channel, ChannelEvent, PlatformType, TemplateEngine};
use crate::communication::Message;
use crate::error::{AgentError, Result};

/// Channel registration
#[derive(Clone)]
pub struct ChannelRegistration {
    /// Channel ID
    pub id: String,
    /// Channel name
    pub name: String,
    /// Platform type
    pub platform: PlatformType,
    /// Connection mode
    pub connection_mode: ConnectionMode,
    /// Whether the channel is enabled
    pub enabled: bool,
    /// Channel instance
    pub channel: Arc<dyn Channel>,
}

/// Channel status
#[derive(Debug, Clone)]
pub struct ChannelStatus {
    /// Channel ID
    pub id: String,
    /// Platform type
    pub platform: PlatformType,
    /// Whether the channel is connected
    pub connected: bool,
    /// Whether the channel is listening
    pub listening: bool,
    /// Message count (received)
    pub messages_received: u64,
    /// Message count (sent)
    pub messages_sent: u64,
    /// Error count
    pub error_count: u64,
    /// Last error message
    pub last_error: Option<String>,
    /// Last activity timestamp
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Channel manager configuration
#[derive(Debug, Clone)]
pub struct ChannelManagerConfig {
    /// Auto-connect channels on startup
    pub auto_connect: bool,
    /// Auto-start listeners on connect
    pub auto_listen: bool,
    /// Default event channel buffer size
    pub event_buffer_size: usize,
    /// Enable template engine
    pub enable_templates: bool,
    /// Template directory path
    pub template_directory: Option<String>,
}

impl Default for ChannelManagerConfig {
    fn default() -> Self {
        Self {
            auto_connect: true,
            auto_listen: true,
            event_buffer_size: 1000,
            enable_templates: true,
            template_directory: None,
        }
    }
}

/// Channel manager
pub struct ChannelManager {
    config: ChannelManagerConfig,
    channels: Arc<RwLock<HashMap<String, ChannelRegistration>>>,
    statuses: Arc<RwLock<HashMap<String, ChannelStatus>>>,
    event_tx: mpsc::Sender<ChannelEvent>,
    event_rx: Arc<RwLock<Option<mpsc::Receiver<ChannelEvent>>>>,
    template_engine: Arc<RwLock<Option<TemplateEngine>>>,
    listener_handles: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl ChannelManager {
    /// Create a new channel manager
    pub async fn new(config: ChannelManagerConfig) -> Result<(Self, mpsc::Receiver<ChannelEvent>)> {
        let (event_tx, event_rx) = mpsc::channel(config.event_buffer_size);

        let mut template_engine = None;
        if config.enable_templates {
            let mut engine = TemplateEngine::new();

            for template in super::message_templates::built_in::all() {
                engine.register_template(template)?;
            }

            if let Some(ref dir) = config.template_directory {
                engine.load_from_directory(dir)?;
            }

            template_engine = Some(engine);
        }

        let manager = Self {
            config,
            channels: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            template_engine: Arc::new(RwLock::new(template_engine)),
            listener_handles: Arc::new(RwLock::new(HashMap::new())),
        };

        let event_rx = manager.event_rx.write().await.take().unwrap();
        Ok((manager, event_rx))
    }

    /// Register a channel
    pub async fn register_channel(
        &self,
        id: &str,
        name: &str,
        channel: Arc<dyn Channel>,
    ) -> Result<()> {
        let registration = ChannelRegistration {
            id: id.to_string(),
            name: name.to_string(),
            platform: channel.platform(),
            connection_mode: channel.connection_mode(),
            enabled: true,
            channel: channel.clone(),
        };

        let status = ChannelStatus {
            id: id.to_string(),
            platform: channel.platform(),
            connected: false,
            listening: false,
            messages_received: 0,
            messages_sent: 0,
            error_count: 0,
            last_error: None,
            last_activity: None,
        };

        self.channels
            .write()
            .await
            .insert(id.to_string(), registration);
        self.statuses.write().await.insert(id.to_string(), status);

        info!("Registered channel: {} ({:?})", id, channel.platform());

        if self.config.auto_connect {
            self.connect_channel(id).await?;
        }

        Ok(())
    }

    /// Unregister a channel
    pub async fn unregister_channel(&self, id: &str) -> Result<()> {
        self.disconnect_channel(id).await.ok();

        self.channels.write().await.remove(id);
        self.statuses.write().await.remove(id);

        info!("Unregistered channel: {}", id);
        Ok(())
    }

    /// Connect a channel
    pub async fn connect_channel(&self, id: &str) -> Result<()> {
        let channel = {
            let channels = self.channels.read().await;
            let registration = channels
                .get(id)
                .ok_or_else(|| AgentError::configuration(format!("Channel not found: {}", id)))?;
            registration.channel.clone()
        };

        let mut statuses = self.statuses.write().await;
        if let Some(status) = statuses.get_mut(id) {
            status.connected = true;
            status.last_activity = Some(chrono::Utc::now());
        }

        info!("Connected channel: {}", id);

        if self.config.auto_listen {
            self.start_channel_listener(id).await?;
        }

        let _ = self
            .event_tx
            .send(ChannelEvent::ConnectionStateChanged {
                platform: channel.platform(),
                connected: true,
                reason: None,
            })
            .await;

        Ok(())
    }

    /// Disconnect a channel
    pub async fn disconnect_channel(&self, id: &str) -> Result<()> {
        self.stop_channel_listener(id).await.ok();

        let channel = {
            let channels = self.channels.read().await;
            let registration = channels
                .get(id)
                .ok_or_else(|| AgentError::configuration(format!("Channel not found: {}", id)))?;
            registration.channel.clone()
        };

        let mut statuses = self.statuses.write().await;
        if let Some(status) = statuses.get_mut(id) {
            status.connected = false;
            status.listening = false;
            status.last_activity = Some(chrono::Utc::now());
        }

        info!("Disconnected channel: {}", id);

        let _ = self
            .event_tx
            .send(ChannelEvent::ConnectionStateChanged {
                platform: channel.platform(),
                connected: false,
                reason: None,
            })
            .await;

        Ok(())
    }

    /// Start channel listener
    pub async fn start_channel_listener(&self, id: &str) -> Result<()> {
        let channel = {
            let channels = self.channels.read().await;
            let registration = channels
                .get(id)
                .ok_or_else(|| AgentError::configuration(format!("Channel not found: {}", id)))?;
            registration.channel.clone()
        };

        let event_tx = self.event_tx.clone();
        let channel_id = id.to_string();
        let statuses = self.statuses.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = channel.start_listener(event_tx.clone()).await {
                error!("Channel {} listener error: {}", channel_id, e);

                let mut statuses = statuses.write().await;
                if let Some(status) = statuses.get_mut(&channel_id) {
                    status.error_count += 1;
                    status.last_error = Some(e.to_string());
                }
            }
        });

        self.listener_handles
            .write()
            .await
            .insert(id.to_string(), handle);

        let mut statuses = self.statuses.write().await;
        if let Some(status) = statuses.get_mut(id) {
            status.listening = true;
        }

        info!("Started listener for channel: {}", id);
        Ok(())
    }

    /// Stop channel listener
    pub async fn stop_channel_listener(&self, id: &str) -> Result<()> {
        if let Some(handle) = self.listener_handles.write().await.remove(id) {
            handle.abort();
        }

        let channel = {
            let channels = self.channels.read().await;
            let registration = channels
                .get(id)
                .ok_or_else(|| AgentError::configuration(format!("Channel not found: {}", id)))?;
            registration.channel.clone()
        };

        channel.stop_listener().await?;

        let mut statuses = self.statuses.write().await;
        if let Some(status) = statuses.get_mut(id) {
            status.listening = false;
        }

        info!("Stopped listener for channel: {}", id);
        Ok(())
    }

    /// Send message through a channel
    pub async fn send_message(
        &self,
        channel_id: &str,
        target: &str,
        message: &Message,
    ) -> Result<()> {
        let channel = {
            let channels = self.channels.read().await;
            let registration = channels.get(channel_id).ok_or_else(|| {
                AgentError::configuration(format!("Channel not found: {}", channel_id))
            })?;
            registration.channel.clone()
        };

        channel.send(target, message).await?;

        let mut statuses = self.statuses.write().await;
        if let Some(status) = statuses.get_mut(channel_id) {
            status.messages_sent += 1;
            status.last_activity = Some(chrono::Utc::now());
        }

        Ok(())
    }

    /// Send message using template
    pub async fn send_template_message(
        &self,
        channel_id: &str,
        target: &str,
        template_id: &str,
        variables: &HashMap<String, String>,
    ) -> Result<()> {
        let platform = {
            let channels = self.channels.read().await;
            let registration = channels.get(channel_id).ok_or_else(|| {
                AgentError::configuration(format!("Channel not found: {}", channel_id))
            })?;
            registration.platform
        };

        let template_engine = self.template_engine.read().await;
        let engine = template_engine
            .as_ref()
            .ok_or_else(|| AgentError::configuration("Template engine not enabled"))?;

        let message = engine.create_message(template_id, variables, platform)?;
        drop(template_engine);

        self.send_message(channel_id, target, &message).await
    }

    /// Broadcast message to all channels
    pub async fn broadcast_message(&self, message: &Message) -> Vec<(String, Result<()>)> {
        let channels = self.channels.read().await;
        let mut results = Vec::new();

        for (id, registration) in channels.iter() {
            if !registration.enabled {
                continue;
            }

            let result = registration.channel.send(id, message).await;
            results.push((id.clone(), result));
        }

        results
    }

    /// Get channel status
    pub async fn get_channel_status(&self, id: &str) -> Option<ChannelStatus> {
        self.statuses.read().await.get(id).cloned()
    }

    /// Get all channel statuses
    pub async fn get_all_statuses(&self) -> Vec<ChannelStatus> {
        self.statuses.read().await.values().cloned().collect()
    }

    /// Get channels by platform
    pub async fn get_channels_by_platform(
        &self,
        platform: PlatformType,
    ) -> Vec<ChannelRegistration> {
        let channels = self.channels.read().await;
        channels
            .values()
            .filter(|r| r.platform == platform)
            .cloned()
            .collect()
    }

    /// Enable/disable channel
    pub async fn set_channel_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        let mut channels = self.channels.write().await;
        let registration = channels
            .get_mut(id)
            .ok_or_else(|| AgentError::configuration(format!("Channel not found: {}", id)))?;

        registration.enabled = enabled;

        if enabled {
            info!("Enabled channel: {}", id);
        } else {
            info!("Disabled channel: {}", id);
            drop(channels);
            self.disconnect_channel(id).await.ok();
        }

        Ok(())
    }

    /// Connect all enabled channels
    pub async fn connect_all(&self) -> Vec<(String, Result<()>)> {
        let channels = self.channels.read().await;
        let channel_ids: Vec<String> = channels
            .values()
            .filter(|r| r.enabled)
            .map(|r| r.id.clone())
            .collect();
        drop(channels);

        let mut results = Vec::new();
        for id in channel_ids {
            let result = self.connect_channel(&id).await;
            results.push((id, result));
        }

        results
    }

    /// Disconnect all channels
    pub async fn disconnect_all(&self) -> Vec<(String, Result<()>)> {
        let channels = self.channels.read().await;
        let channel_ids: Vec<String> = channels.keys().cloned().collect();
        drop(channels);

        let mut results = Vec::new();
        for id in channel_ids {
            let result = self.disconnect_channel(&id).await;
            results.push((id, result));
        }

        results
    }

    /// Get template engine
    pub async fn get_template_engine(&self) -> Option<TemplateEngine> {
        self.template_engine.read().await.clone()
    }

    /// Shutdown all channels
    pub async fn shutdown(&self) {
        info!("Shutting down channel manager...");

        for (id, _) in self.disconnect_all().await {
            debug!("Disconnected channel: {}", id);
        }

        let mut handles = self.listener_handles.write().await;
        for (id, handle) in handles.drain() {
            handle.abort();
            debug!("Aborted listener for channel: {}", id);
        }

        info!("Channel manager shutdown complete");
    }
}

/// Channel router for routing messages to appropriate channels
pub struct ChannelRouter {
    platform_routes: HashMap<PlatformType, String>,
    default_channels: HashMap<PlatformType, String>,
}

impl ChannelRouter {
    /// Create a new channel router
    pub fn new() -> Self {
        Self {
            platform_routes: HashMap::new(),
            default_channels: HashMap::new(),
        }
    }

    /// Register a route
    pub fn register_route(&mut self, platform: PlatformType, channel_id: &str) {
        self.platform_routes
            .insert(platform, channel_id.to_string());
    }

    /// Set default channel for platform
    pub fn set_default(&mut self, platform: PlatformType, channel_id: &str) {
        self.default_channels
            .insert(platform, channel_id.to_string());
    }

    /// Get channel ID for platform
    pub fn get_channel(&self, platform: PlatformType) -> Option<&String> {
        self.platform_routes
            .get(&platform)
            .or_else(|| self.default_channels.get(&platform))
    }

    /// Route message to appropriate channel
    pub fn route(&self, platform: PlatformType, _message: &Message) -> Option<&String> {
        self.get_channel(platform)
    }
}

impl Default for ChannelRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Channel health monitor
pub struct ChannelHealthMonitor {
    check_interval: Duration,
    max_errors: u64,
    health_status: Arc<RwLock<HashMap<String, bool>>>,
}

impl ChannelHealthMonitor {
    /// Create a new health monitor
    pub fn new(check_interval: Duration, max_errors: u64) -> Self {
        Self {
            check_interval,
            max_errors,
            health_status: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start monitoring
    pub async fn start(&self, manager: Arc<ChannelManager>) {
        let interval = self.check_interval;
        let max_errors = self.max_errors;
        let health_status = self.health_status.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;

                let statuses = manager.get_all_statuses().await;
                for status in statuses {
                    let healthy = status.error_count < max_errors && status.connected;

                    let mut health = health_status.write().await;
                    let was_healthy = health.get(&status.id).copied().unwrap_or(true);

                    if was_healthy && !healthy {
                        warn!("Channel {} is now unhealthy", status.id);
                    } else if !was_healthy && healthy {
                        info!("Channel {} is now healthy", status.id);
                    }

                    health.insert(status.id.clone(), healthy);
                }
            }
        });
    }

    /// Check if channel is healthy
    pub async fn is_healthy(&self, channel_id: &str) -> bool {
        self.health_status
            .read()
            .await
            .get(channel_id)
            .copied()
            .unwrap_or(true)
    }

    /// Get all unhealthy channels
    pub async fn get_unhealthy_channels(&self) -> Vec<String> {
        let health = self.health_status.read().await;
        health
            .iter()
            .filter(|(_, healthy)| !**healthy)
            .map(|(id, _)| id.clone())
            .collect()
    }
}
