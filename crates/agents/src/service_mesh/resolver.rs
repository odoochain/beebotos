//! Service Resolver Module
//!
//! 服务解析器，集成 DID Resolver 提供服务发现功能。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::registry::{ServiceEntry, ServiceRegistry};
use crate::did::{DIDDocument, DIDResolver};
use crate::error::{AgentError, Result};

/// 服务解析器
///
/// 提供服务发现功能，集成 DID Resolver 支持链上身份验证。
#[derive(Clone)]
pub struct ServiceResolver {
    /// 服务注册表
    registry: Arc<dyn ServiceRegistry>,
    /// DID Resolver
    did_resolver: Arc<DIDResolver>,
    /// 解析缓存
    cache: Arc<RwLock<HashMap<String, CachedResolution>>>,
    /// 缓存 TTL（秒）
    cache_ttl_secs: u64,
}

/// 缓存的解析结果
#[derive(Debug, Clone)]
struct CachedResolution {
    entries: Vec<ServiceEntry>,
    timestamp: std::time::Instant,
}

impl CachedResolution {
    fn is_expired(&self, ttl_secs: u64) -> bool {
        self.timestamp.elapsed().as_secs() > ttl_secs
    }
}

impl ServiceResolver {
    /// 创建新的服务解析器
    pub fn new(registry: Arc<dyn ServiceRegistry>, did_resolver: Arc<DIDResolver>) -> Self {
        Self {
            registry,
            did_resolver,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_secs: 60,
        }
    }

    /// 设置缓存 TTL
    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.cache_ttl_secs = ttl_secs;
        self
    }

    /// 按名称解析服务
    ///
    /// 从注册表中查找指定名称的所有健康服务实例
    pub async fn resolve_by_name(&self, name: &str) -> Result<Vec<ServiceEntry>> {
        let cache_key = format!("name:{}", name);

        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if !cached.is_expired(self.cache_ttl_secs) {
                    debug!("Service resolution cache hit for name: {}", name);
                    return Ok(cached.entries.clone());
                }
            }
        }

        // 从注册表查询
        debug!("Resolving service by name: {}", name);
        let entries = self
            .registry
            .find_by_name(name)
            .await
            .map_err(|e| AgentError::ServiceMesh(format!("Registry query failed: {}", e)))?;

        // 缓存结果
        let cached = CachedResolution {
            entries: entries.clone(),
            timestamp: std::time::Instant::now(),
        };

        let mut cache = self.cache.write().await;
        cache.insert(cache_key, cached);

        Ok(entries)
    }

    /// 按 DID 解析服务
    ///
    /// 首先解析 DID 获取 DID Document，然后查找关联的服务
    pub async fn resolve_by_did(&self, did: &str) -> Result<Option<ServiceEntry>> {
        let cache_key = format!("did:{}", did);

        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if !cached.is_expired(self.cache_ttl_secs) {
                    debug!("Service resolution cache hit for DID: {}", did);
                    return Ok(cached.entries.first().cloned());
                }
            }
        }

        // 解析 DID
        debug!("Resolving service by DID: {}", did);
        let doc = self
            .did_resolver
            .resolve(did)
            .await
            .map_err(|e| AgentError::DIDResolution(e.to_string()))?;

        // 从注册表查找与该 DID 关联的服务
        let entry = self
            .registry
            .find_by_did(did)
            .await
            .map_err(|e| AgentError::ServiceMesh(format!("Registry query failed: {}", e)))?;

        // 如果从注册表找不到，尝试从 DID Document 的服务端点构建
        let entries = if let Some(ref entry) = entry {
            vec![entry.clone()]
        } else {
            self.build_service_from_did(&doc)
                .map(|e| vec![e])
                .unwrap_or_default()
        };

        // 缓存结果
        let cached = CachedResolution {
            entries: entries.clone(),
            timestamp: std::time::Instant::now(),
        };

        let mut cache = self.cache.write().await;
        cache.insert(cache_key, cached);

        Ok(entries.into_iter().next())
    }

    /// 按标签解析服务
    ///
    /// 查找具有指定标签的所有服务
    pub async fn resolve_by_tag(&self, tag: &str) -> Result<Vec<ServiceEntry>> {
        let cache_key = format!("tag:{}", tag);

        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if !cached.is_expired(self.cache_ttl_secs) {
                    debug!("Service resolution cache hit for tag: {}", tag);
                    return Ok(cached.entries.clone());
                }
            }
        }

        // 从注册表查询
        debug!("Resolving service by tag: {}", tag);
        let entries = self
            .registry
            .find_by_tag(tag)
            .await
            .map_err(|e| AgentError::ServiceMesh(format!("Registry query failed: {}", e)))?;

        // 缓存结果
        let cached = CachedResolution {
            entries: entries.clone(),
            timestamp: std::time::Instant::now(),
        };

        let mut cache = self.cache.write().await;
        cache.insert(cache_key, cached);

        Ok(entries)
    }

    /// 按元数据解析服务
    ///
    /// 查找具有指定元数据键值对的服务
    pub async fn resolve_by_metadata(&self, key: &str, value: &str) -> Result<Vec<ServiceEntry>> {
        debug!("Resolving service by metadata: {}={}", key, value);

        let all_services = self
            .registry
            .list_all()
            .await
            .map_err(|e| AgentError::ServiceMesh(format!("Registry query failed: {}", e)))?;

        let filtered: Vec<_> = all_services
            .into_iter()
            .filter(|entry| entry.metadata.get(key).map(|v| v.as_str()) == Some(value))
            .collect();

        Ok(filtered)
    }

    /// 解析多个服务
    ///
    /// 批量解析多个服务名称
    pub async fn resolve_many(
        &self,
        names: &[String],
    ) -> Result<HashMap<String, Vec<ServiceEntry>>> {
        let mut results = HashMap::new();

        for name in names {
            match self.resolve_by_name(name).await {
                Ok(entries) => {
                    results.insert(name.clone(), entries);
                }
                Err(e) => {
                    warn!("Failed to resolve service {}: {}", name, e);
                    results.insert(name.clone(), vec![]);
                }
            }
        }

        Ok(results)
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Service resolver cache cleared");
    }

    /// 获取缓存统计
    pub async fn cache_stats(&self) -> (usize, u64) {
        let cache = self.cache.read().await;
        (cache.len(), self.cache_ttl_secs)
    }

    /// 从 DID Document 构建服务条目
    fn build_service_from_did(&self, doc: &DIDDocument) -> Option<ServiceEntry> {
        // 从 DID Document 的服务端点提取信息
        if doc.service.is_empty() {
            return None;
        }

        // 使用第一个服务端点
        let service = &doc.service[0];

        // 这里简化处理，实际应该根据 service_endpoint 解析地址和协议
        Some(ServiceEntry {
            id: doc.id.clone(),
            name: service.id.clone(),
            version: "1.0.0".to_string(),
            endpoints: vec![], // 应该从 service_endpoint 解析
            state: super::registry::ServiceState::Healthy,
            did: Some(doc.id.clone()),
            metadata: HashMap::new(),
            tags: vec!["did-resolved".to_string()],
            registered_at: 0,
            last_heartbeat_at: 0,
            ttl_secs: 300,
        })
    }

    /// 获取注册表引用
    pub fn registry(&self) -> &Arc<dyn ServiceRegistry> {
        &self.registry
    }

    /// 获取 DID Resolver 引用
    pub fn did_resolver(&self) -> &Arc<DIDResolver> {
        &self.did_resolver
    }
}

