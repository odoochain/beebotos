//! Skill Registry
//!
//! Central registry for skill discovery and management.
//!
//! 🟠 HIGH FIX: Now thread-safe with RwLock for concurrent access.
//! CODE QUALITY FIX: Use tokio::sync::RwLock for async compatibility.

use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::skills::loader::LoadedSkill;

/// Skill registry
///
/// 🟠 HIGH FIX: Thread-safe with RwLock
pub struct SkillRegistry {
    skills: RwLock<HashMap<String, RegisteredSkill>>,
    categories: RwLock<HashMap<String, Vec<String>>>,
}

/// Semantic version
///
/// 🟡 MEDIUM FIX: Proper version management
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(version: &str) -> Result<Self, VersionError> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat(version.to_string()));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| VersionError::InvalidNumber(parts[0].to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| VersionError::InvalidNumber(parts[1].to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| VersionError::InvalidNumber(parts[2].to_string()))?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Version errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum VersionError {
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),
    #[error("Invalid version number: {0}")]
    InvalidNumber(String),
}

/// Skill definition for registry
#[derive(Debug, Clone)]
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
}

/// Registered skill
#[derive(Debug, Clone)]
pub struct RegisteredSkill {
    pub skill: LoadedSkill,
    pub category: String,
    pub tags: Vec<String>,
    pub installed_at: u64,
    pub usage_count: u64,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: RwLock::new(HashMap::new()),
            categories: RwLock::new(HashMap::new()),
        }
    }

    /// Register a skill
    ///
    /// 🟠 HIGH FIX: Thread-safe with write locks
    /// CODE QUALITY FIX: Use async RwLock for async compatibility
    pub async fn register(&self, skill: LoadedSkill, category: impl Into<String>, tags: Vec<String>) {
        let skill_id = skill.id.clone();
        let category = category.into();

        let registered = RegisteredSkill {
            skill,
            category: category.clone(),
            tags,
            installed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(std::time::Duration::from_secs(0))
                .as_secs(),
            usage_count: 0,
        };

        {
            let mut skills = self.skills.write().await;
            skills.insert(skill_id.clone(), registered);
        }

        // Add to category
        let mut categories = self.categories.write().await;
        categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(skill_id);
    }

    /// Get skill by ID
    ///
    /// 🟠 HIGH FIX: Thread-safe with read lock
    /// CODE QUALITY FIX: Use async RwLock for async compatibility
    pub async fn get(&self, skill_id: &str) -> Option<RegisteredSkill> {
        let skills = self.skills.read().await;
        skills.get(skill_id).cloned()
    }

    /// Find skills by category
    ///
    /// 🟠 HIGH FIX: Thread-safe with read locks
    /// CODE QUALITY FIX: Use async RwLock for async compatibility
    pub async fn by_category(&self, category: &str) -> Vec<RegisteredSkill> {
        let categories = self.categories.read().await;
        let skills = self.skills.read().await;

        categories
            .get(category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| skills.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find skills by tag
    ///
    /// 🟠 HIGH FIX: Thread-safe with read lock
    /// CODE QUALITY FIX: Use async RwLock for async compatibility
    pub async fn by_tag(&self, tag: &str) -> Vec<RegisteredSkill> {
        let skills = self.skills.read().await;
        skills
            .values()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Search skills by name
    ///
    /// 🟠 HIGH FIX: Thread-safe with read lock
    /// CODE QUALITY FIX: Use async RwLock for async compatibility
    pub async fn search(&self, query: &str) -> Vec<RegisteredSkill> {
        let skills = self.skills.read().await;
        let query_lower = query.to_lowercase();
        skills
            .values()
            .filter(|s| {
                s.skill.name.to_lowercase().contains(&query_lower)
                    || s.skill
                        .manifest
                        .description
                        .to_lowercase()
                        .contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// List all skills
    ///
    /// 🟠 HIGH FIX: Thread-safe with read lock
    /// CODE QUALITY FIX: Use async RwLock for async compatibility
    pub async fn list_all(&self) -> Vec<RegisteredSkill> {
        let skills = self.skills.read().await;
        skills.values().cloned().collect()
    }

    /// Increment usage count
    ///
    /// 🟠 HIGH FIX: Thread-safe with write lock
    /// CODE QUALITY FIX: Use async RwLock for async compatibility
    pub async fn record_usage(&self, skill_id: &str) {
        let mut skills = self.skills.write().await;
        if let Some(skill) = skills.get_mut(skill_id) {
            skill.usage_count += 1;
        }
    }

    /// Unregister skill
    ///
    /// 🟠 HIGH FIX: Thread-safe with write lock
    /// CODE QUALITY FIX: Use async RwLock for async compatibility
    pub async fn unregister(&self, skill_id: &str) -> Option<RegisteredSkill> {
        let mut skills = self.skills.write().await;
        skills.remove(skill_id)
    }

    /// Get categories
    ///
    /// 🟠 HIGH FIX: Thread-safe with read lock
    /// CODE QUALITY FIX: Use async RwLock for async compatibility
    pub async fn categories(&self) -> Vec<String> {
        let categories = self.categories.read().await;
        categories.keys().cloned().collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
