//! Completion Announcement
//!
//! Sent when subagent completes execution (OpenClaw feature).

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::session::SessionKey;

/// Completion announcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    pub run_id: String,
    pub parent: SessionKey,
    pub child: SessionKey,
    pub status: CompletionStatus,
    pub result_summary: String,
    pub duration: Duration,
    pub tokens_used: TokenUsage,
    pub estimated_cost: f64,
    pub transcript_path: PathBuf,
}

/// Completion status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompletionStatus {
    Success,
    Error,
    Timeout,
    Cancelled,
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt: usize,
    pub completion: usize,
}

impl TokenUsage {
    pub fn total(&self) -> usize {
        self.prompt + self.completion
    }
}

impl Announcement {
    /// Format announcement as message
    pub fn format_message(&self) -> String {
        format!(
            "🔔 Subagent Completed\nRun ID: {}\nDuration: {:.2}s\nTokens: {} prompt + {} \
             completion = {} total\nCost: ${:.4}\nStatus: {:?}\nResult: {}",
            self.run_id,
            self.duration.as_secs_f64(),
            self.tokens_used.prompt,
            self.tokens_used.completion,
            self.tokens_used.total(),
            self.estimated_cost,
            self.status,
            self.result_summary,
        )
    }
}
