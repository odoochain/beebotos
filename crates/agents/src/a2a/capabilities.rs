use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: Vec<CapabilityParameter>,
    pub returns: CapabilityReturn,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityParameter {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub description: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<ParameterType>),
    Object(HashMap<String, ParameterType>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityReturn {
    pub return_type: ParameterType,
    pub description: String,
}

pub struct CapabilityRegistry {
    capabilities: HashMap<String, AgentCapability>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    pub fn register(&mut self, capability: AgentCapability) {
        self.capabilities.insert(capability.id.clone(), capability);
    }

    pub fn get(&self, id: &str) -> Option<&AgentCapability> {
        self.capabilities.get(id)
    }

    pub fn list_all(&self) -> Vec<&AgentCapability> {
        self.capabilities.values().collect()
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&AgentCapability> {
        self.capabilities
            .values()
            .filter(|c| c.name.contains(tag) || c.description.contains(tag))
            .collect()
    }
}
