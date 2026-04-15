//! Agent types

use serde::{Deserialize, Serialize};

/// Agent unique identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub [u8; 32]);

impl AgentId {
    /// Generate new random agent ID
    pub fn new() -> Self {
        Self(rand::random())
    }

    /// Create from string identifier
    ///
    /// 🔴 CRITICAL SECURITY FIX: Uses BLAKE3 hash for strings > 32 bytes
    /// to prevent collision attacks from truncated IDs.
    pub fn from_string(id: impl AsRef<str>) -> Self {
        let id_str = id.as_ref();
        let bytes = id_str.as_bytes();

        if bytes.len() <= 32 {
            // Short ID: direct copy
            let mut arr = [0u8; 32];
            arr[..bytes.len()].copy_from_slice(bytes);
            Self(arr)
        } else {
            // 🔴 CRITICAL SECURITY FIX: Long IDs are hashed to prevent collisions
            let hash = blake3::hash(bytes);
            Self(*hash.as_bytes())
        }
    }

    /// From string (hex)
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes[..32.min(bytes.len())]);
        Ok(Self(arr))
    }

    /// To hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "agent_{}", &self.to_hex()[..16])
    }
}

/// DID (Decentralized Identifier)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    pub method: String,
    pub identifier: String,
}

impl DID {
    /// Create new DID
    pub fn new(method: impl Into<String>, identifier: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            identifier: identifier.into(),
        }
    }

    /// Parse from string with validation
    ///
    /// 🟡 MEDIUM FIX: Validates per DID Core spec
    pub fn parse(did: &str) -> Option<Self> {
        let parts: Vec<&str> = did.split(':').collect();

        // Basic format check
        if parts.len() < 3 || parts[0] != "did" {
            return None;
        }

        let method = parts[1];
        let identifier = parts[2..].join(":");

        // 🟡 MEDIUM FIX: Validate method name (lowercase only)
        if !method.chars().all(|c| c.is_ascii_lowercase()) {
            return None;
        }

        // 🟡 MEDIUM FIX: Validate identifier is not empty
        if identifier.is_empty() {
            return None;
        }

        Some(Self {
            method: method.to_string(),
            identifier,
        })
    }

    /// Validate the entire DID
    ///
    /// 🟡 MEDIUM FIX: Full validation per W3C DID Core
    pub fn validate(&self) -> Result<(), DIDError> {
        // Method must be lowercase
        if !self.method.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(DIDError::InvalidMethod(self.method.clone()));
        }

        // Identifier must not be empty
        if self.identifier.is_empty() {
            return Err(DIDError::EmptyIdentifier);
        }

        // Identifier should only contain valid chars
        let valid_chars = |c: char| c.is_ascii_alphanumeric() || ".-_:%".contains(c);
        if !self.identifier.chars().all(valid_chars) {
            return Err(DIDError::InvalidIdentifier(self.identifier.clone()));
        }

        Ok(())
    }
}

/// DID validation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum DIDError {
    #[error("Invalid method name: {0}")]
    InvalidMethod(String),
    #[error("Empty identifier")]
    EmptyIdentifier,
    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),
}

impl std::fmt::Display for DID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "did:{}:{}", self.method, self.identifier)
    }
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Task identifier
///
/// 🟠 HIGH FIX: Uses UUID v4 instead of atomic counter
/// to prevent overflow and provide better uniqueness guarantees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub uuid::Uuid);

impl TaskId {
    /// Generate new task ID using UUID v4
    pub fn new() -> Self {
        // UUID v4 provides 122 bits of randomness - no overflow risk
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}
