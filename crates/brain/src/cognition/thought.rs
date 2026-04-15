//! Thought Representation

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A thought
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub id: Uuid,
    pub content: String,
    pub confidence: f64,
    pub source: ThoughtSource,
}

/// Source of thought
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThoughtSource {
    Perception,
    Memory,
    Inference,
    Imagination,
}

impl Thought {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            content: content.into(),
            confidence: 1.0,
            source: ThoughtSource::Inference,
        }
    }
}
