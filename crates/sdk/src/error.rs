//! SDK Errors
//!
//! Error types for the SDK.

use std::fmt;

/// SDK result type
pub type Result<T> = std::result::Result<T, SdkError>;

/// SDK errors
#[derive(Debug, Clone)]
pub enum SdkError {
    /// Invalid argument
    InvalidArgument(String),
    /// Not found
    NotFound(String),
    /// Already exists
    AlreadyExists(String),
    /// Permission denied
    PermissionDenied(String),
    /// Rate limited
    RateLimited,
    /// Timeout
    Timeout,
    /// Network error
    Network(String),
    /// Serialization error
    Serialization(String),
    /// Storage error
    Storage(String),
    /// API error
    Api { code: u16, message: String },
    /// Internal error
    Internal(String),
    /// Quota exceeded
    QuotaExceeded {
        resource: String,
        limit: u64,
        used: u64,
    },
    /// Not implemented
    NotImplemented(String),
    /// Cancelled
    Cancelled,
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SdkError::InvalidArgument(s) => write!(f, "Invalid argument: {}", s),
            SdkError::NotFound(s) => write!(f, "Not found: {}", s),
            SdkError::AlreadyExists(s) => write!(f, "Already exists: {}", s),
            SdkError::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
            SdkError::RateLimited => write!(f, "Rate limited"),
            SdkError::Timeout => write!(f, "Operation timed out"),
            SdkError::Network(s) => write!(f, "Network error: {}", s),
            SdkError::Serialization(s) => write!(f, "Serialization error: {}", s),
            SdkError::Storage(s) => write!(f, "Storage error: {}", s),
            SdkError::Api { code, message } => write!(f, "API error {}: {}", code, message),
            SdkError::Internal(s) => write!(f, "Internal error: {}", s),
            SdkError::QuotaExceeded {
                resource,
                limit,
                used,
            } => {
                write!(
                    f,
                    "Quota exceeded for {}: {}/{} used",
                    resource, used, limit
                )
            }
            SdkError::NotImplemented(s) => write!(f, "Not implemented: {}", s),
            SdkError::Cancelled => write!(f, "Operation cancelled"),
        }
    }
}

impl std::error::Error for SdkError {}

impl SdkError {
    /// Is this error retryable?
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SdkError::Timeout
                | SdkError::Network(_)
                | SdkError::RateLimited
                | SdkError::Api {
                    code: 429 | 500 | 502 | 503 | 504,
                    ..
                }
        )
    }

    /// HTTP status code equivalent
    pub fn http_status(&self) -> u16 {
        match self {
            SdkError::InvalidArgument(_) => 400,
            SdkError::PermissionDenied(_) => 403,
            SdkError::NotFound(_) => 404,
            SdkError::AlreadyExists(_) => 409,
            SdkError::RateLimited => 429,
            SdkError::Timeout => 408,
            SdkError::Api { code, .. } => *code,
            _ => 500,
        }
    }

    /// Error code string
    pub fn code(&self) -> &'static str {
        match self {
            SdkError::InvalidArgument(_) => "INVALID_ARGUMENT",
            SdkError::NotFound(_) => "NOT_FOUND",
            SdkError::AlreadyExists(_) => "ALREADY_EXISTS",
            SdkError::PermissionDenied(_) => "PERMISSION_DENIED",
            SdkError::RateLimited => "RATE_LIMITED",
            SdkError::Timeout => "TIMEOUT",
            SdkError::Network(_) => "NETWORK_ERROR",
            SdkError::Serialization(_) => "SERIALIZATION_ERROR",
            SdkError::Storage(_) => "STORAGE_ERROR",
            SdkError::Api { .. } => "API_ERROR",
            SdkError::Internal(_) => "INTERNAL_ERROR",
            SdkError::QuotaExceeded { .. } => "QUOTA_EXCEEDED",
            SdkError::NotImplemented(_) => "NOT_IMPLEMENTED",
            SdkError::Cancelled => "CANCELLED",
        }
    }
}

// Conversions from other error types

impl From<std::io::Error> for SdkError {
    fn from(e: std::io::Error) -> Self {
        SdkError::Storage(e.to_string())
    }
}

impl From<serde_json::Error> for SdkError {
    fn from(e: serde_json::Error) -> Self {
        SdkError::Serialization(e.to_string())
    }
}

impl From<std::time::SystemTimeError> for SdkError {
    fn from(e: std::time::SystemTimeError) -> Self {
        SdkError::Internal(e.to_string())
    }
}

impl From<reqwest::Error> for SdkError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            SdkError::Timeout
        } else {
            SdkError::Network(e.to_string())
        }
    }
}

/// Error context for better error messages
#[derive(Debug)]
pub struct ErrorContext {
    pub operation: String,
    pub resource: Option<String>,
    pub cause: Option<SdkError>,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            resource: None,
            cause: None,
        }
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn with_cause(mut self, cause: SdkError) -> Self {
        self.cause = Some(cause);
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "during {}", self.operation)?;
        if let Some(resource) = &self.resource {
            write!(f, " on {}", resource)?;
        }
        if let Some(cause) = &self.cause {
            write!(f, ": {}", cause)?;
        }
        Ok(())
    }
}
