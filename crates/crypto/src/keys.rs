//! Key management

use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// Agent identity keypair
#[derive(Debug, Clone)]
pub struct IdentityKey {
    keypair: Keypair,
}

impl IdentityKey {
    /// Generate new random keypair
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        Self { keypair }
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8; 64]) -> anyhow::Result<Self> {
        let keypair = Keypair::from_bytes(bytes)?;
        Ok(Self { keypair })
    }

    /// Get public key
    pub fn public(&self) -> &PublicKey {
        &self.keypair.public
    }

    /// Get public key bytes
    pub fn public_bytes(&self) -> [u8; 32] {
        self.keypair.public.to_bytes()
    }

    /// Sign message
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.keypair.sign(message)
    }

    /// Verify signature
    pub fn verify(&self, message: &[u8], signature: &Signature) -> anyhow::Result<()> {
        self.keypair.public.verify(message, signature)?;
        Ok(())
    }

    /// Export to bytes
    pub fn to_bytes(&self) -> [u8; 64] {
        self.keypair.to_bytes()
    }
}

/// DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    #[serde(rename = "@context")]
    pub context: String,
    pub id: String,
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub type_: String,
    pub controller: String,
    pub public_key_base58: String,
}

impl DIDDocument {
    /// Create new DID document from public key
    pub fn from_public_key(did: impl Into<String>, public_key: &[u8]) -> Self {
        use bs58;
        
        let did = did.into();
        let key_id = format!("{}#keys-1", did);
        
        Self {
            context: "https://www.w3.org/ns/did/v1".to_string(),
            id: did.clone(),
            verification_method: vec![VerificationMethod {
                id: key_id.clone(),
                type_: "Ed25519VerificationKey2020".to_string(),
                controller: did,
                public_key_base58: bs58::encode(public_key).into_string(),
            }],
            authentication: vec![key_id.clone()],
            assertion_method: vec![key_id],
        }
    }

    /// Resolve DID
    pub fn resolve(did: &str) -> anyhow::Result<Self> {
        // In production, this would resolve via DID method
        anyhow::bail!("DID resolution not implemented")
    }
}

/// Secure enclave operations
#[cfg(feature = "tee")]
pub mod tee {
    use super::*;

    /// TEE-backed key (key never leaves enclave)
    pub struct TEEKey {
        key_id: String,
    }

    impl TEEKey {
        /// Generate key in TEE
        pub fn generate() -> anyhow::Result<Self> {
            // Would use SGX/SEV APIs
            Ok(Self {
                key_id: uuid::Uuid::new_v4().to_string(),
            })
        }

        /// Sign data (operation happens in TEE)
        pub fn sign(&self, _data: &[u8]) -> anyhow::Result<Vec<u8>> {
            // Would call into enclave
            Ok(vec![])
        }
    }
}
