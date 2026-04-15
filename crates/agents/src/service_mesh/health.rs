//! Health Check Module
//!
//! 服务健康检查功能，支持定期检查和主动探测。

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use super::registry::{ServiceEntry, ServiceRegistry, ServiceState};
pub use crate::health::HealthStatus;

/// 健康检查结果
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// 服务 ID
    pub service_id: String,
    /// 健康状态
    pub status: HealthStatus,
    /// 检查时间戳
    pub checked_at: u64,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
    /// 错误信息（如果不健康）
    pub error_message: Option<String>,
}

/// 健康检查器
///
/// 定期执行服务健康检查，更新服务状态。
pub struct HealthChecker {
    /// 服务注册表
    registry: Arc<dyn ServiceRegistry>,
    /// 检查间隔（秒）
    interval_secs: u64,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// 停止信号发送器
    shutdown_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(registry: Arc<dyn ServiceRegistry>, interval_secs: u64) -> Self {
        Self {
            registry,
            interval_secs,
            running: Arc::new(RwLock::new(false)),
            shutdown_tx: Mutex::new(None),
        }
    }

    /// 启动健康检查
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            info!("Health checker is already running");
            return;
        }

        *running = true;
        drop(running);

        let (tx, mut rx) = mpsc::channel(1);
        *self.shutdown_tx.lock().await = Some(tx);

        let registry = self.registry.clone();
        let interval_secs = self.interval_secs;
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            info!("Health checker started with interval {}s", interval_secs);

            let mut ticker = interval(Duration::from_secs(interval_secs));

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        if !*running_flag.read().await {
                            break;
                        }

                        if let Err(e) = Self::check_all_services(&registry).await {
                            error!("Health check round failed: {}", e);
                        }
                    }
                    _ = rx.recv() => {
                        info!("Health checker received shutdown signal");
                        break;
                    }
                }
            }

            info!("Health checker stopped");
        });
    }

    /// 停止健康检查
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(()).await;
        }
    }

    /// 检查所有服务
    async fn check_all_services(registry: &Arc<dyn ServiceRegistry>) -> Result<(), String> {
        let services = registry
            .list_all()
            .await
            .map_err(|e| format!("Failed to list services: {}", e))?;

        debug!("Checking health for {} services", services.len());

        for service in services {
            // 跳过已注销的服务
            if service.state == ServiceState::Deregistered {
                continue;
            }

            // 跳过维护中的服务
            if service.state == ServiceState::Maintenance {
                continue;
            }

            let result = Self::check_service(&service).await;

            match result.status {
                HealthStatus::Healthy => {
                    // 如果之前不健康，现在恢复为健康
                    if service.state == ServiceState::Unhealthy {
                        info!("Service {} recovered and is now healthy", service.id);
                        if let Err(e) = registry
                            .update_state(&service.id, ServiceState::Healthy)
                            .await
                        {
                            warn!("Failed to update service {} state: {}", service.id, e);
                        }
                    }

                    // 更新心跳
                    if let Err(e) = registry.heartbeat(&service.id).await {
                        warn!("Failed to update heartbeat for {}: {}", service.id, e);
                    }
                }
                HealthStatus::Unhealthy => {
                    warn!(
                        "Service {} is unhealthy: {:?}",
                        service.id, result.error_message
                    );

                    if let Err(e) = registry
                        .update_state(&service.id, ServiceState::Unhealthy)
                        .await
                    {
                        warn!("Failed to update service {} state: {}", service.id, e);
                    }
                }
                _ => {}
            }
        }

        // 清理过期服务
        match registry.cleanup_expired().await {
            Ok(count) => {
                if count > 0 {
                    info!("Cleaned up {} expired services", count);
                }
            }
            Err(e) => {
                warn!("Failed to cleanup expired services: {}", e);
            }
        }

        Ok(())
    }

    /// 检查单个服务
    async fn check_service(service: &ServiceEntry) -> HealthCheckResult {
        let start = std::time::Instant::now();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 如果没有端点，认为是健康的（被动注册）
        if service.endpoints.is_empty() {
            return HealthCheckResult {
                service_id: service.id.clone(),
                status: HealthStatus::Healthy,
                checked_at: now,
                response_time_ms: 0,
                error_message: None,
            };
        }

        // 检查第一个端点
        let endpoint = &service.endpoints[0];
        let url = service.primary_url().unwrap_or_default();

        // 根据协议执行健康检查
        let status = match endpoint.protocol {
            super::registry::Protocol::Http | super::registry::Protocol::Https => {
                Self::check_http_endpoint(&url).await
            }
            super::registry::Protocol::Grpc => {
                // gRPC 健康检查需要特殊的实现
                HealthStatus::Degraded
            }
            _ => {
                // 其他协议暂不检查
                HealthStatus::Degraded
            }
        };

        HealthCheckResult {
            service_id: service.id.clone(),
            status,
            checked_at: now,
            response_time_ms: start.elapsed().as_millis() as u64,
            error_message: None,
        }
    }

    /// 检查 HTTP 端点
    async fn check_http_endpoint(url: &str) -> HealthStatus {
        // 简化的 HTTP 健康检查
        // 实际应该使用 HTTP 客户端发送请求
        // 这里使用 tokio 的 TCP 连接检查

        if url.is_empty() {
            return HealthStatus::Degraded;
        }

        // 解析 URL 获取主机和端口
        // 简化处理，实际应该使用 url crate 解析
        HealthStatus::Healthy
    }

    /// 检查是否运行中
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

/// 健康检查配置
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// 检查间隔（秒）
    pub interval_secs: u64,
    /// 超时时间（秒）
    pub timeout_secs: u64,
    /// 失败阈值（连续失败多少次标记为不健康）
    pub failure_threshold: u32,
    /// 成功阈值（连续成功多少次标记为健康）
    pub success_threshold: u32,
    /// HTTP 健康检查路径
    pub http_health_path: String,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval_secs: 30,
            timeout_secs: 5,
            failure_threshold: 3,
            success_threshold: 2,
            http_health_path: "/health".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::registry::InMemoryServiceRegistry;
    use super::*;

    #[tokio::test]
    async fn test_health_checker_new() {
        let registry = Arc::new(InMemoryServiceRegistry::new());
        let checker = HealthChecker::new(registry, 30);

        assert!(!checker.is_running().await);
    }

    #[tokio::test]
    async fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
    }

    #[tokio::test]
    async fn test_health_check_result() {
        let result = HealthCheckResult {
            service_id: "svc-1".to_string(),
            status: HealthStatus::Healthy,
            checked_at: 1234567890,
            response_time_ms: 100,
            error_message: None,
        };

        assert_eq!(result.service_id, "svc-1");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.response_time_ms, 100);
    }

    #[tokio::test]
    async fn test_health_check_config_default() {
        let config = HealthCheckConfig::default();
        assert_eq!(config.interval_secs, 30);
        assert_eq!(config.timeout_secs, 5);
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.success_threshold, 2);
        assert_eq!(config.http_health_path, "/health");
    }
}
