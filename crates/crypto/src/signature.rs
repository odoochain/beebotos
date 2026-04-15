//! Digital Signatures

use crate::error::CryptoResult;
use crate::types::{Key32, Signature64};

/// Signer trait
pub trait Signer {
    fn sign(&self, message: &[u8]) -> CryptoResult<Signature64>;
    fn verify(&self, message: &[u8], signature: &Signature64) -> CryptoResult<bool>;
}

/// Ed25519 signer
pub struct Ed25519Signer {
    keypair: ed25519_dalek::Keypair,
}

impl Ed25519Signer {
    pub fn generate() -> Self {
        use rand::rngs::OsRng;
        let mut csprng = OsRng;
        Self {
            keypair: ed25519_dalek::Keypair::generate(&mut csprng),
        }
    }
}
