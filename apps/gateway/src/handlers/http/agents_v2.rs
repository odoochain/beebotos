//! Agent HTTP Handlers (V2 - Using new AgentRuntime trait)
//!
//! 🟢 P1 FIX: Migrated to use AgentRuntime trait and StateStore (CQRS).
//! This version is decoupled from the concrete beebotos_agents implementation.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use gateway::{
    error::GatewayError,
    middleware::{require_any_role, AuthUser},
    AgentConfigBuilder,
    LlmConfig, MemoryConfig,
    StateQuery, StateCommand,
    QueryResult, AgentState,
    TaskConfig, AgentStateCommand,
};
use serde::Deserialize;
use serde_json::json;

use crate::models::{AgentResponse, CreateAgentRequest, PaginatedResponse, PaginationParams, ModelInfo};
use crate::AppState;

/// List all agents with pagination (V2)
///
/// Uses StateStore (CQRS) for efficient querying.
pub async fn list_agents_v2(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    axum::extract::Query(params): axum::extract::Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<AgentResponse>>, GatewayError> {
    require_any_role(&user, &["user", "admin"])?;

    // 🟢 P1 FIX: Use StateStore (CQRS) for querying
    let query_result = state
        .state_store
        .query(StateQuery::ListAgents {
            filter: None,
            limit: params.per_page as usize,
            offset: params.offset() as usize,
        })
        .await
        .map_err(|e| GatewayError::agent(format!("Failed to list agents: {}", e)))?;

    let (agents, total) = match query_result {
        QueryResult::AgentList { agents, total, .. } => (agents, total),
        _ => return Err(GatewayError::internal("Unexpected query result")),
    };

    // Convert to response models
    let responses: Vec<AgentResponse> = agents
        .into_iter()
        .map(|info| AgentResponse {
            id: info.agent_id,
            name: info.config.name,
            description: Some(info.config.description),
            status: info.current_state.to_string(),
            capabilities: info.config.capabilities.into_iter().map(|c| c.name).collect(),
            model: ModelInfo {
                provider: info.config.llm_config.provider,
                name: info.config.llm_config.model,
            },
            created_at: info.created_at,
            updated_at: info.updated_at,
            last_heartbeat: None,
        })
        .collect();

    Ok(Json(PaginatedResponse::new(
        responses,
        total as i64,
        params.page,
        params.per_page,
    )))
}

/// Create new agent (V2)
///
/// Uses AgentRuntime trait for decoupled agent lifecycle management.
pub async fn create_agent_v2(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<CreateAgentRequest>,
) -> Result<impl IntoResponse, GatewayError> {
    require_any_role(&user, &["user", "admin"])?;

    // Validate input
    if req.name.is_empty() {
        return Err(GatewayError::validation(vec![
            gateway::error::ValidationError {
                field: "name".to_string(),
                message: "Name is required".to_string(),
                code: "required".to_string(),
            },
        ]));
    }

    // 🟢 P1 FIX: Build GatewayAgentConfig using builder pattern
    let agent_config = AgentConfigBuilder::new(
        uuid::Uuid::new_v4().to_string(),
        &req.name
    )
    .description(&req.description.unwrap_or_default())
    .with_llm(LlmConfig {
        provider: req.model_provider.unwrap_or_else(|| "openai".to_string()),
        model: req.model_name.unwrap_or_else(|| "gpt-4".to_string()),
        api_key: None,
        temperature: 0.7,
        max_tokens: 2000,
    })
    .with_memory(MemoryConfig {
        memory_type: "local".to_string(),
        storage_path: "data/memory".to_string(),
        max_entries: 10000,
    })
    .build();

    // 🟢 P1 FIX: Use AgentRuntime trait to spawn agent
    let handle = state
        .agent_runtime
        .spawn(agent_config.clone())
        .await
        .map_err(|e| GatewayError::agent(format!("Failed to spawn agent: {}", e)))?;

    // 🟢 P1 FIX: Register in StateStore for persistence
    state
        .state_store
        .execute(StateCommand::RegisterAgent {
            agent_id: handle.agent_id.clone(),
            config: agent_config.clone(),
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("owner_id".to_string(), user.user_id.clone());
                meta.insert("created_by".to_string(), user.user_id.clone());
                meta
            },
        })
        .await
        .map_err(|e| GatewayError::state(format!("Failed to register agent: {}", e)))?;

    tracing::info!(
        agent_id = %handle.agent_id,
        user_id = %user.user_id,
        "Agent created via AgentRuntime trait"
    );

    let response = AgentResponse {
        id: handle.agent_id,
        name: agent_config.name,
        description: Some(agent_config.description),
        status: "registered".to_string(),
        capabilities: agent_config.capabilities.into_iter().map(|c| c.name).collect(),
        model: ModelInfo {
            provider: agent_config.llm_config.provider,
            name: agent_config.llm_config.model,
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_heartbeat: None,
    };

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "agent": response,
            "message": "Agent created successfully via AgentRuntime",
        })),
    ))
}

