//! DID (Decentralized Identifier) Resolver - Chain-Integrated
//!
//! 🔒 P0 FIX: Integrated with beebotos_chain::identity for on-chain
//! verification. This resolver queries the blockchain to verify DIDs and fetch
//! agent information.

// 🔒 P0 FIX: Import chain identity types
use std::sync::Arc;

use beebotos_chain::compat::Address;
use beebotos_chain::CachedIdentityRegistry;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// DID document - Re-exported from chain layer with agent-specific extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub service: Vec<Service>,
    /// On-chain verification status
    pub on_chain_verified: bool,
    /// Agent address if registered on-chain
    pub agent_address: Option<String>,
    /// Reputation score from chain
    pub reputation: Option<u64>,
}

// Note: From<ChainDIDDocument> implementation removed as it's not needed
// The DID resolver builds DIDDocument directly from chain data

/// Verification method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub vm_type: String,
    pub controller: String,
    pub public_key_hex: Option<String>,
    pub public_key_base58: Option<String>,
}

/// Service endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub service_type: String,
    pub service_endpoint: String,
}

/// On-chain DID resolver
///
/// 🔒 P0 FIX: This resolver integrates with the blockchain identity registry
/// to provide verifiable, on-chain identity resolution.
pub struct DIDResolver {
    /// Cache for resolved DIDs
    cache: RwLock<std::collections::HashMap<String, CachedDIDEntry>>,
    /// Chain identity registry client (optional - falls back to local
    /// resolution if None)
    identity_registry: Option<Arc<dyn ChainIdentityRegistry>>,
    /// Cache TTL in seconds
    cache_ttl_secs: u64,
}

/// Cached DID entry with timestamp
struct CachedDIDEntry {
    document: DIDDocument,
    timestamp: std::time::Instant,
}

/// Trait for chain identity registry operations
#[async_trait::async_trait]
pub trait ChainIdentityRegistry: Send + Sync {
    /// Resolve DID to address on chain
    async fn resolve_did(&self, did: &str) -> anyhow::Result<Option<Address>>;
    /// Get agent reputation
    async fn get_reputation(&self, did: &str) -> anyhow::Result<Option<u64>>;
    /// Check if DID is registered
    async fn is_registered(&self, did: &str) -> anyhow::Result<bool>;
}

/// Chain identity registry wrapper for OnChainIdentityRegistry
///
/// 🔒 P0 FIX: Generic over providers that implement AlloyProvider
pub struct ChainIdentityRegistryWrapper<
    P: beebotos_chain::compat::AlloyProvider + Clone + Send + Sync + 'static,
> {
    registry: CachedIdentityRegistry<P>,
}

impl<P: beebotos_chain::compat::AlloyProvider + Clone + Send + Sync + 'static>
    ChainIdentityRegistryWrapper<P>
{
    pub fn new(registry: CachedIdentityRegistry<P>) -> Self {
        Self { registry }
    }
}

