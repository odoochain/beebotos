use anyhow::{Result, Context, anyhow};
use reqwest::{header, Request, Response};
use std::time::Duration;
use futures::Stream;
use std::pin::Pin;

use crate::network::{NetworkClient, NetworkConfig, DefaultRequestInterceptor, LoggingInterceptor, RetryPolicy};
use crate::error::CliError;

pub struct ApiClient {
    network: NetworkClient,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    /// Create new API client with default configuration
    pub fn new() -> Result<Self> {
        let base_url = std::env::var("BEEBOTOS_API_URL")
            .unwrap_or_else(|_| "https://api.beebotos.io/v1".to_string());
        
        let api_key = std::env::var("BEEBOTOS_API_KEY")
            .map_err(|_| anyhow!("BEEBOTOS_API_KEY not set"))?;
        
        Self::with_config(base_url, api_key)
    }
    
    /// Create client with custom base URL and API key
    pub fn with_config(base_url: String, api_key: String) -> Result<Self> {
        let mut config = NetworkConfig::default();
        
        // Load custom configuration from environment
        if let Ok(timeout) = std::env::var("BEEBOTOS_HTTP_TIMEOUT") {
            if let Ok(secs) = timeout.parse() {
                config.timeout = Duration::from_secs(secs);
            }
        }
        
        if let Ok(connect_timeout) = std::env::var("BEEBOTOS_HTTP_CONNECT_TIMEOUT") {
            if let Ok(secs) = connect_timeout.parse() {
                config.connect_timeout = Duration::from_secs(secs);
            }
        }
        
        // Enable proxy from environment
        config.proxy = crate::network::ProxyConfig::from_env();
        
        // Build network client
        let mut network = NetworkClient::new(config)?;
        
        // Add interceptors
        network.add_request_interceptor(DefaultRequestInterceptor::new(
            api_key.clone(),
            format!("BeeBotOS-CLI/{}", env!("CARGO_PKG_VERSION")),
        ));
        network.add_request_interceptor(LoggingInterceptor);
        network.add_response_interceptor(LoggingInterceptor);
        
        // Configure retry policy
        let retry_policy = RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            retryable_status_codes: vec![408, 429, 500, 502, 503, 504],
        };
        network.set_retry_policy(retry_policy);
        
