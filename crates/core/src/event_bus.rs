//! System-wide Event Bus for decoupled inter-module communication
//!
//! This module provides a type-safe, asynchronous event bus that allows
//! different modules to communicate without direct dependencies.
//!
//! # Example
//! ```
//! use beebotos_core::event_bus::{SystemEvent, SystemEventBus};
//!
//! #[derive(Clone)]
//! struct MyEvent {
//!     data: String,
//! }
//! impl SystemEvent for MyEvent {
//!     fn event_type(&self) -> &'static str {
//!         "my.event"
//!     }
//!     fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
//!         chrono::Utc::now()
//!     }
//! }
//!
//! let bus = SystemEventBus::new();
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info, warn};

/// System event trait - all events must implement this
pub trait SystemEvent: Clone + Send + Sync + 'static {
    /// Returns the event type as a string
    fn event_type(&self) -> &'static str;
    /// Returns the event timestamp
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;
}

/// Type-erased event channel for storage
struct TypeErasedChannel {
    sender: broadcast::Sender<Arc<dyn Any + Send + Sync>>,
    #[allow(dead_code)]
    type_id: TypeId,
    #[allow(dead_code)]
    type_name: &'static str,
}

/// Unified system event bus
pub struct SystemEventBus {
    channels: RwLock<HashMap<TypeId, TypeErasedChannel>>,
    subscribers: RwLock<HashMap<TypeId, Vec<mpsc::UnboundedSender<Arc<dyn Any + Send + Sync>>>>>,
}

impl SystemEventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
            subscribers: RwLock::new(HashMap::new()),
        }
    }

    /// Register an event type with the bus
    pub async fn register_event_type<T: SystemEvent>(&self, buffer_size: usize) {
        let type_id = TypeId::of::<T>();
        let (sender, _) = broadcast::channel(buffer_size);

        let mut channels = self.channels.write().await;
        channels.insert(
            type_id,
            TypeErasedChannel {
                sender,
                type_id,
                type_name: std::any::type_name::<T>(),
            },
        );

        info!("Registered event type: {}", std::any::type_name::<T>());
    }

    /// Publish an event to all subscribers
    pub async fn publish<T: SystemEvent>(&self, event: T) -> Result<(), EventBusError> {
        let type_id = TypeId::of::<T>();
        let channels = self.channels.read().await;

        let channel = channels
            .get(&type_id)
            .ok_or_else(|| EventBusError::TypeNotRegistered(std::any::type_name::<T>()))?;

        // Send to broadcast channel
        let event_arc: Arc<dyn Any + Send + Sync> = Arc::new(event);
        match channel.sender.send(event_arc.clone()) {
            Ok(recv_count) => debug!("Published event to {} subscribers", recv_count),
            Err(_) => warn!("No active subscribers for event type"),
        }

        // Send to direct subscribers
        let subscribers = self.subscribers.read().await;
        if let Some(subs) = subscribers.get(&type_id) {
            for sub in subs {
                let _ = sub.send(event_arc.clone());
            }
        }

        Ok(())
    }

    /// Subscribe to events (broadcast mode)
    ///
    /// Returns a type-erased receiver that can be converted to the specific
    /// type. Use `TypedEventReceiver` wrapper for type-safe access.
    pub async fn subscribe<T: SystemEvent>(&self) -> Result<TypedEventReceiver<T>, EventBusError> {
        let type_id = TypeId::of::<T>();
        let channels = self.channels.read().await;

        let channel = channels
            .get(&type_id)
            .ok_or_else(|| EventBusError::TypeNotRegistered(std::any::type_name::<T>()))?;

        Ok(TypedEventReceiver::new(channel.sender.subscribe()))
    }

    /// Check if an event type is registered
    pub async fn is_registered<T: SystemEvent>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        let channels = self.channels.read().await;
        channels.contains_key(&type_id)
    }

    /// Get registered event type count
    pub async fn registered_types_count(&self) -> usize {
        let channels = self.channels.read().await;
        channels.len()
    }
}

impl Default for SystemEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Type-safe event receiver wrapper
///
/// Wraps the underlying type-erased receiver and performs type conversion
/// on receive operations.
pub struct TypedEventReceiver<T: SystemEvent> {
    inner: broadcast::Receiver<Arc<dyn Any + Send + Sync>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: SystemEvent> TypedEventReceiver<T> {
    fn new(inner: broadcast::Receiver<Arc<dyn Any + Send + Sync>>) -> Self {
        Self {
            inner,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Receive an event from the channel
    ///
    /// Returns `Ok(T)` on success, or an error if the type conversion fails
    /// or the channel is closed.
    pub async fn recv(&mut self) -> Result<T, EventBusError> {
        match self.inner.recv().await {
            Ok(arc_any) => {
                // Downcast to the specific type
                arc_any
                    .downcast::<T>()
                    .map(|arc| (*arc).clone())
                    .map_err(|_| {
                        EventBusError::TypeMismatch("Failed to downcast event".to_string())
                    })
            }
            Err(broadcast::error::RecvError::Closed) => Err(EventBusError::ChannelClosed),
            Err(broadcast::error::RecvError::Lagged(n)) => Err(EventBusError::Lagged(n)),
        }
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Result<T, EventBusError> {
        match self.inner.try_recv() {
            Ok(arc_any) => arc_any
                .downcast::<T>()
                .map(|arc| (*arc).clone())
                .map_err(|_| EventBusError::TypeMismatch("Failed to downcast event".to_string())),
            Err(broadcast::error::TryRecvError::Empty) => Err(EventBusError::ChannelEmpty),
            Err(broadcast::error::TryRecvError::Closed) => Err(EventBusError::ChannelClosed),
            Err(broadcast::error::TryRecvError::Lagged(n)) => Err(EventBusError::Lagged(n)),
        }
    }

    /// Resubscribe to the channel
    pub fn resubscribe(&self) -> Self {
        Self::new(self.inner.resubscribe())
    }
}

impl<T: SystemEvent> Clone for TypedEventReceiver<T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.resubscribe())
    }
}

/// Event bus error types
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    /// Event type not registered in the bus
    #[error("Event type not registered: {0}")]
    TypeNotRegistered(&'static str),
    /// Error sending event
    #[error("Send error: {0}")]
    SendError(String),
    /// Type mismatch in channel
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    /// Channel is closed
    #[error("Channel closed")]
    ChannelClosed,
    /// Channel is empty
    #[error("Channel empty")]
    ChannelEmpty,
    /// Consumer lagged behind by N messages
    #[error("Lagged by {0} messages")]
    Lagged(u64),
}

/// Global event bus handle type
pub type EventBusHandle = Arc<SystemEventBus>;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestEvent {
        data: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    }

    impl SystemEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            "test.event"
        }

        fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
            self.timestamp
        }
    }

    #[tokio::test]
    async fn test_event_bus_basic() {
        let bus = SystemEventBus::new();
        bus.register_event_type::<TestEvent>(100).await;

        let mut rx = bus.subscribe::<TestEvent>().await.unwrap();

        let event = TestEvent {
            data: "hello".to_string(),
            timestamp: chrono::Utc::now(),
        };

        bus.publish(event.clone()).await.unwrap();

        // TypedEventReceiver automatically handles type conversion
        let received = rx.recv().await.unwrap();
        assert_eq!(received.data, "hello");
    }

    #[tokio::test]
    async fn test_unregistered_type() {
        let bus = SystemEventBus::new();

        let event = TestEvent {
            data: "test".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let result = bus.publish(event).await;
        assert!(result.is_err());
    }
}
