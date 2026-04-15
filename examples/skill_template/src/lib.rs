//! BeeBotOS Skill Template
//! 
//! This is a template for creating custom skills for BeeBotOS agents.

use beebot_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Skill configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub timeout_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            timeout_seconds: 30,
        }
    }
}

/// Skill state
pub struct Skill {
    config: Config,
    http_client: HttpClient,
}

impl Skill {
    /// Create new skill instance
    pub fn new(config: Config) -> Result<Self, SkillError> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;
        
        Ok(Self {
            config,
            http_client,
        })
    }

    /// Handle incoming message
    pub async fn handle_message(&self, ctx: &Context, msg: Message) -> Result<Message, SkillError> {
        match msg.action.as_str() {
            "fetch_data" => self.fetch_data(ctx, msg).await,
            "process" => self.process(ctx, msg).await,
            _ => Err(SkillError::UnknownAction(msg.action)),
        }
    }

    /// Fetch data from external API
    async fn fetch_data(&self, ctx: &Context, msg: Message) -> Result<Message, SkillError> {
        ctx.log("Fetching data...").await?;
        
        let url = msg.params.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillError::MissingParam("url"))?;
        
        let response = self.http_client
            .get(url)
            .header("Authorization", &format!("Bearer {}", self.config.api_key))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        
        Ok(Message::response(&msg, serde_json::json!({
            "status": "success",
            "data": data,
        })))
    }

    /// Process data
    async fn process(&self, ctx: &Context, msg: Message) -> Result<Message, SkillError> {
        ctx.log("Processing data...").await?;
        
        let input = msg.params.get("input")
            .ok_or_else(|| SkillError::MissingParam("input"))?;
        
        // Process the input
        let result = self.transform_data(input).await?;
        
        Ok(Message::response(&msg, serde_json::json!({
            "status": "success",
            "result": result,
        })))
    }

    /// Transform data
    async fn transform_data(&self, input: &serde_json::Value) -> Result<serde_json::Value, SkillError> {
        // Implement your transformation logic here
        Ok(input.clone())
    }
}

// Register the skill
beebot_sdk::export_skill!(Skill);
