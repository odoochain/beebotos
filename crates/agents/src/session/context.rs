//! Session Context Management
//!
//! Manages conversation context and state for agent sessions.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Session context for maintaining conversation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: String,
    pub variables: HashMap<String, String>,
    pub message_count: usize,
    pub tokens_used: usize,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SessionContext {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            variables: HashMap::new(),
            message_count: 0,
            tokens_used: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn set_variable(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }

    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    pub fn increment_messages(&mut self) {
        self.message_count += 1;
    }

    pub fn add_tokens(&mut self, tokens: usize) {
        self.tokens_used += tokens;
    }
}
