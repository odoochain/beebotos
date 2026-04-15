//! Subagent Commands
//!
//! CLI commands for /subagents management (OpenClaw feature).

use serde::{Deserialize, Serialize};

use crate::session::SessionKey;

/// Subagent command handler
pub struct SubagentCommands;

/// Subagent info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentInfo {
    pub run_id: String,
    pub session_key: SessionKey,
    pub status: SubagentStatus,
    pub started_at: u64,
    pub duration_secs: u64,
}

/// Subagent status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SubagentStatus {
    Running,
    Completed,
    Error,
    Timeout,
}

impl SubagentCommands {
    pub fn new() -> Self {
        Self
    }

    /// List all subagents for a parent
    pub async fn list(&self, _parent: &SessionKey) -> Vec<SubagentInfo> {
        // Implementation would query active subagents
        vec![]
    }

    /// Stop a subagent
    pub async fn stop(&self, _run_id: &str) -> Result<(), SubagentCommandError> {
        Ok(())
    }

    /// Get subagent logs
    pub async fn log(&self, _run_id: &str, _lines: Option<usize>) -> String {
        String::new()
    }
}

/// Command errors
#[derive(Debug, Clone)]
pub enum SubagentCommandError {
    NotFound,
    StopFailed(String),
}
