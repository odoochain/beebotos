//! Skill Loader
//!
//! Loads and manages WASM-based skills from ClawHub.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Skill loader
pub struct SkillLoader {
    skill_paths: Vec<PathBuf>,
    loaded_skills: HashMap<String, LoadedSkill>,
}

use crate::skills::registry::Version;

/// Loaded skill info
#[derive(Debug, Clone)]
pub struct LoadedSkill {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub wasm_path: PathBuf,
    pub manifest: SkillManifest,
}

/// Skill manifest
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillManifest {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
    pub entry_point: String,
}

impl SkillLoader {
    pub fn new() -> Self {
        Self {
            skill_paths: vec![],
            loaded_skills: HashMap::new(),
        }
    }

    /// Add skill search path
    pub fn add_path(&mut self, path: impl AsRef<Path>) {
        self.skill_paths.push(path.as_ref().to_path_buf());
    }

    /// Load skill from path
    pub async fn load_skill(&mut self, skill_id: &str) -> Result<LoadedSkill, SkillLoadError> {
        // Check if already loaded
        if let Some(skill) = self.loaded_skills.get(skill_id) {
            return Ok(skill.clone());
        }

        // Search for skill in paths
        for path in &self.skill_paths {
            let skill_path = path.join(skill_id);
            if skill_path.exists() {
                let manifest = self.load_manifest(&skill_path).await?;
                let wasm_path = skill_path.join("skill.wasm");

                let skill = LoadedSkill {
                    id: skill_id.to_string(),
                    name: manifest.name.clone(),
                    version: manifest.version.clone(),
                    wasm_path,
                    manifest,
                };

                self.loaded_skills
                    .insert(skill_id.to_string(), skill.clone());
                return Ok(skill);
            }
        }

        Err(SkillLoadError::SkillNotFound(skill_id.to_string()))
    }

    /// Load manifest from skill directory
    async fn load_manifest(&self, path: &Path) -> Result<SkillManifest, SkillLoadError> {
        let manifest_path = path.join("skill.yaml");
        let content = tokio::fs::read_to_string(&manifest_path)
            .await
            .map_err(|e| SkillLoadError::IoError(e.to_string()))?;

        let manifest: SkillManifest = serde_yaml::from_str(&content)
            .map_err(|e| SkillLoadError::ParseError(e.to_string()))?;

        Ok(manifest)
    }

    /// Get loaded skill
    pub fn get_skill(&self, skill_id: &str) -> Option<&LoadedSkill> {
        self.loaded_skills.get(skill_id)
    }

    /// List loaded skills
    pub fn list_skills(&self) -> Vec<&LoadedSkill> {
        self.loaded_skills.values().collect()
    }

    /// Unload skill
    pub fn unload_skill(&mut self, skill_id: &str) {
        self.loaded_skills.remove(skill_id);
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Skill load errors
#[derive(Debug, Clone)]
pub enum SkillLoadError {
    SkillNotFound(String),
    IoError(String),
    ParseError(String),
    InvalidManifest(String),
}

impl std::fmt::Display for SkillLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillLoadError::SkillNotFound(s) => write!(f, "Skill not found: {}", s),
            SkillLoadError::IoError(s) => write!(f, "IO error: {}", s),
            SkillLoadError::ParseError(s) => write!(f, "Parse error: {}", s),
            SkillLoadError::InvalidManifest(s) => write!(f, "Invalid manifest: {}", s),
        }
    }
}

impl std::error::Error for SkillLoadError {}
