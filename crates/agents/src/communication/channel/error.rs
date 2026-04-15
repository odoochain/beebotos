//! Channel Error Types
//!
//! CODE QUALITY FIX: Unified error types for all communication channels.
//! All channel implementations should use these error types for consistency.

use thiserror::Error;

/// Unified error type for channel operations
#[derive(Error, Debug, Clone)]
pub enum ChannelError {
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Message send failed
    #[error("Failed to send message: {0}")]
    SendFailed(String),

    /// Message receive failed
    #[error("Failed to receive message: {0}")]
    ReceiveFailed(String),

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimited(String),

    /// Channel not initialized
    #[error("Channel not initialized")]
    NotInitialized,

    /// Channel already connected
    #[error("Channel already connected")]
    AlreadyConnected,

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// API error from platform
    #[error("Platform API error {code}: {message}")]
    Api { code: u16, message: String },

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for channel operations
pub type ChannelResult<T> = Result<T, ChannelError>;

/// Convert from string to ChannelError
impl From<String> for ChannelError {
    fn from(s: String) -> Self {
        ChannelError::Unknown(s)
    }
}

/// Convert from &str to ChannelError
impl From<&str> for ChannelError {
    fn from(s: &str) -> Self {
        ChannelError::Unknown(s.to_string())
    }
}
