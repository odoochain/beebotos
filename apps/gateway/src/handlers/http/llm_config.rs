//! LLM Global Configuration HTTP Handler
//!
//! Provides read-only access to the current global LLM configuration.
//! Sensitive fields (API keys) are masked for security.

use axum::extract::State;
use axum::Json;
use gateway::{
    error::GatewayError,
    middleware::{require_any_role, AuthUser},
};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;

/// Global LLM configuration response
#[derive(Debug, Serialize)]
pub struct LlmGlobalConfigResponse {
    pub default_provider: String,
    pub fallback_chain: Vec<String>,
    pub cost_optimization: bool,
    pub max_tokens: u32,
    pub system_prompt: String,
    pub request_timeout: u64,
    pub providers: Vec<ProviderConfigResponse>,
}

/// Provider configuration (with masked API key)
#[derive(Debug, Serialize)]
pub struct ProviderConfigResponse {
    pub name: String,
    pub api_key_masked: String,
    pub model: String,
    pub base_url: String,
    pub temperature: f32,
    pub context_window: Option<u32>,
}

/// Get current global LLM configuration (read-only, sensitive fields masked)
pub async fn get_llm_global_config(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<LlmGlobalConfigResponse>, GatewayError> {
    require_any_role(&user, &["user", "admin"])?;

    let config = &state.config.models;

    let providers = config
        .providers
        .iter()
        .map(|(name, provider)| ProviderConfigResponse {
            name: name.clone(),
            api_key_masked: mask_api_key(provider.api_key.as_deref().unwrap_or("")),
            model: provider.model.clone().unwrap_or_default(),
            base_url: provider.base_url.clone().unwrap_or_default(),
            temperature: provider.temperature,
            context_window: provider.context_window.map(|v| v as u32),
        })
        .collect();

    Ok(Json(LlmGlobalConfigResponse {
        default_provider: config.default_provider.clone(),
        fallback_chain: config.fallback_chain.clone(),
        cost_optimization: config.cost_optimization,
        max_tokens: config.max_tokens,
        system_prompt: config.system_prompt.clone(),
        request_timeout: config.request_timeout,
        providers,
    }))
}

/// Mask an API key for display (show first 4 and last 4 chars)
fn mask_api_key(key: &str) -> String {
    if key.len() <= 12 {
        "****".to_string()
    } else {
        format!("{}****{}", &key[..4], &key[key.len() - 4..])
    }
}
