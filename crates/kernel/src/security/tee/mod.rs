//! Trusted Execution Environment (TEE)
//!
//! Hardware-based secure execution support for BeeBotOS Kernel.
//!
//! ## Supported TEE Platforms
//!
//! - **Intel SGX** - Intel Software Guard Extensions
//! - **AMD SEV** - AMD Secure Encrypted Virtualization
//! - **AWS Nitro Enclaves** - AWS cloud-based TEE
//! - **Simulation** - Software simulation for testing
//!
//! ## Example Usage
//!
//! ```rust
//! use beebotos_kernel::security::tee::{TeeEnclave, TeeProviderFactory};
//!
//! // Detect available TEE and create provider
//! let provider = TeeProviderFactory::detect_best_available().expect("No TEE available");
//!
//! // Create and initialize enclave
//! let mut enclave = TeeEnclave::new(provider).expect("Failed to create enclave");
//! enclave.initialize().expect("Failed to initialize");
//!
//! // Get attestation report
//! let report = enclave.attest(None).expect("Attestation failed");
//! ```

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod nitro;
mod provider;
mod sev;
mod sgx;
mod simulation;

pub use nitro::NitroProvider;
pub use provider::{TeeCapabilities, TeeMeasurement, TeeProvider, TeeProviderFactory};
pub use sev::SevProvider;
pub use sgx::SgxProvider;
pub use simulation::SimulationProvider;

/// TEE provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TeeProviderType {
    /// Intel SGX
    Sgx,
    /// AMD SEV
    Sev,
    /// ARM TrustZone
    TrustZone,
    /// AWS Nitro Enclaves
    Nitro,
    /// Azure Confidential Computing
    AzureCC,
    /// Google Confidential VMs
    GoogleCC,
    /// Software simulation (testing)
    Simulation,
}

impl TeeProviderType {
    /// Check if TEE is available on this system
    pub fn is_available(&self) -> bool {
        match self {
            TeeProviderType::Sgx => sgx::is_available(),
            TeeProviderType::Sev => sev::is_available(),
            TeeProviderType::TrustZone => false, // TODO: ARM TrustZone detection
            TeeProviderType::Nitro => nitro::is_available(),
            TeeProviderType::AzureCC => std::env::var("AZURE_CC").is_ok(),
            TeeProviderType::GoogleCC => std::env::var("GOOGLE_CLOUD_VM").is_ok(),
            TeeProviderType::Simulation => true,
        }
    }

    /// Get provider priority (higher = preferred)
    pub fn priority(&self) -> u32 {
        match self {
            // Hardware TEEs have higher priority
            TeeProviderType::Sgx => 100,
            TeeProviderType::Sev => 100,
            TeeProviderType::Nitro => 90,
            TeeProviderType::TrustZone => 80,
            TeeProviderType::AzureCC => 70,
            TeeProviderType::GoogleCC => 70,
            // Simulation is lowest priority
            TeeProviderType::Simulation => 10,
        }
    }

    /// Get attestation capabilities for this provider type
    pub fn attestation_capabilities(&self) -> AttestationCapabilities {
        match self {
            TeeProviderType::Sgx | TeeProviderType::Sev | TeeProviderType::Nitro => {
                AttestationCapabilities {
                    remote_attestation: true,
                    local_attestation: true,
                    custom_claims: true,
                }
            }
            TeeProviderType::TrustZone => AttestationCapabilities {
                remote_attestation: false,
                local_attestation: true,
                custom_claims: false,
            },
            _ => AttestationCapabilities {
                remote_attestation: false,
                local_attestation: false,
                custom_claims: false,
            },
        }
    }
}

/// Attestation capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttestationCapabilities {
    /// Whether remote attestation is supported
    pub remote_attestation: bool,
    /// Whether local attestation is supported
    pub local_attestation: bool,
    /// Whether custom claims are supported
    pub custom_claims: bool,
}

/// TEE attestation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationReport {
    /// TEE provider type
    pub provider_type: TeeProviderType,
    /// Attestation quote data
    pub quote: Vec<u8>,
    /// Enclave measurement
    pub measurement: TeeMeasurement,
    /// Report timestamp
    pub timestamp: u64,
    /// Attestation claims
    pub claims: Vec<AttestationClaim>,
    /// Platform-specific additional data
    pub platform_data: Option<Vec<u8>>,
}

