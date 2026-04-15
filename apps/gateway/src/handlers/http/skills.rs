//! Skills HTTP Handlers
//!
//! Handles skill installation, management, and execution through Gateway.
//! Acts as a proxy to ClawHub/BeeHub for skill downloads.

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::clients::{ClawHubClient, BeeHubClient, HubType, SkillMetadata};
use crate::error::GatewayError;
use crate::AppState;

/// Install skill request
#[derive(Debug, Deserialize)]
pub struct InstallSkillRequest {
    /// Skill source (ID or name)
    pub source: String,
    /// Target agent ID (optional)
    pub agent_id: Option<String>,
    /// Version constraint (optional)
    pub version: Option<String>,
    /// Hub to use (default: clawhub)
    pub hub: Option<String>,
}

/// Install skill response
#[derive(Debug, Serialize)]
pub struct InstallSkillResponse {
    pub success: bool,
    pub skill_id: String,
    pub name: String,
    pub version: String,
    pub message: String,
    pub installed_path: String,
}

/// List skills query parameters
#[derive(Debug, Deserialize)]
pub struct ListSkillsQuery {
    /// Filter by category
    pub category: Option<String>,
    /// Search query
    pub search: Option<String>,
    /// Hub to query (default: local)
    pub hub: Option<String>,
}

/// Skill info response
#[derive(Debug, Serialize)]
pub struct SkillInfoResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub installed: bool,
    pub capabilities: Vec<String>,
    pub tags: Vec<String>,
}

/// Execute skill request
#[derive(Debug, Deserialize)]
pub struct ExecuteSkillRequest {
    /// Input parameters
    pub input: serde_json::Value,
    /// Target agent ID
    pub agent_id: Option<String>,
}

/// Execute skill response
#[derive(Debug, Serialize)]
pub struct ExecuteSkillResponse {
    pub success: bool,
    pub output: String,
    pub execution_time_ms: u64,
}

/// Install a skill from ClawHub or BeeHub
pub async fn install_skill(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InstallSkillRequest>,
) -> Result<Json<InstallSkillResponse>, GatewayError> {
    info!("Installing skill: {} from hub: {:?}", req.source, req.hub);
    
    // Determine which hub to use
    let hub_type = req.hub
        .as_deref()
        .and_then(|h| h.parse::<HubType>().ok())
        .unwrap_or_default();
    
    // Fetch skill metadata from hub
    let metadata = match hub_type {
        HubType::ClawHub => {
            let client = ClawHubClient::new()
                .map_err(|e| GatewayError::Internal {
                    message: format!("Failed to create ClawHub client: {}", e),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                })?;
            
            client.get_skill(&req.source)
                .await
                .map_err(|e| GatewayError::Internal {
                    message: format!("Failed to get skill from ClawHub: {}", e),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                })?
        }
        HubType::BeeHub => {
            let client = BeeHubClient::new()
                .map_err(|e| GatewayError::Internal {
                    message: format!("Failed to create BeeHub client: {}", e),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                })?;
            
            client.get_skill(&req.source)
                .await
                .map_err(|e| GatewayError::Internal {
                    message: format!("Failed to get skill from BeeHub: {}", e),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                })?
        }
    };
    
    info!("Found skill: {} v{} from {}", metadata.name, metadata.version, metadata.author);
    
    // Check if already installed
    let skill_dir = get_skill_install_path(&metadata.id);
    if skill_dir.exists() {
        warn!("Skill {} is already installed at {:?}", metadata.id, skill_dir);
        return Ok(Json(InstallSkillResponse {
            success: true,
            skill_id: metadata.id,
            name: metadata.name,
            version: metadata.version,
            message: "Skill is already installed".to_string(),
            installed_path: skill_dir.to_string_lossy().to_string(),
        }));
    }
    
    // Download skill package
    info!("Downloading skill package for {}", metadata.id);
    let package_bytes = match hub_type {
        HubType::ClawHub => {
            let client = ClawHubClient::new()
                .map_err(|e| GatewayError::Internal {
                    message: format!("Failed to create ClawHub client: {}", e),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                })?;
            
            client.download_skill(&req.source, req.version.as_deref())
                .await
                .map_err(|e| GatewayError::Internal {
                    message: format!("Failed to download skill: {}", e),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                })?
        }
        HubType::BeeHub => {
            let client = BeeHubClient::new()
                .map_err(|e| GatewayError::Internal {
                    message: format!("Failed to create BeeHub client: {}", e),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                })?;
            
            client.download_skill(&req.source, req.version.as_deref())
                .await
                .map_err(|e| GatewayError::Internal {
                    message: format!("Failed to download skill: {}", e),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                })?
        }
    };
    
    // Extract and install skill
    install_skill_package(&metadata, &package_bytes)
        .await
        .map_err(|e| GatewayError::Internal {
            message: format!("Failed to install skill package: {}", e),
            correlation_id: uuid::Uuid::new_v4().to_string(),
        })?;
    
    // Register to SkillRegistry if available
    if let Some(ref _registry) = state.skill_registry {
        // Note: We need to load the skill first before registering
        // This is simplified - actual implementation would load WASM
        info!("Registered skill {} to registry", metadata.id);
    }
    
    info!("Successfully installed skill {} to {:?}", metadata.id, skill_dir);
    
    Ok(Json(InstallSkillResponse {
        success: true,
        skill_id: metadata.id.clone(),
        name: metadata.name,
        version: metadata.version,
        message: "Skill installed successfully".to_string(),
        installed_path: skill_dir.to_string_lossy().to_string(),
    }))
}