#[async_trait::async_trait]
impl<P: beebotos_chain::compat::AlloyProvider + Clone + Send + Sync + 'static> ChainIdentityRegistry
    for ChainIdentityRegistryWrapper<P>
{
    async fn resolve_did(&self, did: &str) -> anyhow::Result<Option<Address>> {
        match self.registry.get_agent_id_by_did_cached(did).await {
            Ok(Some(agent_id)) => match self.registry.get_agent_cached(agent_id).await {
                Ok(agent_info) => Ok(Some(agent_info.owner)),
                Err(e) => {
                    warn!("Failed to get agent info for {}: {}", did, e);
                    Ok(None)
                }
            },
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("Failed to resolve DID {}: {}", did, e);
                Ok(None)
            }
        }
    }

    async fn get_reputation(&self, did: &str) -> anyhow::Result<Option<u64>> {
        match self.registry.get_agent_id_by_did_cached(did).await {
            Ok(Some(agent_id)) => match self.registry.get_agent_cached(agent_id).await {
                Ok(agent_info) => Ok(Some(agent_info.reputation.to::<u64>())),
                Err(e) => {
                    warn!("Failed to get reputation for {}: {}", did, e);
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }

    async fn is_registered(&self, did: &str) -> anyhow::Result<bool> {
        // Check if DID exists by looking up agent ID
        match self.registry.get_agent_id_by_did_cached(did).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
}

/// DID resolution error
#[derive(Debug, thiserror::Error)]
pub enum DIDResolutionError {
    #[error("DID not found: {0}")]
    NotFound(String),
    #[error("Invalid DID format: {0}")]
    InvalidFormat(String),
    #[error("Chain verification failed: {0}")]
    ChainVerificationFailed(String),
    #[error("Cache error: {0}")]
    CacheError(String),
}

impl DIDResolver {
    /// Create new resolver (local only, no chain integration)
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(std::collections::HashMap::new()),
            identity_registry: None,
            cache_ttl_secs: 300, // 5 minutes default
        }
    }

    /// Create new resolver with chain integration
    ///
    /// 🔒 P0 FIX: This enables on-chain identity verification
    pub fn with_chain_registry(registry: Arc<dyn ChainIdentityRegistry>) -> Self {
        Self {
            cache: RwLock::new(std::collections::HashMap::new()),
            identity_registry: Some(registry),
            cache_ttl_secs: 300,
        }
    }

    /// Set cache TTL
    pub fn with_cache_ttl(mut self, ttl_secs: u64) -> Self {
        self.cache_ttl_secs = ttl_secs;
        self
    }

    /// Resolve DID to document
    ///
    /// 🔒 P0 FIX: First checks on-chain registry if available, then falls back
    /// to local resolution
    pub async fn resolve(&self, did: &str) -> Result<DIDDocument, DIDResolutionError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(did) {
                if entry.timestamp.elapsed().as_secs() < self.cache_ttl_secs {
                    debug!("DID {} resolved from cache", did);
                    return Ok(entry.document.clone());
                }
            }
        }

        // Parse DID
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 || parts[0] != "did" {
            return Err(DIDResolutionError::InvalidFormat(did.to_string()));
        }

        let method = parts[1];
        let identifier = parts[2..].join(":");

        // Try on-chain resolution first if registry is available
        if let Some(registry) = &self.identity_registry {
            match self.resolve_on_chain(did, registry).await {
                Ok(Some(doc)) => {
                    // Cache and return
                    self.cache_did(did.to_string(), doc.clone()).await;
                    return Ok(doc);
                }
                Ok(None) => {
                    debug!("DID {} not found on chain, trying local resolution", did);
                }
                Err(e) => {
                    warn!("On-chain resolution failed for {}: {}", did, e);
                }
            }
        }

        // Fall back to local resolution
        let doc = match method {
            "beebot" => self.resolve_beebot(&identifier).await,
            "ethr" => self.resolve_ethr(&identifier).await,
            "key" => self.resolve_key(&identifier).await,
            _ => None,
        };

        match doc {
            Some(mut document) => {
                // If we have a chain registry, verify the DID
                if let Some(registry) = &self.identity_registry {
                    match registry.is_registered(did).await {
                        Ok(true) => {
                            document.on_chain_verified = true;
                            // Get additional on-chain info
                            if let Ok(Some(reputation)) = registry.get_reputation(did).await {
                                document.reputation = Some(reputation);
                            }
                            if let Ok(Some(address)) = registry.resolve_did(did).await {
                                document.agent_address = Some(format!("{:?}", address));
                            }
                        }
                        Ok(false) => {
                            document.on_chain_verified = false;
                        }
                        Err(e) => {
                            warn!("Failed to verify DID {} on chain: {}", did, e);
                            document.on_chain_verified = false;
                        }
                    }
                }

                // Cache result
                self.cache_did(did.to_string(), document.clone()).await;
                Ok(document)
            }
            None => Err(DIDResolutionError::NotFound(did.to_string())),
        }
    }

    /// Resolve DID using on-chain registry
    async fn resolve_on_chain(
        &self,
        did: &str,
        registry: &Arc<dyn ChainIdentityRegistry>,
    ) -> anyhow::Result<Option<DIDDocument>> {
        info!("Resolving DID {} on chain", did);

        if !registry.is_registered(did).await? {
            return Ok(None);
        }

        let address = registry.resolve_did(did).await?;
        let reputation = registry.get_reputation(did).await?;

        // Build DID document from on-chain data
        let doc = DIDDocument {
            context: vec!["https://www.w3.org/ns/did/v1".to_string()],
            id: did.to_string(),
            verification_method: vec![VerificationMethod {
                id: format!("{}#keys-1", did),
                vm_type: "EcdsaSecp256k1RecoveryMethod2020".to_string(),
                controller: did.to_string(),
                public_key_hex: address.as_ref().map(|a| format!("{:?}", a)),
                public_key_base58: None,
            }],
            authentication: vec![format!("{}#keys-1", did)],
            assertion_method: vec![format!("{}#keys-1", did)],
            service: vec![],
            on_chain_verified: true,
            agent_address: address.map(|a| format!("{:?}", a)),
            reputation,
        };

        info!("DID {} resolved successfully from chain", did);
        Ok(Some(doc))
    }

    /// Cache a resolved DID
    async fn cache_did(&self, did: String, document: DIDDocument) {
        let mut cache = self.cache.write().await;
        cache.insert(
            did,
            CachedDIDEntry {
                document,
                timestamp: std::time::Instant::now(),
            },
        );
    }

    /// Resolve beebot method
    async fn resolve_beebot(&self, identifier: &str) -> Option<DIDDocument> {
        Some(DIDDocument {
            context: vec!["https://www.w3.org/ns/did/v1".to_string()],
            id: format!("did:beebot:{}", identifier),
            verification_method: vec![VerificationMethod {
                id: format!("did:beebot:{}#keys-1", identifier),
                vm_type: "Ed25519VerificationKey2020".to_string(),
                controller: format!("did:beebot:{}", identifier),
                public_key_hex: Some(format!("0x{}", identifier)),
                public_key_base58: None,
            }],
            authentication: vec![format!("did:beebot:{}#keys-1", identifier)],
            assertion_method: vec![format!("did:beebot:{}#keys-1", identifier)],
            service: vec![],
            on_chain_verified: false,
            agent_address: None,
            reputation: None,
        })
    }

    /// Resolve ethr method
    async fn resolve_ethr(&self, identifier: &str) -> Option<DIDDocument> {
        Some(DIDDocument {
            context: vec!["https://www.w3.org/ns/did/v1".to_string()],
            id: format!("did:ethr:{}", identifier),
            verification_method: vec![VerificationMethod {
                id: format!("did:ethr:{}#owner", identifier),
                vm_type: "EcdsaSecp256k1RecoveryMethod2020".to_string(),
                controller: format!("did:ethr:{}", identifier),
                public_key_hex: Some(identifier.to_string()),
                public_key_base58: None,
            }],
            authentication: vec![format!("did:ethr:{}#owner", identifier)],
            assertion_method: vec![format!("did:ethr:{}#owner", identifier)],
            service: vec![],
            on_chain_verified: false,
            agent_address: Some(format!("0x{}", identifier.trim_start_matches("0x"))),
            reputation: None,
        })
    }

    /// Resolve key method
    async fn resolve_key(&self, identifier: &str) -> Option<DIDDocument> {
        Some(DIDDocument {
            context: vec!["https://www.w3.org/ns/did/v1".to_string()],
            id: format!("did:key:{}", identifier),
            verification_method: vec![VerificationMethod {
                id: format!("did:key:{}#keys-1", identifier),
                vm_type: "Ed25519VerificationKey2020".to_string(),
                controller: format!("did:key:{}", identifier),
                public_key_hex: None,
                public_key_base58: Some(identifier.to_string()),
            }],
            authentication: vec![format!("did:key:{}#keys-1", identifier)],
            assertion_method: vec![format!("did:key:{}#keys-1", identifier)],
            service: vec![],
            on_chain_verified: false,
            agent_address: None,
            reputation: None,
        })
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("DID resolver cache cleared");
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, u64) {
        let cache = self.cache.read().await;
        (cache.len(), self.cache_ttl_secs)
    }
}

impl Default for DIDResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_did_resolver_new() {
        let resolver = DIDResolver::new();
        let (size, ttl) = resolver.cache_stats().await;
        assert_eq!(size, 0);
        assert_eq!(ttl, 300);
    }

    #[tokio::test]
    async fn test_resolve_beebot_did() {
        let resolver = DIDResolver::new();
        let result = resolver.resolve("did:beebot:test123").await;
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.id, "did:beebot:test123");
        assert!(!doc.verification_method.is_empty());
    }

    #[tokio::test]
    async fn test_resolve_invalid_did() {
        let resolver = DIDResolver::new();
        let result = resolver.resolve("invalid:did").await;
        assert!(matches!(result, Err(DIDResolutionError::InvalidFormat(_))));
    }

    #[tokio::test]
    async fn test_cache_works() {
        let resolver = DIDResolver::new();

        // First resolution
        let _ = resolver.resolve("did:beebot:cached").await;
        let (size1, _) = resolver.cache_stats().await;
        assert_eq!(size1, 1);

        // Second resolution should hit cache
        let _ = resolver.resolve("did:beebot:cached").await;
        let (size2, _) = resolver.cache_stats().await;
        assert_eq!(size2, 1); // Still 1, not 2
    }
}
