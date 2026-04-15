//! Permission System
//!
//! Capability-based permission system for agent operations.
//!
//! # Security Model
//! - Each agent has a set of capabilities
//! - Each operation requires specific capabilities
//! - Runtime permission checking with audit logging

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::{AgentError, Result};
use crate::types::AgentId;

/// Capability represents a permission to perform an action on a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Capability {
    pub resource: String,
    pub action: String,
}

impl Capability {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }

    /// Wildcard capability for a resource (all actions)
    pub fn wildcard(resource: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: "*".to_string(),
        }
    }

    /// Check if this capability matches a required capability
    pub fn matches(&self, required: &Capability) -> bool {
        self.resource == required.resource && (self.action == "*" || self.action == required.action)
    }
}

/// Permission check result
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionResult {
    Allowed,
    Denied { reason: String },
}

/// Runtime permission checker
pub struct PermissionChecker {
    /// Agent capabilities storage
    agent_capabilities: Arc<RwLock<HashMap<AgentId, HashSet<Capability>>>>,
    /// Audit log for permission checks
    audit_log: Arc<RwLock<Vec<PermissionAuditEvent>>>,
    /// Enable audit logging
    enable_audit: bool,
    /// Default deny (if true, no capabilities means deny all)
    default_deny: bool,
}

impl PermissionChecker {
    pub fn new() -> Self {
        Self {
            agent_capabilities: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            enable_audit: true,
            default_deny: true,
        }
    }

    pub fn without_audit_logging(mut self) -> Self {
        self.enable_audit = false;
        self
    }

    pub fn with_default_allow(mut self) -> Self {
        self.default_deny = false;
        self
    }

    /// Register agent capabilities
    pub async fn register_agent_capabilities(
        &self,
        agent_id: AgentId,
        capabilities: Vec<Capability>,
    ) {
        let caps: HashSet<Capability> = capabilities.into_iter().collect();
        let cap_count = caps.len();
        let mut storage = self.agent_capabilities.write().await;
        storage.insert(agent_id, caps);
        info!(
            "Registered {} capabilities for agent {}",
            cap_count, agent_id
        );
    }

    /// Check if agent has required capability
    pub async fn check_permission(
        &self,
        agent_id: &AgentId,
        required: &Capability,
        context: Option<PermissionContext>,
    ) -> Result<()> {
        let result = self.evaluate_permission(agent_id, required).await;

        // Audit log
        if self.enable_audit {
            self.log_permission_check(agent_id, required, &result, context)
                .await;
        }

        match result {
            PermissionResult::Allowed => {
                debug!(
                    "Permission granted: {} on {}",
                    required.action, required.resource
                );
                Ok(())
            }
            PermissionResult::Denied { reason } => {
                warn!(
                    "Permission denied for agent {}: {} on {} - {}",
                    agent_id, required.action, required.resource, reason
                );
                Err(AgentError::CapabilityDenied(format!(
                    "Missing capability: {} on {} - {}",
                    required.action, required.resource, reason
                )))
            }
        }
    }

    /// Evaluate permission without logging
    async fn evaluate_permission(
        &self,
        agent_id: &AgentId,
        required: &Capability,
    ) -> PermissionResult {
        let capabilities = self.agent_capabilities.read().await;

        match capabilities.get(agent_id) {
            Some(caps) => {
                // Check if any capability matches
                if caps.iter().any(|cap| cap.matches(required)) {
                    PermissionResult::Allowed
                } else {
                    PermissionResult::Denied {
                        reason: format!(
                            "Agent does not have {} permission on {}",
                            required.action, required.resource
                        ),
                    }
                }
            }
            None => {
                if self.default_deny {
                    PermissionResult::Denied {
                        reason: "Agent has no registered capabilities".to_string(),
                    }
                } else {
                    PermissionResult::Allowed
                }
            }
        }
    }

