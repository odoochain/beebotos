//! Storage backend for skills

use crate::models::Skill;

#[allow(dead_code)]
pub struct Storage {
    // Would use proper database in production
}

#[allow(dead_code)]
impl Storage {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn save_skill(&self, skill: &Skill) -> anyhow::Result<()> {
        // Save to database
        let _ = skill;
        Ok(())
    }

    pub async fn get_skill(&self, id: &str) -> anyhow::Result<Option<Skill>> {
        // Load from database
        let _ = id;
        Ok(None)
    }
}