/// Get agent by ID (V2)
pub async fn get_agent_v2(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, GatewayError> {
    // 🟢 P1 FIX: Use StateStore to query agent info
    let query_result = state
        .state_store
        .query(StateQuery::GetAgentInfo { agent_id: id.clone() })
        .await
        .map_err(|e| GatewayError::agent(format!("Failed to get agent: {}", e)))?;

    let info = match query_result {
        QueryResult::AgentInfo { .. } => {
            // TODO: Check ownership from metadata
            query_result
        }
        _ => return Err(GatewayError::not_found("Agent", &id)),
    };

    Ok(Json(json!({
        "agent": info,
        "version": "v2 (AgentRuntime)",
    })))
}

/// Delete agent (V2)
pub async fn delete_agent_v2(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, GatewayError> {
    // 🟢 P1 FIX: Use AgentRuntime trait to stop agent
    state
        .agent_runtime
        .stop(&id)
        .await
        .map_err(|e| GatewayError::agent(format!("Failed to stop agent: {}", e)))?;

    // 🟢 P1 FIX: Archive in StateStore
    state
        .state_store
        .execute(StateCommand::ArchiveAgent { agent_id: id.clone() })
        .await
        .map_err(|e| GatewayError::state(format!("Failed to archive agent: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Agent deleted successfully",
            "agent_id": id,
        })),
    ))
}

/// Start agent (V2)
pub async fn start_agent_v2(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, GatewayError> {
    // 🟢 P1 FIX: Use AgentRuntime trait
    state
        .agent_runtime
        .send_command(&id, AgentStateCommand::Start)
        .await
        .map_err(|e| GatewayError::agent(format!("Failed to start agent: {}", e)))?;

    // 🟢 P1 FIX: Record state transition in StateStore
    state
        .state_store
        .execute(StateCommand::Transition {
            agent_id: id.clone(),
            from: AgentState::Registered,
            to: AgentState::Working,
            reason: Some("User requested start".to_string()),
        })
        .await
        .map_err(|e| GatewayError::state(format!("Failed to record transition: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Agent start command sent",
            "agent_id": id,
        })),
    ))
}

/// Stop agent (V2)
pub async fn stop_agent_v2(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, GatewayError> {
    // 🟢 P1 FIX: Use AgentRuntime trait
    state
        .agent_runtime
        .send_command(&id, AgentStateCommand::Stop)
        .await
        .map_err(|e| GatewayError::agent(format!("Failed to stop agent: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Agent stop command sent",
            "agent_id": id,
        })),
    ))
}

