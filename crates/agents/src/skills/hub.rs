//! Skills Hub

use crate::error::Result;
use crate::skills::registry::Version;
use std::collections::HashMap;

/// Skill hub manages skill lifecycle
pub struct SkillsHub {
    skills: HashMap<String, SkillInfo>,
}

#[derive(Debug, Clone)]
pub struct SkillInfo {
    pub name: String,
    pub version: Version,
    pub enabled: bool,
}

impl SkillsHub {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, info: SkillInfo) {
        self.skills.insert(info.name.clone(), info);
    }

    pub fn get(&self, name: &str) -> Option<&SkillInfo> {
        self.skills.get(name)
    }

    pub fn list(&self) -> Vec<&SkillInfo> {
        self.skills.values().collect()
    }

    pub fn enable(&mut self, name: &str) -> Result<()> {
        if let Some(skill) = self.skills.get_mut(name) {
            skill.enabled = true;
            Ok(())
        } else {
            Err(crate::error::AgentError::not_found(format!("Skill {}", name)))
        }
    }

    pub fn disable(&mut self, name: &str) -> Result<()> {
        if let Some(skill) = self.skills.get_mut(name) {
            skill.enabled = false;
            Ok(())
        } else {
            Err(crate::error::AgentError::not_found(format!("Skill {}", name)))
        }
    }
}

impl Default for SkillsHub {
    fn default() -> Self {
        Self::new()
    }
}
