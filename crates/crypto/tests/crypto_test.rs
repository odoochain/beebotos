//! Crypto tests

use beebotos_crypto::hashing::{hash, HashAlgorithm};
use beebotos_crypto::signatures::*;

#[test]
fn test_ed25519_signer() {
    let signer = Ed25519Signer::new();

    let message = b"test message";
    let signature = signer.sign(message).unwrap();

    assert_eq!(signature.len(), 64);
}

#[test]
fn test_hash() {
    let data = b"test data";
    let hash1 = hash(HashAlgorithm::Blake3, data);
    let hash2 = hash(HashAlgorithm::Blake3, data);

    assert_eq!(hash1, hash2);
}
