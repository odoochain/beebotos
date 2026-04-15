//! Health Check Module
//!
//! 🟡 P1 FIX: Production-ready health checks with HTTP endpoint support

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    pub fn http_status_code(&self) -> u16 {
        match self {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200, // Still serving but degraded
            HealthStatus::Unhealthy => 503, // Service Unavailable
        }
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Component health check
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> ComponentHealth;
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 🟡 P1 FIX: Complete health check response for HTTP endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Overall health status
    pub status: HealthStatus,
    /// Service name
    pub service: String,
    /// Service version
    pub version: String,
    /// Current timestamp (ISO 8601)
    pub timestamp: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Individual component health checks
    pub components: Vec<ComponentHealth>,
    /// Summary metrics
    pub summary: HealthSummary,
}

/// Health summary metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub total_components: usize,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unhealthy_count: usize,
}

/// Health registry with HTTP endpoint support
pub struct HealthRegistry {
    checks: RwLock<HashMap<String, Arc<dyn HealthCheck>>>,
    service_name: String,
    service_version: String,
    start_time: std::time::Instant,
}

impl HealthRegistry {
    pub fn new(service_name: impl Into<String>, service_version: impl Into<String>) -> Self {
        Self {
            checks: RwLock::new(HashMap::new()),
            service_name: service_name.into(),
            service_version: service_version.into(),
            start_time: std::time::Instant::now(),
        }
    }

    /// Create with default service info
    pub fn default_with_service() -> Self {
        Self::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }

    /// Register a health check
    pub async fn register(&self, name: impl Into<String>, check: Arc<dyn HealthCheck>) {
        let mut checks = self.checks.write().await;
        checks.insert(name.into(), check);
    }

    /// Run all health checks
    pub async fn check_all(&self) -> Vec<ComponentHealth> {
        let checks = self.checks.read().await;
        let mut results = Vec::new();

        for (_name, check) in checks.iter() {
            let start = std::time::Instant::now();
            let mut health = check.check().await;
            health.latency_ms = start.elapsed().as_millis() as u64;
            results.push(health);
        }

        results
    }

    /// Get overall health status
    pub async fn overall_status(&self) -> HealthStatus {
        let results = self.check_all().await;

        if results.iter().all(|r| r.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if results.iter().any(|r| r.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        }
    }

    /// 🟡 P1 FIX: Generate complete health check response for HTTP endpoint
    pub async fn health_check_response(&self) -> HealthCheckResponse {
        let components = self.check_all().await;
        let status = self.calculate_overall_status(&components);

        let healthy_count = components
            .iter()
            .filter(|c| c.status == HealthStatus::Healthy)
            .count();
        let degraded_count = components
            .iter()
            .filter(|c| c.status == HealthStatus::Degraded)
            .count();
        let unhealthy_count = components
            .iter()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .count();

        HealthCheckResponse {
            status,
            service: self.service_name.clone(),
            version: self.service_version.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            components,
            summary: HealthSummary {
                total_components: healthy_count + degraded_count + unhealthy_count,
                healthy_count,
                degraded_count,
                unhealthy_count,
            },
        }
    }

    fn calculate_overall_status(&self, components: &[ComponentHealth]) -> HealthStatus {
        if components.iter().all(|c| c.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if components
            .iter()
            .any(|c| c.status == HealthStatus::Unhealthy)
        {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        }
    }
}

impl Default for HealthRegistry {
    fn default() -> Self {
        Self::default_with_service()
    }
}

/// 🟡 P1 FIX: Common health check implementations

/// Simple health check that always returns healthy
pub struct AlwaysHealthyCheck {
    name: String,
}

impl AlwaysHealthyCheck {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait::async_trait]
impl HealthCheck for AlwaysHealthyCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> ComponentHealth {
        ComponentHealth {
            name: self.name.clone(),
            status: HealthStatus::Healthy,
            message: Some("OK".to_string()),
            latency_ms: 0,
            metadata: None,
        }
    }
}

/// Health check that calls a function
pub struct FnHealthCheck<F> {
    name: String,
    check_fn: F,
}

impl<F, Fut> FnHealthCheck<F>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<(), String>> + Send,
{
    pub fn new(name: impl Into<String>, check_fn: F) -> Self {
        Self {
            name: name.into(),
            check_fn,
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut> HealthCheck for FnHealthCheck<F>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<(), String>> + Send,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        match (self.check_fn)().await {
            Ok(()) => ComponentHealth {
                name: self.name.clone(),
                status: HealthStatus::Healthy,
                message: Some("OK".to_string()),
                latency_ms: start.elapsed().as_millis() as u64,
                metadata: None,
            },
            Err(msg) => ComponentHealth {
                name: self.name.clone(),
                status: HealthStatus::Unhealthy,
                message: Some(msg),
                latency_ms: start.elapsed().as_millis() as u64,
                metadata: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_status_serialization() {
        assert_eq!(
            serde_json::to_string(&HealthStatus::Healthy).unwrap(),
            "\"healthy\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Degraded).unwrap(),
            "\"degraded\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Unhealthy).unwrap(),
            "\"unhealthy\""
        );
    }

    #[tokio::test]
    async fn test_always_healthy_check() {
        let check = AlwaysHealthyCheck::new("test");
        let health = check.check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.name, "test");
    }

    #[tokio::test]
    async fn test_fn_health_check_success() {
        let check = FnHealthCheck::new("test", || async { Ok(()) });
        let health = check.check().await;
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_fn_health_check_failure() {
        let check = FnHealthCheck::new("test", || async { Err("failed".to_string()) });
        let health = check.check().await;
        assert_eq!(health.status, HealthStatus::Unhealthy);
        assert_eq!(health.message, Some("failed".to_string()));
    }

    #[tokio::test]
    async fn test_health_registry_response() {
        let registry = HealthRegistry::new("test-service", "1.0.0");
        registry
            .register(
                "component1",
                Arc::new(AlwaysHealthyCheck::new("component1")),
            )
            .await;

        let response = registry.health_check_response().await;
        assert_eq!(response.service, "test-service");
        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.status, HealthStatus::Healthy);
        assert_eq!(response.summary.total_components, 1);
        assert_eq!(response.summary.healthy_count, 1);
    }
}