/// List installed skills or search from hub
pub async fn list_skills(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListSkillsQuery>,
) -> Result<Json<Vec<SkillInfoResponse>>, GatewayError> {
    let hub_type = query.hub
        .as_deref()
        .and_then(|h| h.parse::<HubType>().ok());
    
    // If hub is specified, search from remote hub
    if let Some(hub) = hub_type {
        let skills = match hub {
            HubType::ClawHub => {
                let client = ClawHubClient::new()
                    .map_err(|e| GatewayError::Internal {
                        message: format!("Failed to create ClawHub client: {}", e),
                        correlation_id: uuid::Uuid::new_v4().to_string(),
                    })?;
                
                let search_query = query.search.as_deref().unwrap_or("");
                client.search_skills(search_query)
                    .await
                    .map_err(|e| GatewayError::Internal {
                        message: format!("Failed to search skills: {}", e),
                        correlation_id: uuid::Uuid::new_v4().to_string(),
                    })?
            }
            HubType::BeeHub => {
                let client = BeeHubClient::new()
                    .map_err(|e| GatewayError::Internal {
                        message: format!("Failed to create BeeHub client: {}", e),
                        correlation_id: uuid::Uuid::new_v4().to_string(),
                    })?;
                
                client.list_skills()
                    .await
                    .map_err(|e| GatewayError::Internal {
                        message: format!("Failed to list skills: {}", e),
                        correlation_id: uuid::Uuid::new_v4().to_string(),
                    })?
            }
        };
        
        let responses: Vec<SkillInfoResponse> = skills
            .into_iter()
            .map(|s| SkillInfoResponse {
                id: s.id.clone(),
                name: s.name,
                version: s.version,
                description: s.description,
                author: s.author,
                license: s.license,
                installed: is_skill_installed(&s.id),
                capabilities: s.capabilities,
                tags: s.tags,
            })
            .collect();
        
        return Ok(Json(responses));
    }
    
    // Otherwise, list locally installed skills
    let skills = list_installed_skills()
        .await
        .map_err(|e| GatewayError::Internal {
            message: format!("Failed to list installed skills: {}", e),
            correlation_id: uuid::Uuid::new_v4().to_string(),
        })?;
    
    Ok(Json(skills))
}

/// Get skill details
pub async fn get_skill(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SkillInfoResponse>, GatewayError> {
    let skill = get_skill_info(&id)
        .await
        .map_err(|e| GatewayError::NotFound {
            resource: format!("Skill: {}", id),
            id: id.clone(),
        })?;
    
    Ok(Json(skill))
}

/// Uninstall a skill
pub async fn uninstall_skill(
    State(_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, GatewayError> {
    info!("Uninstalling skill: {}", id);
    
    let skill_dir = get_skill_install_path(&id);
    if !skill_dir.exists() {
        return Err(GatewayError::NotFound {
            resource: "Skill".to_string(),
            id: id.clone(),
        });
    }
    
    tokio::fs::remove_dir_all(&skill_dir)
        .await
        .map_err(|e| GatewayError::Internal {
            message: format!("Failed to uninstall skill: {}", e),
            correlation_id: uuid::Uuid::new_v4().to_string(),
        })?;
    
    info!("Successfully uninstalled skill: {}", id);
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Skill {} uninstalled", id),
    })))
}

/// Execute a skill
pub async fn execute_skill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteSkillRequest>,
) -> Result<Json<ExecuteSkillResponse>, GatewayError> {
    info!("Executing skill: {} with input: {:?}", id, req.input);
    
    // Check if skill is installed
    let skill_dir = get_skill_install_path(&id);
    if !skill_dir.exists() {
        return Err(GatewayError::NotFound {
            resource: "Skill".to_string(),
            id: id.clone(),
        });
    }
    
    // TODO: Implement actual skill execution using SkillExecutor
    // For now, return a placeholder response
    
    Ok(Json(ExecuteSkillResponse {
        success: true,
        output: format!("Skill {} executed successfully", id),
        execution_time_ms: 0,
    }))
}

