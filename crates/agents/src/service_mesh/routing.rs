//! Service Routing Module
//!
//! 服务路由和负载均衡功能。

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::RwLock;
use tracing::{debug, info};

use super::registry::{ServiceEntry, ServiceState};
use super::resolver::ServiceResolver;
use crate::error::{AgentError, Result};

/// 路由策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStrategy {
    /// 轮询
    RoundRobin,
    /// 随机
    Random,
    /// 最少连接
    LeastConnections,
    /// 加权轮询
    WeightedRoundRobin,
    /// 一致性哈希
    ConsistentHash,
    /// 主备（第一个健康，其他备份）
    PrimaryBackup,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::RoundRobin
    }
}

/// 服务路由器
///
/// 提供服务路由和负载均衡功能。
pub struct ServiceRouter {
    /// 服务解析器
    resolver: ServiceResolver,
    /// 路由策略
    strategy: RoutingStrategy,
    /// 轮询计数器
    round_robin_counter: AtomicUsize,
    /// 加权轮询计数器
    weighted_counter: AtomicUsize,
    /// 连接计数（用于最少连接策略）
    connection_counts: RwLock<HashMap<String, usize>>,
}

impl ServiceRouter {
    /// 创建新的服务路由器
    pub fn new(resolver: ServiceResolver, strategy: super::LoadBalanceStrategy) -> Self {
        let strategy = match strategy {
            super::LoadBalanceStrategy::RoundRobin => RoutingStrategy::RoundRobin,
            super::LoadBalanceStrategy::Random => RoutingStrategy::Random,
            super::LoadBalanceStrategy::LeastConnections => RoutingStrategy::LeastConnections,
            super::LoadBalanceStrategy::WeightedRoundRobin => RoutingStrategy::WeightedRoundRobin,
            super::LoadBalanceStrategy::ConsistentHash => RoutingStrategy::ConsistentHash,
        };

        Self {
            resolver,
            strategy,
            round_robin_counter: AtomicUsize::new(0),
            weighted_counter: AtomicUsize::new(0),
            connection_counts: RwLock::new(HashMap::new()),
        }
    }

    /// 设置路由策略
    pub fn set_strategy(&mut self, strategy: RoutingStrategy) {
        self.strategy = strategy;
        info!("Routing strategy changed to {:?}", strategy);
    }

    /// 路由请求到服务
    ///
    /// 根据路由策略选择一个健康的服务实例
    pub async fn route(
        &self,
        service_name: &str,
        routing_key: Option<&str>,
    ) -> Result<ServiceEntry> {
        debug!("Routing request for service: {}", service_name);

        // 解析服务
        let services = self
            .resolver
            .resolve_by_name(service_name)
            .await
            .map_err(|e| AgentError::ServiceMesh(format!("Service resolution failed: {}", e)))?;

        if services.is_empty() {
            return Err(AgentError::ServiceMesh(format!(
                "No available instances for service: {}",
                service_name
            )));
        }

        // 过滤健康的服务
        let healthy_services: Vec<_> = services
            .into_iter()
            .filter(|s| s.state == ServiceState::Healthy)
            .collect();

        if healthy_services.is_empty() {
            return Err(AgentError::ServiceMesh(format!(
                "No healthy instances for service: {}",
                service_name
            )));
        }

        // 根据策略选择服务
        let selected = match self.strategy {
            RoutingStrategy::RoundRobin => self.select_round_robin(&healthy_services),
            RoutingStrategy::Random => self.select_random(&healthy_services),
            RoutingStrategy::LeastConnections => {
                self.select_least_connections(&healthy_services).await
            }
            RoutingStrategy::WeightedRoundRobin => {
                self.select_weighted_round_robin(&healthy_services)
            }
            RoutingStrategy::ConsistentHash => {
                let key = routing_key.ok_or_else(|| {
                    AgentError::InvalidConfig(
                        "Routing key required for consistent hash".to_string(),
                    )
                })?;
                self.select_consistent_hash(&healthy_services, key)
            }
            RoutingStrategy::PrimaryBackup => self.select_primary_backup(&healthy_services),
        };

        // 增加连接计数
        self.increment_connection(&selected.id).await;

        debug!(
            "Selected service instance: {} for {}",
            selected.id, service_name
        );

        Ok(selected)
    }