/// Attestation claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationClaim {
    /// Claim key
    pub key: String,
    /// Claim value
    pub value: String,
}

/// Sealed data header (reserved for future use)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct SealedDataHeader {
    pub version: u32,
    pub provider_type: TeeProviderType,
    pub key_id: [u8; 32],
    pub data_len: u64,
}

/// TEE enclave - main interface for TEE operations
pub struct TeeEnclave {
    provider: Arc<dyn TeeProvider>,
    provider_type: TeeProviderType,
    initialized: bool,
    #[allow(dead_code)]
    config: EnclaveConfig,
}

/// Enclave configuration
#[derive(Debug, Clone)]
pub struct EnclaveConfig {
    /// Debug mode flag
    pub debug_mode: bool,
    /// Memory size in bytes (None for default)
    pub memory_size: Option<usize>,
    /// Thread count (None for default)
    pub thread_count: Option<usize>,
}

impl Default for EnclaveConfig {
    fn default() -> Self {
        Self {
            debug_mode: false,
            memory_size: None,
            thread_count: None,
        }
    }
}

impl TeeEnclave {
    /// Create new enclave with default configuration
    pub fn new(provider_type: TeeProviderType) -> Result<Self, TeeError> {
        Self::with_config(provider_type, EnclaveConfig::default())
    }

    /// Create new enclave with custom configuration
    pub fn with_config(
        provider_type: TeeProviderType,
        config: EnclaveConfig,
    ) -> Result<Self, TeeError> {
        if !provider_type.is_available() {
            return Err(TeeError::NotAvailable(provider_type));
        }

        let provider: Arc<dyn TeeProvider> = match provider_type {
            TeeProviderType::Sgx => Arc::new(SgxProvider::new(&config)?),
            TeeProviderType::Sev => Arc::new(SevProvider::new(&config)?),
            TeeProviderType::Nitro => Arc::new(NitroProvider::new(&config)?),
            TeeProviderType::Simulation => Arc::new(SimulationProvider::new(&config)?),
            _ => return Err(TeeError::UnsupportedProvider(provider_type)),
        };

        Ok(Self {
            provider,
            provider_type,
            initialized: false,
            config,
        })
    }

    /// Initialize the enclave
    pub fn initialize(&mut self) -> Result<(), TeeError> {
        if self.initialized {
            return Err(TeeError::AlreadyInitialized);
        }

        tracing::info!("Initializing {:?} TEE enclave", self.provider_type);

        self.provider.initialize()?;
        self.initialized = true;

        tracing::info!(
            "{:?} TEE enclave initialized successfully",
            self.provider_type
        );
        Ok(())
    }

    /// Check if enclave is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the provider type
    pub fn provider_type(&self) -> TeeProviderType {
        self.provider_type
    }

    /// Get provider capabilities
    pub fn capabilities(&self) -> TeeCapabilities {
        self.provider.capabilities()
    }