/// Get agent status (V2)
pub async fn get_agent_status_v2(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, GatewayError> {
    // 🟢 P1 FIX: Use AgentRuntime trait
    let status = state
        .agent_runtime
        .status(&id)
        .await
        .map_err(|e| GatewayError::agent(format!("Failed to get agent status: {}", e)))?;

    Ok(Json(json!({
        "agent_id": status.agent_id,
        "state": status.state.to_string(),
        "current_task": status.current_task,
        "last_activity": status.last_activity,
        "total_tasks": status.total_tasks,
        "failed_tasks": status.failed_tasks,
        "kernel_task_id": status.kernel_task_id,
    })))
}

/// Execute task on agent (V2)
#[derive(Debug, Deserialize)]
pub struct ExecuteTaskRequest {
    pub task_type: String,
    pub input: serde_json::Value,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    60
}

pub async fn execute_task_v2(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<ExecuteTaskRequest>,
) -> Result<Json<serde_json::Value>, GatewayError> {
    let task_config = TaskConfig {
        task_type: req.task_type,
        input: req.input,
        timeout_secs: req.timeout_secs,
        priority: 5,
    };

    // 🟢 P1 FIX: Use AgentRuntime trait to execute task
    let result = state
        .agent_runtime
        .execute_task(&id, task_config)
        .await
        .map_err(|e| GatewayError::agent(format!("Failed to execute task: {}", e)))?;

    Ok(Json(json!({
        "success": result.success,
        "output": result.output,
        "execution_time_ms": result.execution_time_ms,
        "error": result.error,
    })))
}

/// Bind an agent to a channel
#[derive(Debug, Deserialize)]
pub struct BindChannelRequest {
    pub platform: String,
    pub channel_id: String,
}

pub async fn bind_agent_channel(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<BindChannelRequest>,
) -> Result<impl IntoResponse, GatewayError> {
    let store = state
        .channel_binding_store
        .as_ref()
        .ok_or_else(|| GatewayError::internal("Channel binding store not initialized"))?;

    // Verify agent exists
    let _ = state
        .agent_runtime
        .status(&id)
        .await
        .map_err(|e| GatewayError::agent(format!("Agent not found: {}", e)))?;

    store
        .bind(&req.platform, &req.channel_id, &id)
        .await
        .map_err(|e| GatewayError::internal(format!("Failed to bind channel: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Channel bound successfully",
            "agent_id": id,
            "platform": req.platform,
            "channel_id": req.channel_id,
        })),
    ))
}

/// Unbind an agent from a channel
pub async fn unbind_agent_channel(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path((id, channel_id)): Path<(String, String)>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, GatewayError> {
    let store = state
        .channel_binding_store
        .as_ref()
        .ok_or_else(|| GatewayError::internal("Channel binding store not initialized"))?;

    let platform = params
        .get("platform")
        .cloned()
        .unwrap_or_else(|| "webchat".to_string());

    store
        .unbind(&platform, &channel_id)
        .await
        .map_err(|e| GatewayError::internal(format!("Failed to unbind channel: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Channel unbound successfully",
            "agent_id": id,
            "platform": platform,
            "channel_id": channel_id,
        })),
    ))
}

/// List channels bound to an agent
pub async fn list_agent_channels(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, GatewayError> {
    let store = state
        .channel_binding_store
        .as_ref()
        .ok_or_else(|| GatewayError::internal("Channel binding store not initialized"))?;

    let bindings = store
        .list_bindings_for_agent(&id)
        .await
        .map_err(|e| GatewayError::internal(format!("Failed to list bindings: {}", e)))?;

    Ok(Json(json!({
        "agent_id": id,
        "bindings": bindings,
        "total": bindings.len(),
    })))
}

// Migration Guide Comments:
//
// To migrate existing handlers:
//
// 1. Replace `state.agent_service.xxx()` with `state.agent_runtime.xxx()` for agent lifecycle
// 2. Replace database queries with `state.state_store.query()` for reads
// 3. Replace state changes with `state.state_store.execute()` for writes
// 4. Use `GatewayAgentConfig` instead of internal `AgentConfig`
// 5. Use `AgentState` from gateway-lib instead of internal state
//
// Benefits:
// - Decoupled from concrete beebotos_agents implementation
// - Type-safe trait-based interface
// - CQRS pattern for better performance
// - Event sourcing for audit trail
