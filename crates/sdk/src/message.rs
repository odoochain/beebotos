//! Message Types
//!
//! A2A messaging primitives.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{AgentId, TaskId};

/// Message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Text message
    Text,
    /// Task request
    TaskRequest,
    /// Task response
    TaskResponse,
    /// Status update
    StatusUpdate,
    /// Error notification
    Error,
    /// Capability advertisement
    CapabilityAdvert,
    /// Ping/keepalive
    Ping,
    /// Pong response
    Pong,
}

/// Message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub message_type: MessageType,
    pub from: AgentId,
    pub to: AgentId,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub timestamp: u64,
    pub task_id: Option<TaskId>,
    pub parent_id: Option<String>,
}

impl Message {
    /// Create reply to this message
    pub fn reply(&self, content: impl Into<String>) -> MessageBuilder {
        MessageBuilder::new(MessageType::Text)
            .from(self.to.clone())
            .to(self.from.clone())
            .content(content)
            .parent(self.id.clone())
    }

    /// Check if this is a reply
    pub fn is_reply(&self) -> bool {
        self.parent_id.is_some()
    }

    /// Get thread ID (root message ID)
    pub fn thread_id(&self) -> &str {
        self.parent_id.as_ref().unwrap_or(&self.id)
    }
}

/// Message builder
pub struct MessageBuilder {
    message_type: MessageType,
    from: Option<AgentId>,
    to: Option<AgentId>,
    content: String,
    metadata: HashMap<String, String>,
    task_id: Option<TaskId>,
    parent_id: Option<String>,
}

impl MessageBuilder {
    pub fn new(message_type: MessageType) -> Self {
        Self {
            message_type,
            from: None,
            to: None,
            content: String::new(),
            metadata: HashMap::new(),
            task_id: None,
            parent_id: None,
        }
    }

    pub fn from(mut self, agent_id: AgentId) -> Self {
        self.from = Some(agent_id);
        self
    }

    pub fn to(mut self, agent_id: AgentId) -> Self {
        self.to = Some(agent_id);
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn task(mut self, task_id: TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    pub fn parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    pub fn build(self) -> Message {
        Message {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: self.message_type,
            from: self.from.expect("from is required"),
            to: self.to.expect("to is required"),
            content: self.content,
            metadata: self.metadata,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            task_id: self.task_id,
            parent_id: self.parent_id,
        }
    }
}

/// Task request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task_type: String,
    pub input: serde_json::Value,
    pub parameters: HashMap<String, String>,
    pub deadline: Option<u64>,
}

impl TaskRequest {
    pub fn new(task_type: impl Into<String>) -> Self {
        Self {
            task_type: task_type.into(),
            input: serde_json::Value::Null,
            parameters: HashMap::new(),
            deadline: None,
        }
    }

    pub fn with_input(mut self, input: impl serde::Serialize) -> Result<Self, serde_json::Error> {
        self.input = serde_json::to_value(input)?;
        Ok(self)
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    pub fn with_deadline(mut self, deadline: u64) -> Self {
        self.deadline = Some(deadline);
        self
    }
}

/// Task response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<TaskError>,
    pub token_usage: crate::types::TokenUsage,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl TaskResponse {
    pub fn success(output: impl serde::Serialize) -> Result<Self, serde_json::Error> {
        Ok(Self {
            success: true,
            output: Some(serde_json::to_value(output)?),
            error: None,
            token_usage: crate::types::TokenUsage::default(),
            execution_time_ms: 0,
        })
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: None,
            error: Some(TaskError {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
            token_usage: crate::types::TokenUsage::default(),
            execution_time_ms: 0,
        }
    }
}

/// Status update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub agent_id: AgentId,
    pub status: AgentStatus,
    pub current_task: Option<TaskId>,
    pub load: f32, // 0.0 - 1.0
    pub queue_size: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Busy,
    Paused,
    Error,
    ShuttingDown,
}

impl StatusUpdate {
    pub fn new(agent_id: AgentId, status: AgentStatus) -> Self {
        Self {
            agent_id,
            status,
            current_task: None,
            load: 0.0,
            queue_size: 0,
        }
    }

    pub fn with_load(mut self, load: f32) -> Self {
        self.load = load.clamp(0.0, 1.0);
        self
    }
}

/// Capability advertisement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAdvert {
    pub agent_id: AgentId,
    pub capabilities: Vec<Capability>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub version: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

impl Capability {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            version: "1.0.0".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn with_input_schema(mut self, schema: impl serde::Serialize) -> Result<Self, serde_json::Error> {
        self.input_schema = serde_json::to_value(schema)?;
        Ok(self)
    }

    pub fn with_output_schema(mut self, schema: impl serde::Serialize) -> Result<Self, serde_json::Error> {
        self.output_schema = serde_json::to_value(schema)?;
        Ok(self)
    }
}