/// 服务解析结果
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// 解析到的服务
    pub entries: Vec<ServiceEntry>,
    /// 是否从缓存获取
    pub from_cache: bool,
    /// 解析耗时（毫秒）
    pub resolution_time_ms: u64,
}

/// 服务解析器构建器
pub struct ServiceResolverBuilder {
    registry: Option<Arc<dyn ServiceRegistry>>,
    did_resolver: Option<Arc<DIDResolver>>,
    cache_ttl_secs: u64,
}

impl ServiceResolverBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            registry: None,
            did_resolver: None,
            cache_ttl_secs: 60,
        }
    }

    /// 设置服务注册表
    pub fn with_registry(mut self, registry: Arc<dyn ServiceRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// 设置 DID Resolver
    pub fn with_did_resolver(mut self, resolver: Arc<DIDResolver>) -> Self {
        self.did_resolver = Some(resolver);
        self
    }

    /// 设置缓存 TTL
    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.cache_ttl_secs = ttl_secs;
        self
    }

    /// 构建服务解析器
    pub fn build(self) -> Result<ServiceResolver> {
        let registry = self
            .registry
            .ok_or_else(|| AgentError::InvalidConfig("Service registry is required".to_string()))?;

        let did_resolver = self
            .did_resolver
            .ok_or_else(|| AgentError::InvalidConfig("DID resolver is required".to_string()))?;

        Ok(ServiceResolver {
            registry,
            did_resolver,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_secs: self.cache_ttl_secs,
        })
    }
}

impl Default for ServiceResolverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use super::super::registry::{
        InMemoryServiceRegistry, Protocol, ServiceEndpoint, ServiceEntry,
    };
    use super::*;

    #[tokio::test]
    async fn test_service_resolver_new() {
        let registry = Arc::new(InMemoryServiceRegistry::new());
        let did_resolver = Arc::new(DIDResolver::new());

        let resolver = ServiceResolver::new(registry, did_resolver);

        let (size, ttl) = resolver.cache_stats().await;
        assert_eq!(size, 0);
        assert_eq!(ttl, 60);
    }

    #[tokio::test]
    async fn test_resolve_by_name() {
        let registry = Arc::new(InMemoryServiceRegistry::new());
        let did_resolver = Arc::new(DIDResolver::new());

        let resolver = ServiceResolver::new(registry.clone(), did_resolver);

        // 注册测试服务
        let entry =
            ServiceEntry::new("svc-1", "test-service", "1.0.0").with_endpoint(ServiceEndpoint {
                protocol: Protocol::Http,
                address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                path: None,
                weight: 1,
            });

        registry.register(entry).await.unwrap();

        // 解析服务
        let resolved = resolver.resolve_by_name("test-service").await.unwrap();
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].id, "svc-1");

        // 第二次解析应该从缓存获取
        let resolved2 = resolver.resolve_by_name("test-service").await.unwrap();
        assert_eq!(resolved2.len(), 1);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let registry = Arc::new(InMemoryServiceRegistry::new());
        let did_resolver = Arc::new(DIDResolver::new());

        let resolver = ServiceResolver::new(registry.clone(), did_resolver);

        // 注册并解析服务以填充缓存
        let entry = ServiceEntry::new("svc-1", "test-service", "1.0.0");
        registry.register(entry).await.unwrap();

        resolver.resolve_by_name("test-service").await.unwrap();

        let (size, _) = resolver.cache_stats().await;
        assert_eq!(size, 1);

        // 清除缓存
        resolver.clear_cache().await;

        let (size, _) = resolver.cache_stats().await;
        assert_eq!(size, 0);
    }
}
