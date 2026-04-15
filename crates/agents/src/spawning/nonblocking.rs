//! Non-blocking Subagent Spawning
//!
//! Core feature from OpenClaw: sessions_spawn returns immediately
//! with {status: "accepted", runId, childSessionKey}

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::session::SessionKey;
use crate::AgentConfig;

/// Spawn configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnConfig {
    pub parent: SessionKey,
    pub task_description: String,
    pub agent_config: Option<AgentConfig>,
    pub timeout: Duration,
    pub resource_quota: ResourceQuota,
    pub thinking_depth: ThinkingDepth,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            parent: SessionKey::new("default", crate::session::SessionType::Session),
            task_description: String::new(),
            agent_config: None,
            timeout: Duration::from_secs(300),
            resource_quota: ResourceQuota::default(),
            thinking_depth: ThinkingDepth::Medium,
        }
    }
}

/// Resource quota for subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub max_memory_mb: usize,
    pub max_cpu_percent: f32,
    pub max_tokens: usize,
}

impl Default for ResourceQuota {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_percent: 50.0,
            max_tokens: 10000,
        }
    }
}

/// Thinking depth levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ThinkingDepth {
    Off,
    Low,
    Medium,
    High,
}

/// Spawn result - returned immediately (non-blocking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResult {
    pub status: SpawnStatus,
    pub run_id: String,
    pub child_session_key: SessionKey,
    pub estimated_init_time: Duration,
}

/// Spawn status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SpawnStatus {
    Accepted,
    Rejected,
    Queued,
}

impl std::fmt::Display for SpawnStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpawnStatus::Accepted => write!(f, "accepted"),
            SpawnStatus::Rejected => write!(f, "rejected"),
            SpawnStatus::Queued => write!(f, "queued"),
        }
    }
}