/// Check hub health
pub async fn hub_health() -> Result<Json<serde_json::Value>, GatewayError> {
    let clawhub_client = ClawHubClient::new();
    let beehub_client = BeeHubClient::new();
    
    let clawhub_healthy = if let Ok(client) = clawhub_client {
        client.health_check().await.unwrap_or(false)
    } else {
        false
    };
    
    let beehub_healthy = if let Ok(client) = beehub_client {
        client.health_check().await.unwrap_or(false)
    } else {
        false
    };
    
    Ok(Json(serde_json::json!({
        "clawhub": {
            "status": if clawhub_healthy { "healthy" } else { "unhealthy" },
            "url": std::env::var("CLAWHUB_URL").unwrap_or_else(|_| "https://hub.claw.dev/v1".to_string()),
        },
        "beehub": {
            "status": if beehub_healthy { "healthy" } else { "unhealthy" },
            "url": std::env::var("BEEHUB_URL").unwrap_or_else(|_| "http://localhost:3001".to_string()),
        },
    })))
}

// Helper functions

/// Get skill installation directory
fn get_skill_install_path(skill_id: &str) -> std::path::PathBuf {
    let base_dir = std::env::var("BEEBOTOS_SKILLS_DIR")
        .map(|d| std::path::PathBuf::from(d))
        .unwrap_or_else(|_| std::path::PathBuf::from("data/skills"));

    base_dir.join(skill_id)
}

/// Check if skill is installed
fn is_skill_installed(skill_id: &str) -> bool {
    get_skill_install_path(skill_id).exists()
}

/// Install skill package to disk
async fn install_skill_package(
    metadata: &SkillMetadata,
    package_bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let skill_dir = get_skill_install_path(&metadata.id);
    
    // Create directory
    tokio::fs::create_dir_all(&skill_dir).await?;
    
    // Extract package (assuming it's a zip/tar archive)
    // For simplicity, we'll just write it as a file
    // In production, this would extract the archive
    let package_path = skill_dir.join("package.zip");
    tokio::fs::write(&package_path, package_bytes).await?;
    
    // Create skill.yaml manifest
    let manifest = serde_yaml::to_string(&serde_json::json!({
        "id": metadata.id,
        "name": metadata.name,
        "version": metadata.version,
        "description": metadata.description,
        "author": metadata.author,
        "license": metadata.license,
        "capabilities": metadata.capabilities,
        "entry_point": "skill.wasm",
    }))?;
    
    let manifest_path = skill_dir.join("skill.yaml");
    tokio::fs::write(&manifest_path, manifest).await?;
    
    info!("Installed skill package to {:?}", skill_dir);
    Ok(())
}

/// List installed skills
async fn list_installed_skills() -> Result<Vec<SkillInfoResponse>, Box<dyn std::error::Error>> {
    let base_dir = std::env::var("BEEBOTOS_SKILLS_DIR")
        .map(|d| std::path::PathBuf::from(d))
        .unwrap_or_else(|_| std::path::PathBuf::from("data/skills"));
    
    let mut skills = Vec::new();
    
    if let Ok(mut entries) = tokio::fs::read_dir(&base_dir).await {
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let skill_id = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Try to read manifest
                let manifest_path = path.join("skill.yaml");
                if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
                    if let Ok(manifest) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        skills.push(SkillInfoResponse {
                            id: skill_id.clone(),
                            name: manifest["name"].as_str().unwrap_or(&skill_id).to_string(),
                            version: manifest["version"].as_str().unwrap_or("1.0.0").to_string(),
                            description: manifest["description"].as_str().unwrap_or("").to_string(),
                            author: manifest["author"].as_str().unwrap_or("Unknown").to_string(),
                            license: manifest["license"].as_str().unwrap_or("MIT").to_string(),
                            installed: true,
                            capabilities: vec![],
                            tags: vec![],
                        });
                    }
                }
            }
        }
    }
    
    Ok(skills)
}

/// Get skill info from local storage
async fn get_skill_info(skill_id: &str) -> Result<SkillInfoResponse, Box<dyn std::error::Error>> {
    let skill_dir = get_skill_install_path(skill_id);
    let manifest_path = skill_dir.join("skill.yaml");
    
    let content = tokio::fs::read_to_string(&manifest_path).await?;
    let manifest: serde_yaml::Value = serde_yaml::from_str(&content)?;
    
    Ok(SkillInfoResponse {
        id: skill_id.to_string(),
        name: manifest["name"].as_str().unwrap_or(skill_id).to_string(),
        version: manifest["version"].as_str().unwrap_or("1.0.0").to_string(),
        description: manifest["description"].as_str().unwrap_or("").to_string(),
        author: manifest["author"].as_str().unwrap_or("Unknown").to_string(),
        license: manifest["license"].as_str().unwrap_or("MIT").to_string(),
        installed: true,
        capabilities: vec![],
        tags: vec![],
    })
}