    /// Get attestation report
    pub fn attest(&self, user_data: Option<&[u8]>) -> Result<AttestationReport, TeeError> {
        if !self.initialized {
            return Err(TeeError::NotInitialized);
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let measurement = self.provider.get_measurement()?;
        let quote = self.provider.generate_quote(user_data)?;

        let claims = vec![
            AttestationClaim {
                key: "enclave_measurement".to_string(),
                value: hex::encode(measurement.hash),
            },
            AttestationClaim {
                key: "provider".to_string(),
                value: format!("{:?}", self.provider_type),
            },
        ];

        Ok(AttestationReport {
            provider_type: self.provider_type,
            quote,
            measurement,
            timestamp,
            claims,
            platform_data: None,
        })
    }

    /// Verify remote attestation report
    pub fn verify_attestation(
        &self,
        report: &AttestationReport,
    ) -> Result<AttestationVerification, TeeError> {
        if !self.initialized {
            return Err(TeeError::NotInitialized);
        }

        tracing::info!("Verifying attestation from {:?}", report.provider_type);

        self.provider.verify_quote(&report.quote)
    }

    /// Seal data for enclave-only access
    ///
    /// The sealed data can only be unsealed by the same TEE on the same
    /// platform
    pub fn seal(&self, data: &[u8]) -> Result<Vec<u8>, TeeError> {
        if !self.initialized {
            return Err(TeeError::NotInitialized);
        }

        if data.is_empty() {
            return Err(TeeError::InvalidData(
                "Empty data cannot be sealed".to_string(),
            ));
        }

        self.provider.seal_data(data)
    }

    /// Unseal data previously sealed by this or compatible enclave
    pub fn unseal(&self, sealed: &[u8]) -> Result<Vec<u8>, TeeError> {
        if !self.initialized {
            return Err(TeeError::NotInitialized);
        }

        if sealed.len() < 32 {
            return Err(TeeError::InvalidData(
                "Invalid sealed data format".to_string(),
            ));
        }

        self.provider.unseal_data(sealed)
    }

    /// Execute code inside the enclave
    ///
    /// # Safety
    /// This is unsafe because it executes arbitrary code
    pub unsafe fn execute(&self, code: &[u8], input: &[u8]) -> Result<Vec<u8>, TeeError> {
        if !self.initialized {
            return Err(TeeError::NotInitialized);
        }

        self.provider.execute(code, input)
    }

    /// Get the underlying provider for advanced operations
    pub fn provider(&self) -> &dyn TeeProvider {
        self.provider.as_ref()
    }

    /// Shutdown the enclave
    pub fn shutdown(&mut self) -> Result<(), TeeError> {
        if !self.initialized {
            return Ok(());
        }

        tracing::info!("Shutting down {:?} TEE enclave", self.provider_type);
        self.provider.shutdown()?;
        self.initialized = false;
        Ok(())
    }
}

impl Drop for TeeEnclave {
    fn drop(&mut self) {
        if self.initialized {
            let _ = self.shutdown();
        }
    }
}

/// Attestation verification result
#[derive(Debug, Clone)]
pub struct AttestationVerification {
    /// Whether the attestation is valid
    pub valid: bool,
    /// Whether the measurement matches expected value
    pub measurement_matches: bool,
    /// Whether the timestamp is valid
    pub timestamp_valid: bool,
    /// Verification details
    pub details: String,
}

/// TEE errors
#[derive(Debug, Error, Clone)]
pub enum TeeError {
    /// TEE provider not available
    #[error("TEE provider {0:?} is not available on this system")]
    NotAvailable(TeeProviderType),

    /// TEE not initialized
    #[error("TEE enclave not initialized")]
    NotInitialized,

    /// TEE already initialized
    #[error("TEE enclave already initialized")]
    AlreadyInitialized,

    /// Unsupported TEE provider
    #[error("TEE provider {0:?} is not supported")]
    UnsupportedProvider(TeeProviderType),

    /// Initialization failed
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    /// Attestation failed
    #[error("Attestation failed: {0}")]
    AttestationFailed(String),

    /// Quote verification failed
    #[error("Quote verification failed: {0}")]
    VerificationFailed(String),

    /// Sealing failed
    #[error("Sealing failed: {0}")]
    SealingFailed(String),

    /// Unsealing failed
    #[error("Unsealing failed: {0}")]
    UnsealingFailed(String),

    /// Invalid data
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Platform error
    #[error("Platform error: {0}")]
    PlatformError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Helper function to get current timestamp
#[allow(dead_code)]
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_provider_type_priority() {
        assert!(TeeProviderType::Sgx.priority() > TeeProviderType::Simulation.priority());
        assert!(TeeProviderType::Sev.priority() > TeeProviderType::Simulation.priority());
    }

    #[test]
    fn test_attestation_capabilities() {
        let caps = TeeProviderType::Sgx.attestation_capabilities();
        assert!(caps.remote_attestation);
        assert!(caps.local_attestation);

        let sim_caps = TeeProviderType::Simulation.attestation_capabilities();
        assert!(!sim_caps.remote_attestation);
    }

    #[test]
    fn test_enclave_config_default() {
        let config = EnclaveConfig::default();
        assert!(!config.debug_mode);
        assert!(config.memory_size.is_none());
        assert!(config.thread_count.is_none());
    }
}
