//! Hashing

use crate::types::HashOutput;

/// Hash function trait
pub trait Hasher {
    fn hash(data: &[u8]) -> HashOutput;
}

/// Blake3 hasher
pub struct Blake3Hasher;

impl Hasher for Blake3Hasher {
    fn hash(data: &[u8]) -> HashOutput {
        blake3::hash(data).into()
    }
}

/// SHA3-256 hasher
pub struct Sha3Hasher;

impl Hasher for Sha3Hasher {
    fn hash(data: &[u8]) -> HashOutput {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}
