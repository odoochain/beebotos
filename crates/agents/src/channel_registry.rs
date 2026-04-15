//! Channel Registry
//!
//! Provides a plugin-based architecture for registering and managing
//! communication channels. Any channel can be registered dynamically
//! without modifying the core code.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::{mpsc, RwLock};
use tracing::info;

use crate::communication::channel::{Channel, ChannelEvent, ChannelFactory};
use crate::communication::PlatformType;
use crate::deduplicator::MessageDeduplicator;
use crate::error::{AgentError, Result};

/// Channel registration information
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    /// Channel type name
    pub channel_type: String,
    /// Platform type
    pub platform: PlatformType,
    /// Whether the channel is enabled
    pub enabled: bool,
    /// Connection mode
    pub connection_mode: String,
    /// Whether the channel is currently connected
    pub is_connected: bool,
}

/// Channel Registry
///
/// Manages channel factories and instances. Provides a centralized
/// way to register, create, and manage communication channels.
///
/// # Example
/// ```ignore
/// use beebotos_agents::channel_registry::ChannelRegistry;
/// use beebotos_agents::communication::channel::{LarkChannelFactory, DingTalkChannelFactory};
/// use serde_json::json;
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// # let (tx, _rx) = tokio::sync::mpsc::channel(100);
/// let registry = ChannelRegistry::new(tx);
/// registry.register(Box::new(LarkChannelFactory)).await;
/// registry.register(Box::new(DingTalkChannelFactory)).await;
///
/// // Create channel from config
/// let config = json!({"app_id": "xxx", "app_secret": "yyy"});
/// registry.create_channel("lark", &config).await.unwrap();
/// # });
/// ```
pub struct ChannelRegistry {
    /// Registered factories
    factories: RwLock<HashMap<String, Box<dyn ChannelFactory>>>,
    /// Active channel instances
    channels: RwLock<HashMap<String, Arc<RwLock<dyn Channel>>>>,
    /// Platform to channel name mapping
    platform_map: RwLock<HashMap<PlatformType, String>>,
    /// Event bus sender
    #[allow(dead_code)]
    event_bus: mpsc::Sender<ChannelEvent>,
    /// Message deduplicator
    #[allow(dead_code)]
    deduplicator: Arc<MessageDeduplicator>,
}

