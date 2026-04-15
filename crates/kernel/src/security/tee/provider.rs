//! TEE Provider Trait and Factory
//!
//! Defines the interface for all TEE implementations and provides
//! factory methods for creating provider instances.

use serde::{Deserialize, Serialize};

use super::{AttestationVerification, EnclaveConfig, TeeError, TeeProviderType};

/// TEE measurement (enclave hash/measurement)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeeMeasurement {
    /// The measurement hash (typically SHA-256)
    pub hash: [u8; 32],
    /// Measurement version/format
    pub version: u32,
}

impl Default for TeeMeasurement {
    fn default() -> Self {
        Self {
            hash: [0u8; 32],
            version: 1,
        }
    }
}

impl TeeMeasurement {
    /// Create a new measurement from hash bytes
    pub fn new(hash: [u8; 32]) -> Self {
        Self { hash, version: 1 }
    }

    /// Convert measurement to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.hash)
    }
}

/// TEE capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeeCapabilities {
    /// Supports remote attestation
    pub remote_attestation: bool,
    /// Supports local attestation
    pub local_attestation: bool,
    /// Supports data sealing
    pub sealing: bool,
    /// Supports secure execution
    pub secure_execution: bool,
    /// Maximum enclave memory size
    pub max_memory_size: usize,
    /// Maximum number of threads
    pub max_threads: usize,
    /// Platform version
    pub platform_version: u32,
}

impl Default for TeeCapabilities {
    fn default() -> Self {
        Self {
            remote_attestation: false,
            local_attestation: false,
            sealing: true,
            secure_execution: true,
            max_memory_size: 128 * 1024 * 1024, // 128 MB
            max_threads: 4,
            platform_version: 1,
        }
    }
}

/// Core trait for TEE providers
///
/// All TEE implementations (SGX, SEV, Nitro, etc.) must implement this trait.
pub trait TeeProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> TeeProviderType;

    /// Initialize the TEE provider
    fn initialize(&self) -> Result<(), TeeError>;

    /// Shutdown the TEE provider
    fn shutdown(&self) -> Result<(), TeeError>;

    /// Get provider capabilities
    fn capabilities(&self) -> TeeCapabilities;

    /// Get the current measurement of the enclave
    fn get_measurement(&self) -> Result<TeeMeasurement, TeeError>;

    /// Generate an attestation quote with optional user data
    ///
    /// The quote is a cryptographic proof of the enclave's identity
    fn generate_quote(&self, user_data: Option<&[u8]>) -> Result<Vec<u8>, TeeError>;

    /// Verify an attestation quote
    fn verify_quote(&self, quote: &[u8]) -> Result<AttestationVerification, TeeError>;

    /// Seal data to the current enclave
    ///
    /// Sealed data can only be unsealed by enclaves with the same identity
    fn seal_data(&self, data: &[u8]) -> Result<Vec<u8>, TeeError>;

    /// Unseal previously sealed data
    fn unseal_data(&self, sealed: &[u8]) -> Result<Vec<u8>, TeeError>;

    /// Execute code inside the enclave
    ///
    /// # Safety
    /// This executes arbitrary code and is inherently unsafe
    unsafe fn execute(&self, code: &[u8], input: &[u8]) -> Result<Vec<u8>, TeeError>;

    /// Get platform-specific data
    fn get_platform_data(&self) -> Result<Vec<u8>, TeeError>;
}

/// Factory for creating TEE providers
pub struct TeeProviderFactory;

impl TeeProviderFactory {
    /// Create a provider of the specified type
    pub fn create(
        provider_type: TeeProviderType,
        config: &EnclaveConfig,
    ) -> Result<Box<dyn TeeProvider>, TeeError> {
        use super::{NitroProvider, SevProvider, SgxProvider, SimulationProvider};

        if !provider_type.is_available() {
            return Err(TeeError::NotAvailable(provider_type));
        }

        let provider: Box<dyn TeeProvider> = match provider_type {
            TeeProviderType::Sgx => Box::new(SgxProvider::new(config)?),
            TeeProviderType::Sev => Box::new(SevProvider::new(config)?),
            TeeProviderType::Nitro => Box::new(NitroProvider::new(config)?),
            TeeProviderType::Simulation => Box::new(SimulationProvider::new(config)?),
            _ => return Err(TeeError::UnsupportedProvider(provider_type)),
        };

        Ok(provider)
    }