    /// Get agent capabilities
    pub async fn get_capabilities(&self, agent_id: &AgentId) -> Vec<Capability> {
        let capabilities = self.agent_capabilities.read().await;
        capabilities
            .get(agent_id)
            .map(|caps| caps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Remove agent capabilities
    pub async fn remove_agent(&self, agent_id: &AgentId) {
        let mut capabilities = self.agent_capabilities.write().await;
        capabilities.remove(agent_id);
    }

    /// Log permission check to audit log
    async fn log_permission_check(
        &self,
        agent_id: &AgentId,
        capability: &Capability,
        result: &PermissionResult,
        context: Option<PermissionContext>,
    ) {
        let event = PermissionAuditEvent {
            timestamp: chrono::Utc::now(),
            agent_id: agent_id.clone(),
            capability: capability.clone(),
            allowed: matches!(result, PermissionResult::Allowed),
            context,
        };

        let mut log = self.audit_log.write().await;
        log.push(event);

        // Limit audit log size
        if log.len() > 10000 {
            log.drain(0..1000);
        }
    }

    /// Get recent audit events
    pub async fn get_audit_log(&self, limit: usize) -> Vec<PermissionAuditEvent> {
        let log = self.audit_log.read().await;
        log.iter().rev().take(limit).cloned().collect()
    }

    /// Check if agent has any of the required capabilities
    pub async fn check_any_permission(
        &self,
        agent_id: &AgentId,
        required: &[Capability],
    ) -> Result<()> {
        for cap in required {
            if self.evaluate_permission(agent_id, cap).await == PermissionResult::Allowed {
                return Ok(());
            }
        }

        Err(AgentError::CapabilityDenied(
            "Agent does not have any of the required capabilities".to_string(),
        ))
    }

    /// Check if agent has all of the required capabilities
    pub async fn check_all_permissions(
        &self,
        agent_id: &AgentId,
        required: &[Capability],
    ) -> Result<()> {
        for cap in required {
            self.check_permission(agent_id, cap, None).await?;
        }
        Ok(())
    }
}

/// Permission context for audit logging
#[derive(Debug, Clone)]
pub struct PermissionContext {
    pub operation: String,
    pub resource_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl PermissionContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            resource_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_resource(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }
}

/// Permission audit event
#[derive(Debug, Clone)]
pub struct PermissionAuditEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent_id: AgentId,
    pub capability: Capability,
    pub allowed: bool,
    pub context: Option<PermissionContext>,
}

/// Predefined common capabilities
pub mod capabilities {
    use super::Capability;

    pub fn file_read() -> Capability {
        Capability::new("file", "read")
    }

    pub fn file_write() -> Capability {
        Capability::new("file", "write")
    }

    pub fn file_delete() -> Capability {
        Capability::new("file", "delete")
    }

    pub fn network_http() -> Capability {
        Capability::new("network", "http")
    }

    pub fn network_websocket() -> Capability {
        Capability::new("network", "websocket")
    }

    pub fn llm_chat() -> Capability {
        Capability::new("llm", "chat")
    }

    pub fn llm_embedding() -> Capability {
        Capability::new("llm", "embedding")
    }

    pub fn skill_execute() -> Capability {
        Capability::new("skill", "execute")
    }

    pub fn a2a_send() -> Capability {
        Capability::new("a2a", "send")
    }

    pub fn a2a_receive() -> Capability {
        Capability::new("a2a", "receive")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capability_matching() {
        let wildcard = Capability::wildcard("file");
        let read = Capability::new("file", "read");
        let write = Capability::new("file", "write");

        assert!(wildcard.matches(&read));
        assert!(wildcard.matches(&write));

        let specific = Capability::new("file", "read");
        assert!(specific.matches(&read));
        assert!(!specific.matches(&write));
    }

    #[tokio::test]
    async fn test_permission_checker() {
        let checker = PermissionChecker::new();
        let agent_id = AgentId::new();

        // Register agent with capabilities
        checker
            .register_agent_capabilities(
                agent_id,
                vec![
                    Capability::new("file", "read"),
                    Capability::new("file", "write"),
                ],
            )
            .await;

        // Should allow read
        assert!(checker
            .check_permission(&agent_id, &Capability::new("file", "read"), None)
            .await
            .is_ok());

        // Should deny delete
        assert!(checker
            .check_permission(&agent_id, &Capability::new("file", "delete"), None)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_unknown_agent() {
        let checker = PermissionChecker::new();
        let unknown_agent = AgentId::new();

        // Unknown agent should be denied with default_deny=true
        assert!(checker
            .check_permission(&unknown_agent, &Capability::new("file", "read"), None)
            .await
            .is_err());
    }
}
