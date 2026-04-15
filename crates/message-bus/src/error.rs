//! Error types for the message bus

use thiserror::Error;

/// Result type alias for message bus operations
pub type Result<T> = std::result::Result<T, MessageBusError>;

/// Message bus error types
#[derive(Error, Debug, Clone)]
pub enum MessageBusError {
    /// Transport error
    #[error("Transport error: {0}")]
    Transport(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Topic not found
    #[error("Topic not found: {0}")]
    TopicNotFound(String),

    /// Invalid topic format
    #[error("Invalid topic format: {0}")]
    InvalidTopic(String),

    /// Subscription closed
    #[error("Subscription closed")]
    SubscriptionClosed,

    /// Request timeout
    #[error("Request timeout")]
    RequestTimeout,

    /// Codec not available
    #[error("Codec not available: {0}")]
    CodecNotAvailable(String),

    /// Router error
    #[error("Router error: {0}")]
    Router(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl MessageBusError {
    /// Create a transport error
    pub fn transport(msg: impl Into<String>) -> Self {
        Self::Transport(msg.into())
    }

    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a deserialization error
    pub fn deserialization(msg: impl Into<String>) -> Self {
        Self::Deserialization(msg.into())
    }

    /// Create a topic not found error
    pub fn topic_not_found(topic: impl Into<String>) -> Self {
        Self::TopicNotFound(topic.into())
    }

    /// Create an invalid topic error
    pub fn invalid_topic(msg: impl Into<String>) -> Self {
        Self::InvalidTopic(msg.into())
    }

    /// Create a router error
    pub fn router(msg: impl Into<String>) -> Self {
        Self::Router(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::RequestTimeout)
    }

    /// Check if this is a subscription closed error
    pub fn is_subscription_closed(&self) -> bool {
        matches!(self, Self::SubscriptionClosed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = MessageBusError::transport("connection failed");
        assert!(matches!(err, MessageBusError::Transport(_)));
        assert_eq!(err.to_string(), "Transport error: connection failed");

        let err = MessageBusError::invalid_topic("empty topic");
        assert!(err.is_timeout() == false);
    }

    #[test]
    fn test_error_checks() {
        let timeout = MessageBusError::RequestTimeout;
        assert!(timeout.is_timeout());
        assert!(!timeout.is_subscription_closed());

        let closed = MessageBusError::SubscriptionClosed;
        assert!(!closed.is_timeout());
        assert!(closed.is_subscription_closed());
    }
}