impl ChannelRegistry {
    /// Create a new channel registry
    ///
    /// # Arguments
    /// * `event_bus` - Event bus for channel events
    pub fn new(event_bus: mpsc::Sender<ChannelEvent>) -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
            channels: RwLock::new(HashMap::new()),
            platform_map: RwLock::new(HashMap::new()),
            event_bus,
            deduplicator: Arc::new(MessageDeduplicator::default()),
        }
    }

    /// Register a channel factory
    ///
    /// # Arguments
    /// * `factory` - Channel factory implementation
    ///
    /// # Example
    /// ```ignore
    /// # use beebotos_agents::channel_registry::ChannelRegistry;
    /// # use beebotos_agents::communication::channel::LarkChannelFactory;
    /// # tokio::runtime::Runtime::new().unwrap().block_on(async {
    /// # let (tx, _rx) = tokio::sync::mpsc::channel(100);
    /// let registry = ChannelRegistry::new(tx);
    /// registry.register(Box::new(LarkChannelFactory)).await;
    /// # });
    /// ```
    pub async fn register(&self, factory: Box<dyn ChannelFactory>) {
        let name = factory.name().to_string();
        info!("📦 Registering channel factory: {}", name);

        let mut factories = self.factories.write().await;
        factories.insert(name, factory);
    }

    /// Create a channel from configuration
    ///
    /// # Arguments
    /// * `channel_type` - Channel type name
    /// * `config` - Channel configuration
    ///
    /// # Returns
    /// * `Result<Arc<RwLock<dyn Channel>>>` - Channel instance
    pub async fn create_channel(
        &self,
        channel_type: &str,
        config: &Value,
    ) -> Result<Arc<RwLock<dyn Channel>>> {
        let factories = self.factories.read().await;

        let factory = factories.get(channel_type).ok_or_else(|| {
            AgentError::configuration(format!("Unknown channel type: {}", channel_type))
        })?;

        let channel_arc = factory.create(config).await?;

        let mut channels = self.channels.write().await;
        channels.insert(channel_type.to_string(), channel_arc.clone());

        let mut platform_map = self.platform_map.write().await;
        platform_map.insert(factory.platform_type(), channel_type.to_string());

        info!("✅ Created channel: {}", channel_type);
        Ok(channel_arc)
    }

    /// Get channel by type
    ///
    /// # Arguments
    /// * `channel_type` - Channel type name
    ///
    /// # Returns
    /// * `Option<Arc<RwLock<dyn Channel>>>` - Channel instance if exists
    pub async fn get_channel(&self, channel_type: &str) -> Option<Arc<RwLock<dyn Channel>>> {
        let channels = self.channels.read().await;
        channels.get(channel_type).cloned()
    }

    /// Get channel by platform type
    ///
    /// # Arguments
    /// * `platform` - Platform type
    ///
    /// # Returns
    /// * `Option<Arc<RwLock<dyn Channel>>>` - Channel instance if exists
    pub async fn get_channel_by_platform(
        &self,
        platform: PlatformType,
    ) -> Option<Arc<RwLock<dyn Channel>>> {
        let platform_map = self.platform_map.read().await;
        let channel_type = platform_map.get(&platform)?;

        let channels = self.channels.read().await;
        channels.get(channel_type).cloned()
    }

    /// Get all registered channel information
    ///
    /// # Returns
    /// * `Vec<ChannelInfo>` - List of channel information
    pub async fn list_channels(&self) -> Vec<ChannelInfo> {
        let factories = self.factories.read().await;
        let channels = self.channels.read().await;

        factories
            .values()
            .map(|f| {
                let channel_type = f.name();
                let is_connected = channels.contains_key(channel_type);

                ChannelInfo {
                    channel_type: channel_type.to_string(),
                    platform: f.platform_type(),
                    enabled: true,
                    connection_mode: "websocket".to_string(),
                    is_connected,
                }
            })
            .collect()
    }

    /// Check if channel type is registered
    ///
    /// # Arguments
    /// * `channel_type` - Channel type name
    ///
    /// # Returns
    /// * `bool` - True if registered
    pub async fn is_registered(&self, channel_type: &str) -> bool {
        let factories = self.factories.read().await;
        factories.contains_key(channel_type)
    }

    /// Get the number of registered factories
    pub async fn factory_count(&self) -> usize {
        let factories = self.factories.read().await;
        factories.len()
    }

    /// Get the number of active channels
    pub async fn channel_count(&self) -> usize {
        let channels = self.channels.read().await;
        channels.len()
    }

    /// Remove a channel
    ///
    /// # Arguments
    /// * `channel_type` - Channel type name
    pub async fn remove_channel(&self, channel_type: &str) -> Result<()> {
        let mut channels = self.channels.write().await;
        let mut platform_map = self.platform_map.write().await;

        if let Some(channel) = channels.remove(channel_type) {
            // Find and remove from platform map
            let platform = {
                let ch = channel.read().await;
                ch.platform()
            };
            platform_map.remove(&platform);

            info!("🗑️  Removed channel: {}", channel_type);
        }

        Ok(())
    }

    /// Get channel by message ID prefix
    ///
    /// # Arguments
    /// * `msg_id` - Message ID (e.g., "lark:om_xxx" or "lark_om_xxx")
    ///
    /// # Returns
    /// * `Option<Arc<RwLock<dyn Channel>>>` - Channel instance if found
    pub async fn get_channel_by_msg_id(&self, msg_id: &str) -> Option<Arc<RwLock<dyn Channel>>> {
        // Try different prefix patterns
        let prefixes: Vec<&str> = if msg_id.contains(':') {
            msg_id.split(':').next().into_iter().collect()
        } else if msg_id.contains('_') {
            msg_id.split('_').next().into_iter().collect()
        } else {
            vec![]
        };

        for prefix in prefixes {
            if let Some(channel) = self.get_channel(prefix).await {
                return Some(channel);
            }
        }

        None
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        let (tx, _rx) = mpsc::channel(1000);
        Self::new(tx)
    }
}

/// Channel registry builder
///
/// Provides a fluent API for building channel registries
///
/// # Example
/// ```ignore
/// use beebotos_agents::channel_registry::ChannelRegistryBuilder;
/// use beebotos_agents::communication::channel::LarkChannelFactory;
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let registry = ChannelRegistryBuilder::new()
///     .with_channel(Box::new(LarkChannelFactory))
///     .build();
/// # });
/// ```
pub struct ChannelRegistryBuilder {
    factories: Vec<Box<dyn ChannelFactory>>,
    event_bus: Option<mpsc::Sender<ChannelEvent>>,
}

impl ChannelRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
            event_bus: None,
        }
    }

    /// Add a channel factory
    ///
    /// # Arguments
    /// * `factory` - Channel factory implementation
    pub fn with_channel(mut self, factory: Box<dyn ChannelFactory>) -> Self {
        self.factories.push(factory);
        self
    }

    /// Set event bus
    ///
    /// # Arguments
    /// * `event_bus` - Event bus sender
    pub fn with_event_bus(mut self, event_bus: mpsc::Sender<ChannelEvent>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Build the channel registry
    ///
    /// # Returns
    /// * `ChannelRegistry` - Configured registry
    pub async fn build(self) -> ChannelRegistry {
        let event_bus = self
            .event_bus
            .unwrap_or_else(|| {
                let (tx, _rx) = mpsc::channel(1000);
                tx
            });

        let registry = ChannelRegistry::new(event_bus);

        for factory in self.factories {
            registry.register(factory).await;
        }

        registry
    }
}

impl Default for ChannelRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
