use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub repository: Option<String>,
    pub hash: String,
    pub downloads: u64,
    pub rating: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PublishRequest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub license: String,
    pub repository: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublishResponse {
    pub id: String,
    pub message: String,
}
