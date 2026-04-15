//! Social Groups
//!
//! Group dynamics and management.

use std::collections::HashSet;
use uuid::Uuid;

/// Social group
#[derive(Debug, Clone)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub members: HashSet<Uuid>,
    pub leader: Option<Uuid>,
}

impl Group {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            members: HashSet::new(),
            leader: None,
        }
    }

    pub fn add_member(&mut self, agent: Uuid) {
        self.members.insert(agent);
    }

    pub fn remove_member(&mut self, agent: Uuid) {
        self.members.remove(&agent);
    }

    pub fn set_leader(&mut self, agent: Uuid) {
        self.leader = Some(agent);
    }

    pub fn size(&self) -> usize {
        self.members.len()
    }
}
