//! Theory of Mind
//!
//! Modeling other agents' mental states.

use std::collections::HashMap;
use uuid::Uuid;

/// Belief about another agent
#[derive(Debug, Clone)]
pub struct AgentModel {
    pub agent_id: Uuid,
    pub beliefs: HashMap<String, f32>,
    pub desires: Vec<String>,
    pub intentions: Vec<String>,
    pub emotional_state: Option<String>,
}

/// Theory of Mind engine
pub struct TheoryOfMind {
    models: HashMap<Uuid, AgentModel>,
}

impl TheoryOfMind {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn observe(&mut self, agent: Uuid, action: &str) {
        let model = self.models.entry(agent).or_insert(AgentModel {
            agent_id: agent,
            beliefs: HashMap::new(),
            desires: Vec::new(),
            intentions: Vec::new(),
            emotional_state: None,
        });

        // Update model based on observation
        tracing::info!("Observed agent {} performing: {}", agent, action);
    }

    pub fn predict_intent(&self, agent: Uuid) -> Option<String> {
        self.models.get(&agent).and_then(|m| {
            m.intentions.first().cloned()
        })
    }

    pub fn model(&self, agent: Uuid) -> Option<&AgentModel> {
        self.models.get(&agent)
    }
}

impl Default for TheoryOfMind {
    fn default() -> Self {
        Self::new()
    }
}