    /// 轮询选择
    fn select_round_robin(&self, services: &[ServiceEntry]) -> ServiceEntry {
        let index = self.round_robin_counter.fetch_add(1, Ordering::SeqCst) % services.len();
        services[index].clone()
    }

    /// 随机选择
    fn select_random(&self, services: &[ServiceEntry]) -> ServiceEntry {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..services.len());
        services[index].clone()
    }

    /// 最少连接选择
    async fn select_least_connections(&self, services: &[ServiceEntry]) -> ServiceEntry {
        let counts = self.connection_counts.read().await;

        services
            .iter()
            .min_by_key(|s| counts.get(&s.id).copied().unwrap_or(0))
            .cloned()
            .unwrap_or_else(|| services[0].clone())
    }

    /// 加权轮询选择
    fn select_weighted_round_robin(&self, services: &[ServiceEntry]) -> ServiceEntry {
        // 计算总权重
        let total_weight: u32 = services
            .iter()
            .map(|s| s.endpoints.first().map(|e| e.weight).unwrap_or(1))
            .sum();

        if total_weight == 0 {
            return self.select_round_robin(services);
        }

        // 加权选择
        let counter = self.weighted_counter.fetch_add(1, Ordering::SeqCst) as u32 % total_weight;
        let mut current_weight = 0;

        for service in services {
            let weight = service.endpoints.first().map(|e| e.weight).unwrap_or(1);

            current_weight += weight;

            if counter < current_weight {
                return service.clone();
            }
        }

        services[0].clone()
    }

    /// 一致性哈希选择
    fn select_consistent_hash(&self, services: &[ServiceEntry], key: &str) -> ServiceEntry {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        let index = (hash as usize) % services.len();
        services[index].clone()
    }

    /// 主备选择
    fn select_primary_backup(&self, services: &[ServiceEntry]) -> ServiceEntry {
        // 第一个服务是主服务
        services[0].clone()
    }

    /// 增加连接计数
    async fn increment_connection(&self, service_id: &str) {
        let mut counts = self.connection_counts.write().await;
        *counts.entry(service_id.to_string()).or_insert(0) += 1;
    }

    /// 减少连接计数
    pub async fn decrement_connection(&self, service_id: &str) {
        let mut counts = self.connection_counts.write().await;
        if let Some(count) = counts.get_mut(service_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    /// 获取连接计数
    pub async fn get_connection_count(&self, service_id: &str) -> usize {
        let counts = self.connection_counts.read().await;
        counts.get(service_id).copied().unwrap_or(0)
    }

    /// 获取路由策略
    pub fn strategy(&self) -> RoutingStrategy {
        self.strategy
    }
}

/// 负载均衡器
///
/// 提供高级负载均衡功能。
pub struct LoadBalancer {
    /// 服务名称
    service_name: String,
    /// 路由策略
    strategy: RoutingStrategy,
    /// 可用服务列表
    services: RwLock<Vec<ServiceEntry>>,
    /// 健康检查间隔
    #[allow(dead_code)]
    health_check_interval_secs: u64,
}

impl LoadBalancer {
    /// 创建新的负载均衡器
    pub fn new(service_name: impl Into<String>, strategy: RoutingStrategy) -> Self {
        Self {
            service_name: service_name.into(),
            strategy,
            services: RwLock::new(vec![]),
            health_check_interval_secs: 30,
        }
    }

    /// 更新服务列表
    pub async fn update_services(&self, services: Vec<ServiceEntry>) {
        let mut current = self.services.write().await;
        *current = services;
        debug!(
            "Updated {} services for {}",
            current.len(),
            self.service_name
        );
    }

    /// 获取服务名称
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// 获取路由策略
    pub fn strategy(&self) -> RoutingStrategy {
        self.strategy
    }
}

/// 路由结果
#[derive(Debug, Clone)]
pub struct RouteResult {
    /// 选中的服务
    pub service: ServiceEntry,
    /// 路由策略
    pub strategy_used: RoutingStrategy,
    /// 可用服务数
    pub available_services: usize,
    /// 路由耗时（微秒）
    pub routing_time_us: u64,
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;

    use super::super::registry::{
        InMemoryServiceRegistry, Protocol, ServiceEndpoint, ServiceEntry,
    };
    use super::super::resolver::ServiceResolver;
    use super::*;

    fn create_test_service(id: &str, name: &str, port: u16) -> ServiceEntry {
        ServiceEntry::new(id, name, "1.0.0").with_endpoint(ServiceEndpoint {
            protocol: Protocol::Http,
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port),
            path: None,
            weight: 1,
        })
    }

    #[tokio::test]
    async fn test_round_robin_selection() {
        let services = vec![
            create_test_service("svc-1", "test", 8080),
            create_test_service("svc-2", "test", 8081),
            create_test_service("svc-3", "test", 8082),
        ];

        let registry = Arc::new(InMemoryServiceRegistry::new());
        let did_resolver = Arc::new(crate::did::DIDResolver::new());
        let resolver = ServiceResolver::new(registry, did_resolver);
        let router = ServiceRouter::new(resolver, super::super::LoadBalanceStrategy::RoundRobin);

        // 轮询应该选择不同的服务
        let selected1 = router.select_round_robin(&services);
        let selected2 = router.select_round_robin(&services);
        let _selected3 = router.select_round_robin(&services);
        let selected4 = router.select_round_robin(&services);

        // 第1个和第4个应该相同（3个服务循环）
        assert_eq!(selected1.id, selected4.id);
        // 第1个和第2个应该不同
        assert_ne!(selected1.id, selected2.id);
    }

    #[tokio::test]
    async fn test_random_selection() {
        let services = vec![
            create_test_service("svc-1", "test", 8080),
            create_test_service("svc-2", "test", 8081),
        ];

        let registry = Arc::new(InMemoryServiceRegistry::new());
        let did_resolver = Arc::new(crate::did::DIDResolver::new());
        let resolver = ServiceResolver::new(registry, did_resolver);
        let router = ServiceRouter::new(resolver, super::super::LoadBalanceStrategy::Random);

        // 随机选择应该返回有效服务
        let selected = router.select_random(&services);
        assert!(services.iter().any(|s| s.id == selected.id));
    }

    #[tokio::test]
    async fn test_consistent_hash_selection() {
        let services = vec![
            create_test_service("svc-1", "test", 8080),
            create_test_service("svc-2", "test", 8081),
            create_test_service("svc-3", "test", 8082),
        ];

        let registry = Arc::new(InMemoryServiceRegistry::new());
        let did_resolver = Arc::new(crate::did::DIDResolver::new());
        let resolver = ServiceResolver::new(registry, did_resolver);
        let router =
            ServiceRouter::new(resolver, super::super::LoadBalanceStrategy::ConsistentHash);

        // 相同 key 应该选择相同服务
        let key = "user-123";
        let selected1 = router.select_consistent_hash(&services, key);
        let selected2 = router.select_consistent_hash(&services, key);

        assert_eq!(selected1.id, selected2.id);

        // 不同 key 可能选择不同服务
        let selected3 = router.select_consistent_hash(&services, "user-456");
        // 不强制要求不同，只是验证能正常工作
        assert!(services.iter().any(|s| s.id == selected3.id));
    }

    #[tokio::test]
    async fn test_connection_counting() {
        let registry = Arc::new(InMemoryServiceRegistry::new());
        let did_resolver = Arc::new(crate::did::DIDResolver::new());
        let resolver = ServiceResolver::new(registry, did_resolver);
        let router = ServiceRouter::new(
            resolver,
            super::super::LoadBalanceStrategy::LeastConnections,
        );

        // 增加连接计数
        router.increment_connection("svc-1").await;
        router.increment_connection("svc-1").await;
        router.increment_connection("svc-2").await;

        assert_eq!(router.get_connection_count("svc-1").await, 2);
        assert_eq!(router.get_connection_count("svc-2").await, 1);
        assert_eq!(router.get_connection_count("svc-3").await, 0);

        // 减少连接计数
        router.decrement_connection("svc-1").await;
        assert_eq!(router.get_connection_count("svc-1").await, 1);
    }

    #[tokio::test]
    async fn test_load_balancer() {
        let lb = LoadBalancer::new("test-service", RoutingStrategy::RoundRobin);

        assert_eq!(lb.service_name(), "test-service");
        assert_eq!(lb.strategy(), RoutingStrategy::RoundRobin);

        let services = vec![
            create_test_service("svc-1", "test", 8080),
            create_test_service("svc-2", "test", 8081),
        ];

        lb.update_services(services).await;
    }
}
