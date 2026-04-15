//! Agent Resolver - Maps channels/users to agents
//!
//! Provides resolution logic to determine which agent should handle
//! an incoming channel message.

use std::sync::Arc;

use tracing::{error, info, warn};

use crate::error::GatewayError;

/// Resolves a channel message to an agent ID
pub struct AgentResolver {
    /// Optional default agent ID from configuration
    default_agent_id: Option<String>,
    /// State store for querying registered agents
    state_store: Arc<gateway::StateStore>,
    /// Agent runtime for creating fallback agents if needed
    agent_runtime: Arc<dyn gateway::AgentRuntime>,
    /// Channel-to-agent binding store
    channel_binding_store: Option<Arc<gateway::ChannelBindingStore>>,
}

impl AgentResolver {
    /// Create a new agent resolver
    pub fn new(
        default_agent_id: Option<String>,
        state_store: Arc<gateway::StateStore>,
        agent_runtime: Arc<dyn gateway::AgentRuntime>,
    ) -> Self {
        Self {
            default_agent_id,
            state_store,
            agent_runtime,
            channel_binding_store: None,
        }
    }

    /// Set the channel binding store
    pub fn with_channel_binding_store(
        mut self,
        store: Arc<gateway::ChannelBindingStore>,
    ) -> Self {
        self.channel_binding_store = Some(store);
        self
    }

    /// Resolve the target agent ID for a channel message
    ///
    /// Resolution order:
    /// 1. ChannelBindingStore binding for (platform, channel_id)
    /// 2. Configured default_agent_id (if valid and running)
    /// 3. First available agent from StateStore
    /// 4. Error if no agent can be found
    pub async fn resolve(
        &self,
        platform: beebotos_agents::communication::PlatformType,
        channel_id: &str,
        _user_id: &str,
    ) -> Result<String, GatewayError> {
        let platform_str = platform.to_string();

        // 1. Try channel-agent binding store
        if let Some(ref binding_store) = self.channel_binding_store {
            if let Some(agent_id) = binding_store.resolve_agent(&platform_str, channel_id).await {
                match self.agent_runtime.status(&agent_id).await {
                    Ok(status) => {
                        if status.state != gateway::AgentState::Stopped
                            && status.state != gateway::AgentState::Error
                        {
                            info!(
                                "Resolved agent {} from ChannelBindingStore ({}:{})",
                                agent_id, platform_str, channel_id
                            );
                            return Ok(agent_id);
                        }
                        warn!(
                            "Bound agent {} for {}:{} is in state {:?}, skipping",
                            agent_id, platform_str, channel_id, status.state
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Bound agent {} for {}:{} not found: {}",
                            agent_id, platform_str, channel_id, e
                        );
                    }
                }
            }
        }

        // 2. Try configured default agent
        if let Some(ref agent_id) = self.default_agent_id {
            match self.agent_runtime.status(agent_id).await {
                Ok(status) => {
                    if status.state != gateway::AgentState::Stopped
                        && status.state != gateway::AgentState::Error
                    {
                        info!("Resolved agent {} from default_agent_id config", agent_id);
                        return Ok(agent_id.clone());
                    }
                    warn!(
                        "Configured default_agent_id {} is in state {:?}, skipping",
                        agent_id, status.state
                    );
                }
                Err(e) => {
                    warn!("Configured default_agent_id {} not found: {}", agent_id, e);
                }
            }
        }

        // 2. Query StateStore for the first available agent
        let query_result = self
            .state_store
            .query(gateway::StateQuery::ListAgents {
                filter: Some(gateway::AgentFilter {
                    state: None,
                    has_capability: None,
                    created_after: None,
                    created_before: None,
                }),
                limit: 100,
                offset: 0,
            })
            .await
            .map_err(|e| GatewayError::Internal {
                message: format!("Failed to list agents from state store: {}", e),
                correlation_id: uuid::Uuid::new_v4().to_string(),
            })?;

        if let gateway::QueryResult::AgentList { agents, .. } = query_result {
            for agent_info in agents {
                if agent_info.current_state != gateway::AgentState::Stopped
                    && agent_info.current_state != gateway::AgentState::Error
                {
                    info!(
                        "Resolved agent {} from StateStore (first available)",
                        agent_info.agent_id
                    );
                    return Ok(agent_info.agent_id);
                }
            }
        }

        // 3. Auto-create a default agent
        let agent_id = format!("auto-agent-{}-{}", platform_str, channel_id);
        let agent_name = format!("Auto Agent {} {}", platform_str, channel_id);
        let llm_config = gateway::LlmConfig {
            provider: "kimi".to_string(),
            model: "kimi-k2.5".to_string(),
            api_key: None,
            temperature: 0.7,
            max_tokens: 2000,
        };
        let agent_config = gateway::AgentConfigBuilder::new(&agent_id, &agent_name)
            .description("Auto-created default agent for incoming messages")
            .with_llm(llm_config)
            .build();

        info!("🆕 No available agent found, auto-creating default agent {}", agent_id);
        self.agent_runtime.spawn(agent_config).await.map_err(|e| {
            error!("❌ Failed to auto-create default agent {}: {}", agent_id, e);
            GatewayError::Internal {
                message: format!("Failed to auto-create default agent: {}", e),
                correlation_id: uuid::Uuid::new_v4().to_string(),
            }
        })?;

        // Bind to channel if binding store exists
        if let Some(ref binding_store) = self.channel_binding_store {
            if let Err(e) = binding_store.bind(&platform_str, channel_id, &agent_id).await {
                warn!("Failed to bind auto-created agent {} to {}:{}: {}", agent_id, platform_str, channel_id, e);
            } else {
                info!("Bound auto-created agent {} to {}:{}", agent_id, platform_str, channel_id);
            }
        }

        Ok(agent_id)
    }
}