    /// Detect the best available TEE provider
    ///
    /// Returns the highest priority available provider
    pub fn detect_best_available() -> Option<TeeProviderType> {
        let mut available: Vec<(TeeProviderType, u32)> = [
            TeeProviderType::Sgx,
            TeeProviderType::Sev,
            TeeProviderType::Nitro,
            TeeProviderType::TrustZone,
            TeeProviderType::AzureCC,
            TeeProviderType::GoogleCC,
            TeeProviderType::Simulation,
        ]
        .iter()
        .filter(|t| t.is_available())
        .map(|t| (*t, t.priority()))
        .collect();

        // Sort by priority (highest first)
        available.sort_by(|a, b| b.1.cmp(&a.1));

        available.first().map(|(t, _)| *t)
    }

    /// List all available TEE providers
    pub fn list_available() -> Vec<(TeeProviderType, TeeCapabilities)> {
        let providers = [
            TeeProviderType::Sgx,
            TeeProviderType::Sev,
            TeeProviderType::Nitro,
            TeeProviderType::TrustZone,
            TeeProviderType::AzureCC,
            TeeProviderType::GoogleCC,
            TeeProviderType::Simulation,
        ];

        providers
            .iter()
            .filter(|t| t.is_available())
            .filter_map(|t| {
                // Create temporary provider to get capabilities
                let config = EnclaveConfig::default();
                match Self::create(*t, &config) {
                    Ok(provider) => {
                        let caps = provider.capabilities();
                        let _ = provider.shutdown();
                        Some((*t, caps))
                    }
                    Err(_) => None,
                }
            })
            .collect()
    }

    /// Check if any TEE is available
    pub fn any_available() -> bool {
        Self::detect_best_available().is_some()
    }
}

/// Utility functions for TEE providers
pub mod utils {
    use sha2::{Digest, Sha256};

    use super::*;

    /// Compute a measurement hash from code/data
    pub fn compute_measurement(data: &[u8]) -> TeeMeasurement {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);

        TeeMeasurement::new(hash)
    }

    /// Combine multiple measurements into one
    #[allow(dead_code)]
    pub fn combine_measurements(measurements: &[TeeMeasurement]) -> TeeMeasurement {
        let mut hasher = Sha256::new();
        for m in measurements {
            hasher.update(&m.hash);
        }
        let result = hasher.finalize();

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);

        TeeMeasurement::new(hash)
    }

    /// Generate a random key ID
    pub fn generate_key_id() -> [u8; 32] {
        use rand::Rng;
        let mut key_id = [0u8; 32];
        rand::thread_rng().fill(&mut key_id);
        key_id
    }

    /// Simple XOR-based obfuscation for simulation/testing
    /// NOT SECURE - for testing only
    pub fn xor_obfuscate(data: &[u8], key: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, b)| b ^ key[i % key.len()])
            .collect()
    }

    /// Verify measurement matches expected value
    #[allow(dead_code)]
    pub fn verify_measurement(
        actual: &TeeMeasurement,
        expected: &TeeMeasurement,
    ) -> Result<(), TeeError> {
        if actual.hash != expected.hash {
            return Err(TeeError::VerificationFailed(format!(
                "Measurement mismatch: expected {}, got {}",
                expected.to_hex(),
                actual.to_hex()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;
    use super::*;

    #[test]
    fn test_measurement_computation() {
        let data = b"test data";
        let measurement = compute_measurement(data);
        assert_ne!(measurement.hash, [0u8; 32]);
    }

    #[test]
    fn test_combine_measurements() {
        let m1 = compute_measurement(b"data1");
        let m2 = compute_measurement(b"data2");
        let combined = combine_measurements(&[m1, m2]);
        assert_ne!(combined.hash, [0u8; 32]);
    }

    #[test]
    fn test_verify_measurement() {
        let data = b"test data";
        let measurement = compute_measurement(data);

        // Should succeed with same measurement
        assert!(verify_measurement(&measurement, &measurement).is_ok());

        // Should fail with different measurement
        let other = compute_measurement(b"other data");
        assert!(verify_measurement(&measurement, &other).is_err());
    }

    #[test]
    fn test_xor_obfuscate() {
        let data = b"hello world";
        let key = b"key";
        let obfuscated = xor_obfuscate(data, key);
        let deobfuscated = xor_obfuscate(&obfuscated, key);
        assert_eq!(data.to_vec(), deobfuscated);
    }

    #[test]
    fn test_generate_key_id() {
        let id1 = generate_key_id();
        let id2 = generate_key_id();
        assert_ne!(id1, id2);
    }
}
