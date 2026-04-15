//! Social Relationships
//!
//! Managing relationships between agents.

use std::collections::HashMap;
use uuid::Uuid;

/// Relationship types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipType {
    Friend,
    Enemy,
    Neutral,
    Ally,
    Rival,
}

/// Relationship
#[derive(Debug, Clone)]
pub struct Relationship {
    pub with_agent: Uuid,
    pub relationship_type: RelationshipType,
    pub trust: f32,
    pub intimacy: f32,
}

/// Relationship manager
pub struct RelationshipManager {
    relationships: HashMap<Uuid, Relationship>,
}

impl RelationshipManager {
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
        }
    }

    pub fn add(&mut self, relationship: Relationship) {
        self.relationships.insert(relationship.with_agent, relationship);
    }

    pub fn get(&self, agent: Uuid) -> Option<&Relationship> {
        self.relationships.get(&agent)
    }

    pub fn update_trust(&mut self, agent: Uuid, delta: f32) {
        if let Some(rel) = self.relationships.get_mut(&agent) {
            rel.trust = (rel.trust + delta).clamp(0.0, 1.0);
        }
    }
}

impl Default for RelationshipManager {
    fn default() -> Self {
        Self::new()
    }
}