        Ok(Self { network, base_url, api_key })
    }
    
    /// Create client with custom network configuration
    pub fn with_network_config(base_url: String, api_key: String, config: NetworkConfig) -> Result<Self> {
        let mut network = NetworkClient::new(config)?;
        
        network.add_request_interceptor(DefaultRequestInterceptor::new(
            api_key.clone(),
            format!("BeeBotOS-CLI/{}", env!("CARGO_PKG_VERSION")),
        ));
        
        Ok(Self { network, base_url, api_key })
    }
    
    /// Get the underlying network client
    pub fn network(&self) -> &NetworkClient {
        &self.network
    }
    
    /// Execute a request with retry logic
    async fn execute_request(&self, request: Request) -> Result<Response> {
        self.network.execute(request).await
            .map_err(|e| anyhow!("Request failed: {}", e))
    }
    
    /// Build request URL
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
    
    // Agent operations
    pub async fn create_agent(&self, name: &str, template: &str, config: Option<&str>) -> Result<AgentInfo> {
        let url = self.build_url("/agents");
        let body = serde_json::json!({
            "name": name,
            "template": template,
            "config": config,
        });
        
        let request = self.network.inner()
            .post(&url)
            .json(&body)
            .build()?;
        
        let resp = self.execute_request(request).await?;
        
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(CliError::Api {
                status,
                message: format!("Failed to create agent: {}", text),
            }.into());
        }
        
        Ok(resp.json().await?)
    }
    
    pub async fn list_agents(&self, status: Option<&str>, all: bool) -> Result<Vec<AgentInfo>> {
        let mut url = self.build_url("/agents");
        if let Some(s) = status {
            url.push_str(&format!("?status={}", s));
        }
        if all {
            url.push_str(&if status.is_some() { "&" } else { "?" });
            url.push_str("all=true");
        }
        
        let request = self.network.inner()
            .get(&url)
            .build()?;
        
        let resp = self.execute_request(request).await?;
        
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(CliError::Api {
                status,
                message: format!("Failed to list agents: {}", text),
            }.into());
        }
        
        let result: serde_json::Value = resp.json().await?;
        Ok(serde_json::from_value(result["data"].clone())?)
    }
    
    pub async fn get_agent(&self, id: &str) -> Result<AgentInfo> {
        let url = format!("{}/agents/{}", self.base_url, id);
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get agent ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    pub async fn start_agent(&self, id: &str) -> Result<()> {
        let url = format!("{}/agents/{}/start", self.base_url, id);
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to start agent ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    pub async fn stop_agent(&self, id: &str, force: bool) -> Result<()> {
        let url = format!("{}/agents/{}/stop?force={}", self.base_url, id, force);
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to stop agent ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    pub async fn delete_agent(&self, id: &str) -> Result<()> {
        let url = format!("{}/agents/{}", self.base_url, id);
        let resp = self.http
            .delete(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to delete agent ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    pub async fn exec_task(&self, agent_id: &str, input: &str, timeout: u64) -> Result<TaskResult> {
        let url = format!("{}/agents/{}/tasks", self.base_url, agent_id);
        let body = serde_json::json!({
            "input": input,
            "timeout": timeout,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to execute task ({}): {}", status, text));
        }
        
        let task: TaskInfo = resp.json().await?;
        
        // Poll for result
        let result_url = format!("{}/agents/{}/tasks/{}", self.base_url, agent_id, task.id);
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            let resp = self.http
                .get(&result_url)
                .headers(self.headers()?)
                .send()
                .await?;
            
            if resp.status().is_success() {
                let task: TaskInfo = resp.json().await?;
                if task.status == "completed" {
                    return Ok(task.result.ok_or_else(|| anyhow!("Task completed but no result"))?);
                } else if task.status == "failed" {
                    return Err(anyhow!("Task failed: {:?}", task.error));
                }
            }
        }
    }
    
    /// Follow logs for an agent (streaming)
    pub async fn follow_logs(&self, id: &str) -> Result<()> {
        let url = format!("{}/agents/{}/logs/stream", self.base_url, id);
        let mut resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to follow logs ({}): {}", status, text));
        }
        
        // Stream logs to stdout
        while let Some(chunk) = resp.chunk().await? {
            let text = String::from_utf8_lossy(&chunk);
            print!("{}", text);
        }
        
        Ok(())
    }
    
    /// Get logs for an agent
    pub async fn get_logs(&self, id: &str, lines: usize) -> Result<Vec<String>> {
        let url = format!("{}/agents/{}/logs?lines={}", self.base_url, id, lines);
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get logs ({}): {}", status, text));
        }
        
        let result: serde_json::Value = resp.json().await?;
        Ok(serde_json::from_value(result["logs"].clone())?)
    }
    
    // Message operations
    pub async fn send_message(&self, to: &str, message: &str, timeout: u64) -> Result<String> {
        let url = format!("{}/messages", self.base_url);
        let body = serde_json::json!({
            "to": to,
            "content": message,
            "timeout": timeout,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to send message ({}): {}", status, text));
        }
        
        let result: serde_json::Value = resp.json().await?;
        result["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Invalid response: missing 'content' field"))
    }
    
    /// Broadcast a message to multiple agents
    pub async fn broadcast_message(&self, capability: Option<&str>, message: &str) -> Result<Vec<String>> {
        let url = format!("{}/messages/broadcast", self.base_url);
        let body = serde_json::json!({
            "capability": capability,
            "content": message,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to broadcast message ({}): {}", status, text));
        }
        
        let result: serde_json::Value = resp.json().await?;
        Ok(serde_json::from_value(result["recipients"].clone())?)
    }
    
    /// Get message history
    pub async fn get_message_history(&self, agent: Option<&str>, limit: usize) -> Result<Vec<MessageInfo>> {
        let mut url = format!("{}/messages?limit={}", self.base_url, limit);
        if let Some(agent_id) = agent {
            url.push_str(&format!("&agent={}", agent_id));
        }
        
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get message history ({}): {}", status, text));
        }
        
        let result: serde_json::Value = resp.json().await?;
        Ok(serde_json::from_value(result["messages"].clone())?)
    }
    
    // Skill operations
    pub async fn list_skills(&self, category: Option<&str>, search: Option<&str>) -> Result<Vec<SkillInfo>> {
        let mut url = format!("{}/skills", self.base_url);
        if let Some(c) = category {
            url.push_str(&format!("?category={}", c));
        }
        if let Some(s) = search {
            url.push_str(&format!("&search={}", s));
        }
        
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to list skills ({}): {}", status, text));
        }
        
        let result: serde_json::Value = resp.json().await?;
        Ok(serde_json::from_value(result["data"].clone())?)
    }
    
    pub async fn get_skill(&self, id: &str) -> Result<SkillInfo> {
        let url = format!("{}/skills/{}", self.base_url, id);
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get skill ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    pub async fn install_skill(&self, source: &str, agent: Option<&str>, version: Option<&str>) -> Result<SkillInfo> {
        let url = format!("{}/skills/install", self.base_url);
        let body = serde_json::json!({
            "source": source,
            "agent_id": agent,
            "version": version,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to install skill ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    /// Uninstall a skill
    pub async fn uninstall_skill(&self, id: &str, agent: Option<&str>) -> Result<()> {
        let url = format!("{}/skills/{}/uninstall", self.base_url, id);
        let body = serde_json::json!({
            "agent_id": agent,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to uninstall skill ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    /// Update a skill
    pub async fn update_skill(&self, id: &str, agent: Option<&str>) -> Result<SkillInfo> {
        let url = format!("{}/skills/{}/update", self.base_url, id);
        let body = serde_json::json!({
            "agent_id": agent,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to update skill ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    /// Create skill template
    pub async fn create_skill_template(&self, name: &str, template: &str, output: &str) -> Result<()> {
        let url = format!("{}/skills/templates", self.base_url);
        let body = serde_json::json!({
            "name": name,
            "template": template,
            "output": output,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create skill template ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    /// Publish a skill to registry
    pub async fn publish_skill(&self, path: &str, registry: &str) -> Result<PublishResult> {
        let url = format!("{}/skills/publish", self.base_url);
        let body = serde_json::json!({
            "path": path,
            "registry": registry,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to publish skill ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    // DAO operations
    #[allow(dead_code)]
    pub async fn list_proposals(&self, status: Option<&str>) -> Result<Vec<ProposalInfo>> {
        let mut url = format!("{}/dao/proposals", self.base_url);
        if let Some(s) = status {
            url.push_str(&format!("?status={}", s));
        }
        
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to list proposals ({}): {}", status, text));
        }
        
        let result: serde_json::Value = resp.json().await?;
        Ok(serde_json::from_value(result["data"].clone())?)
    }
    
    #[allow(dead_code)]
    pub async fn cast_vote(&self, proposal_id: u64, support: &str, reason: Option<String>) -> Result<()> {
        let url = format!("{}/dao/proposals/{}/votes", self.base_url, proposal_id);
        let body = serde_json::json!({
            "support": support,
            "reason": reason,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to cast vote ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    // Brain operations
    /// Get brain status
    pub async fn get_brain_status(&self) -> Result<BrainStatus> {
        let url = format!("{}/brain/status", self.base_url);
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get brain status ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    /// Store a memory
    pub async fn store_memory(&self, agent: &str, content: &str, memory_type: &str, importance: f32) -> Result<()> {
        let url = format!("{}/brain/agents/{}/memories", self.base_url, agent);
        let body = serde_json::json!({
            "content": content,
            "memory_type": memory_type,
            "importance": importance,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to store memory ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    /// Retrieve memories
    pub async fn retrieve_memories(&self, agent: &str, query: &str, limit: usize) -> Result<Vec<MemoryInfo>> {
        let url = format!("{}/brain/agents/{}/memories?query={}&limit={}", self.base_url, agent, query, limit);
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to retrieve memories ({}): {}", status, text));
        }
        
        let result: serde_json::Value = resp.json().await?;
        Ok(serde_json::from_value(result["memories"].clone())?)
    }
    
    /// Consolidate memories
    pub async fn consolidate_memories(&self, agent: &str) -> Result<()> {
        let url = format!("{}/brain/agents/{}/consolidate", self.base_url, agent);
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to consolidate memories ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    /// Get emotion state
    pub async fn get_emotion_state(&self, agent: &str) -> Result<EmotionState> {
        let url = format!("{}/brain/agents/{}/emotion", self.base_url, agent);
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get emotion state ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    /// Set emotion state
    pub async fn set_emotion_state(&self, agent: &str, pleasure: f32, arousal: f32, dominance: f32) -> Result<()> {
        let url = format!("{}/brain/agents/{}/emotion", self.base_url, agent);
        let body = serde_json::json!({
            "pleasure": pleasure,
            "arousal": arousal,
            "dominance": dominance,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to set emotion state ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    /// Evolve agent
    pub async fn evolve_agent(&self, agent: &str, generations: u32) -> Result<EvolutionResult> {
        let url = format!("{}/brain/agents/{}/evolve", self.base_url, agent);
        let body = serde_json::json!({
            "generations": generations,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to evolve agent ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    // Session operations
    /// Create a session
    pub async fn create_session(&self, agent: &str, name: Option<&str>) -> Result<SessionInfo> {
        let url = format!("{}/sessions", self.base_url);
        let body = serde_json::json!({
            "agent_id": agent,
            "name": name,
        });
        
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create session ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    /// List sessions
    pub async fn list_sessions(&self, agent: Option<&str>, active: bool) -> Result<Vec<SessionInfo>> {
        let mut url = format!("{}/sessions", self.base_url);
        if let Some(agent_id) = agent {
            url.push_str(&format!("?agent={}", agent_id));
        }
        if active {
            url.push_str("&active=true");
        }
        
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to list sessions ({}): {}", status, text));
        }
        
        let result: serde_json::Value = resp.json().await?;
        Ok(serde_json::from_value(result["sessions"].clone())?)
    }
    
    /// Resume a session
    pub async fn resume_session(&self, id: &str) -> Result<SessionInfo> {
        let url = format!("{}/sessions/{}/resume", self.base_url, id);
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to resume session ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    /// Get session details
    pub async fn get_session(&self, id: &str) -> Result<SessionDetail> {
        let url = format!("{}/sessions/{}", self.base_url, id);
        let resp = self.http
            .get(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to get session ({}): {}", status, text));
        }
        
        Ok(resp.json().await?)
    }
    
    /// Archive a session
    pub async fn archive_session(&self, id: &str) -> Result<()> {
        let url = format!("{}/sessions/{}/archive", self.base_url, id);
        let resp = self.http
            .post(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to archive session ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    /// Delete a session
    pub async fn delete_session(&self, id: &str) -> Result<()> {
        let url = format!("{}/sessions/{}", self.base_url, id);
        let resp = self.http
            .delete(&url)
            .headers(self.headers()?)
            .send()
            .await?;
        
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to delete session ({}): {}", status, text));
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    /// Convert HTTP URL to WebSocket URL
    fn http_to_ws_url(http_url: &str) -> Result<String> {
        let ws_url = if http_url.starts_with("https://") {
            http_url.replace("https://", "wss://")
        } else if http_url.starts_with("http://") {
            http_url.replace("http://", "ws://")
        } else {
            http_url.to_string()
        };
        
        // Ensure path ends with /ws
        let ws_url = if ws_url.ends_with("/ws") {
            ws_url
        } else if ws_url.ends_with('/') {
            format!("{}ws", ws_url)
        } else {
            format!("{}/ws", ws_url)
        };
        
        Ok(ws_url)
    }
}

// ChainClient for blockchain operations
pub struct ChainClient;

impl ChainClient {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
    
    pub fn default_address(&self) -> String {
        "0x0000000000000000000000000000000000000000".to_string()
    }
    
    pub async fn get_status(&self) -> Result<ChainStatus> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn get_balance(&self, _address: &str, _token: Option<&str>) -> Result<String> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn transfer(&self, _to: &str, _amount: &str, _token: Option<&str>) -> Result<String> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn wait_for_confirmation(&self, _tx_hash: &str) -> Result<TransactionReceipt> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn deploy_contract(&self, _artifact: &str, _args: &[String]) -> Result<String> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn verify_contract(&self, _address: &str, _artifact: &str) -> Result<()> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn call(&self, _contract: &str, _function: &str, _args: &[String]) -> Result<String> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn send_transaction(&self, _contract: &str, _function: &str, _args: &[String]) -> Result<String> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn watch_events(
        &self, 
        _contract: Option<&str>, 
        _event: Option<&str>, 
        _from_block: Option<u64>
    ) -> Result<Pin<Box<dyn Stream<Item = EventInfo> + Send>>> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
        #[allow(dead_code)]
    pub async fn watch_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = BlockInfo> + Send>>> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    // Payment operations
    pub async fn store_payment_metadata(&self, _tx_hash: &str, _desc: &str) -> Result<()> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn create_mandate(&self, _grantee: &str, _allowance: &str, _token: Option<&str>, _duration: u32) -> Result<MandateInfo> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn list_mandates_as_grantor(&self) -> Result<Vec<MandateInfo>> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn list_mandates_as_grantee(&self) -> Result<Vec<MandateInfo>> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn list_all_mandates(&self) -> Result<Vec<MandateInfo>> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn revoke_mandate(&self, _id: &str) -> Result<()> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
    
    pub async fn get_transactions(&self, _token: Option<&str>, _limit: usize) -> Result<Vec<TransactionInfo>> {
        Err(anyhow!("Chain operations not yet implemented. Enable 'chain' feature for full functionality."))
    }
}

// Data structures
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub last_active: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub status: String,
    pub result: Option<TaskResult>,
    pub error: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TaskResult {
    pub output: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SkillInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub category: String,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub downloads: u64,
    pub rating: f32,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ProposalInfo {
    pub id: u64,
    pub title: String,
    pub status: String,
    pub votes_for: String,
    pub votes_against: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BrainStatus {
    pub memory_used: String,
    pub memory_total: String,
    pub active_agents: usize,
    pub evolution_queue: usize,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MemoryInfo {
    pub memory_type: String,
    pub content: String,
    pub relevance: f32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct EmotionState {
    pub pleasure: f32,
    pub arousal: f32,
    pub dominance: f32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct EvolutionResult {
    pub fitness: f32,
    pub generations: u32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub last_active: String,
    pub key: String,
    pub agent_id: String,
    #[serde(default)]
    pub context_items: usize,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SessionDetail {
    pub id: String,
    pub name: String,
    pub agent_id: String,
    pub status: String,
    pub created_at: String,
    pub context_items: usize,
    pub transcript: Vec<TranscriptEntry>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TranscriptEntry {
    pub timestamp: String,
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct PublishResult {
    pub id: String,
    pub version: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MessageInfo {
    pub timestamp: String,
    pub from: String,
    pub to: Option<String>,
    pub content: String,
}

// Re-export types from websocket module
#[allow(unused_imports)]
pub use crate::websocket::{AgentUpdate, BlockInfo, EventInfo, TaskUpdate};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ChainStatus {
    pub network: String,
    pub chain_id: u64,
    pub block_number: u64,
    pub sync_status: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TransactionReceipt {
    pub block_number: u64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MandateInfo {
    pub id: String,
    pub grantor: String,
    pub grantee: String,
    pub remaining: String,
    pub active: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub amount: String,
    pub token: String,
    pub status: String,
}
    pub active: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub amount: String,
    pub token: String,
    pub status: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_info_serialization() {
        let agent = AgentInfo {
            id: "agent-123".to_string(),
            name: "Test Agent".to_string(),
            status: "running".to_string(),
            last_active: "2024-01-15T10:30:00Z".to_string(),
        };
        let json = serde_json::to_string(&agent).unwrap();
        assert!(json.contains("agent-123"));
        assert!(json.contains("Test Agent"));
    }

    #[test]
    fn test_agent_info_deserialization() {
        let json = r#"{
            "id": "agent-456",
            "name": "My Agent",
            "status": "idle",
            "last_active": "2024-01-15T10:30:00Z"
        }"#;
        let agent: AgentInfo = serde_json::from_str(json).unwrap();
        assert_eq!(agent.id, "agent-456");
        assert_eq!(agent.status, "idle");
    }

    #[test]
    fn test_task_result_serialization() {
        let result = TaskResult {
            output: "Task completed successfully".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Task completed successfully"));
    }

    #[test]
    fn test_skill_info_serialization() {
        let skill = SkillInfo {
            id: "skill-123".to_string(),
            name: "Code Analyzer".to_string(),
            version: "1.0.0".to_string(),
            category: "development".to_string(),
            author: "BeeBotOS Team".to_string(),
            description: "Analyzes code for issues".to_string(),
            capabilities: vec!["analyze".to_string(), "review".to_string()],
            downloads: 1000,
            rating: 4.5,
        };
        let json = serde_json::to_string(&skill).unwrap();
        assert!(json.contains("Code Analyzer"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_proposal_info_serialization() {
        let proposal = ProposalInfo {
            id: 123,
            title: "Add new feature".to_string(),
            status: "active".to_string(),
            votes_for: "1000".to_string(),
            votes_against: "100".to_string(),
        };
        let json = serde_json::to_string(&proposal).unwrap();
        assert!(json.contains("Add new feature"));
    }

    #[test]
    fn test_chain_status_serialization() {
        let status = ChainStatus {
            network: "mainnet".to_string(),
            chain_id: 1,
            block_number: 12345678,
            sync_status: "synced".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("mainnet"));
        assert!(json.contains("12345678"));
    }

    #[test]
    fn test_session_info_serialization() {
        let session = SessionInfo {
            id: "session-123".to_string(),
            name: "My Session".to_string(),
            status: "active".to_string(),
            last_active: "2024-01-15T10:30:00Z".to_string(),
            key: "secret-key".to_string(),
            agent_id: "agent-456".to_string(),
            context_items: 5,
        };
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("session-123"));
        assert!(json.contains("agent-456"));
    }

    #[test]
    fn test_transaction_info_serialization() {
        let tx = TransactionInfo {
            hash: "0xabc123".to_string(),
            amount: "1.5".to_string(),
            token: "BEE".to_string(),
            status: "confirmed".to_string(),
        };
        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains("0xabc123"));
        assert!(json.contains("confirmed"));
    }

    #[test]
    fn test_emotion_state_serialization() {
        let emotion = EmotionState {
            pleasure: 0.5,
            arousal: 0.3,
            dominance: 0.4,
        };
        let json = serde_json::to_string(&emotion).unwrap();
        assert!(json.contains("0.5"));
    }

    #[test]
    fn test_evolution_result_serialization() {
        let result = EvolutionResult {
            fitness: 0.95,
            generations: 100,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("0.95"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_memory_info_serialization() {
        let memory = MemoryInfo {
            memory_type: "episodic".to_string(),
            content: "Remember this event".to_string(),
            relevance: 0.85,
        };
        let json = serde_json::to_string(&memory).unwrap();
        assert!(json.contains("episodic"));
    }

    #[test]
    fn test_message_info_serialization() {
        let msg = MessageInfo {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            from: "agent-1".to_string(),
            to: Some("agent-2".to_string()),
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_mandate_info_serialization() {
        let mandate = MandateInfo {
            id: "mandate-123".to_string(),
            grantor: "0xabc".to_string(),
            grantee: "0xdef".to_string(),
            remaining: "1000".to_string(),
            active: true,
        };
        let json = serde_json::to_string(&mandate).unwrap();
        assert!(json.contains("mandate-123"));
    }

    #[test]
    fn test_brain_status_serialization() {
        let status = BrainStatus {
            memory_used: "50MB".to_string(),
            memory_total: "100MB".to_string(),
            active_agents: 10,
            evolution_queue: 2,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("50MB"));
    }

    #[test]
    fn test_publish_result_serialization() {
        let result = PublishResult {
            id: "skill-123".to_string(),
            version: "1.0.0".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("skill-123"));
    }

    #[test]
    fn test_transcript_entry_serialization() {
        let entry = TranscriptEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("user"));
    }
}
