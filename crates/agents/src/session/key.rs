//! Session Key Implementation
//!
//! Format: agent:<agentId>:<type>:<uuid>
//! Examples:
//! - agent:abc123:session:550e8400-e29b-41d4-a716-446655440000
//! - agent:abc123:subagent:6ba7b810-9dad-11d1-80b4-00c04fd430c8

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Session key for identifying agent sessions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionKey {
    pub agent_id: String,
    pub session_type: SessionType,
    pub uuid: String,
    pub depth: u8,
}

/// Session type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionType {
    Session,
    Standard,
    Subagent,
    Cron,
    Webhook,
    Nested,
    WebSocket,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionType::Session => write!(f, "session"),
            SessionType::Standard => write!(f, "standard"),
            SessionType::Subagent => write!(f, "subagent"),
            SessionType::Cron => write!(f, "cron"),
            SessionType::Webhook => write!(f, "webhook"),
            SessionType::Nested => write!(f, "nested"),
            SessionType::WebSocket => write!(f, "websocket"),
        }
    }
}

impl FromStr for SessionType {
    type Err = SessionKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "session" => Ok(SessionType::Session),
            "standard" => Ok(SessionType::Standard),
            "subagent" => Ok(SessionType::Subagent),
            "cron" => Ok(SessionType::Cron),
            "webhook" => Ok(SessionType::Webhook),
            "nested" => Ok(SessionType::Nested),
            "websocket" => Ok(SessionType::WebSocket),
            _ => Err(SessionKeyError::InvalidSessionType(s.to_string())),
        }
    }
}

/// Session key errors
#[derive(Debug, Clone, PartialEq)]
pub enum SessionKeyError {
    InvalidFormat(String),
    InvalidSessionType(String),
    InvalidUuid(String),
    MaxDepthExceeded(u8),
}

impl fmt::Display for SessionKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionKeyError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            SessionKeyError::InvalidSessionType(s) => write!(f, "Invalid type: {}", s),
            SessionKeyError::InvalidUuid(s) => write!(f, "Invalid UUID: {}", s),
            SessionKeyError::MaxDepthExceeded(d) => write!(f, "Max depth: {}", d),
        }
    }
}

impl std::error::Error for SessionKeyError {}

impl SessionKey {
    pub const MAX_DEPTH: u8 = 5;

    pub fn new(agent_id: impl Into<String>, session_type: SessionType) -> Self {
        Self {
            agent_id: agent_id.into(),
            session_type,
            uuid: Uuid::new_v4().to_string(),
            depth: 0,
        }
    }

    /// Parse from string with explicit depth
    ///
    /// 🟠 HIGH FIX: New format includes depth: agent:<id>:<type>:<depth>:<uuid>
    pub fn parse(key: &str) -> Result<Self, SessionKeyError> {
        let parts: Vec<&str> = key.split(':').collect();

        // Support both old format (4 parts) and new format (5 parts with depth)
        match parts.len() {
            4 => Self::parse_v1(&parts),
            5 => Self::parse_v2(&parts),
            _ => Err(SessionKeyError::InvalidFormat(key.to_string())),
        }
    }

    fn parse_v1(parts: &[&str]) -> Result<Self, SessionKeyError> {
        if parts[0] != "agent" {
            return Err(SessionKeyError::InvalidFormat(parts.join(":")));
        }

        let uuid_str = parts[3];
        Uuid::parse_str(uuid_str)
            .map_err(|_| SessionKeyError::InvalidUuid(uuid_str.to_string()))?;

        // 🟠 HIGH FIX: V1 format doesn't have depth, default to 0
        Ok(Self {
            agent_id: parts[1].to_string(),
            session_type: SessionType::from_str(parts[2])?,
            uuid: uuid_str.to_string(),
            depth: 0,
        })
    }

    fn parse_v2(parts: &[&str]) -> Result<Self, SessionKeyError> {
        if parts[0] != "agent" {
            return Err(SessionKeyError::InvalidFormat(parts.join(":")));
        }

        let depth: u8 = parts[3]
            .parse()
            .map_err(|_| SessionKeyError::InvalidFormat("invalid depth".to_string()))?;

        let uuid_str = parts[4];
        Uuid::parse_str(uuid_str)
            .map_err(|_| SessionKeyError::InvalidUuid(uuid_str.to_string()))?;

        // 🟠 HIGH FIX: V2 format preserves depth
        Ok(Self {
            agent_id: parts[1].to_string(),
            session_type: SessionType::from_str(parts[2])?,
            uuid: uuid_str.to_string(),
            depth,
        })
    }

    pub fn spawn_child(&self) -> Result<Self, SessionKeyError> {
        if self.depth >= Self::MAX_DEPTH {
            return Err(SessionKeyError::MaxDepthExceeded(self.depth));
        }
        Ok(Self {
            agent_id: self.agent_id.clone(),
            session_type: SessionType::Subagent,
            uuid: Uuid::new_v4().to_string(),
            depth: self.depth + 1,
        })
    }

    pub fn to_path(&self) -> PathBuf {
        PathBuf::from(format!(
            "agents/{}/{}/{}",
            self.agent_id, self.session_type, self.uuid
        ))
    }

    pub fn to_transcript_path(&self) -> PathBuf {
        self.to_path().join("transcript.jsonl")
    }

    pub fn validate(&self) -> bool {
        !self.agent_id.is_empty() && !self.uuid.is_empty() && self.depth <= Self::MAX_DEPTH
    }

    pub fn is_subagent(&self) -> bool {
        matches!(
            self.session_type,
            SessionType::Subagent | SessionType::Nested
        )
    }

    /// Get the agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Get the session type
    pub fn session_type(&self) -> SessionType {
        self.session_type
    }

    /// Get the UUID
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// Get the depth
    pub fn depth(&self) -> u8 {
        self.depth
    }
}

impl fmt::Display for SessionKey {
    /// 🟠 HIGH FIX: New format includes depth: agent:<id>:<type>:<depth>:<uuid>
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "agent:{}:{}:{}:{}",
            self.agent_id, self.session_type, self.depth, self.uuid
        )
    }
}

impl FromStr for SessionKey {
    type Err = SessionKeyError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
