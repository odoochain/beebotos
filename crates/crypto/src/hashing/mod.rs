//! Cryptographic Hashing
//!
//! BLAKE3, SHA-3, Keccak, and Poseidon (ZK-friendly).

use blake3::Hasher as Blake3Hasher;
use sha3::{Digest, Sha3_256};

/// Hash types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Blake3,
    Sha3_256,
    Keccak256,
    Poseidon, // ZK-friendly
}

/// Hash result (32 bytes)
pub type Hash = [u8; 32];

/// Hash a message
pub fn hash(algorithm: HashAlgorithm, message: &[u8]) -> Hash {
    match algorithm {
        HashAlgorithm::Blake3 => blake3_hash(message),
        HashAlgorithm::Sha3_256 => sha3_256_hash(message),
        HashAlgorithm::Keccak256 => keccak256_hash(message),
        HashAlgorithm::Poseidon => poseidon_hash(message),
    }
}

/// BLAKE3 hash
pub fn blake3_hash(message: &[u8]) -> Hash {
    let mut hasher = Blake3Hasher::new();
    hasher.update(message);
    let result = hasher.finalize();
    result.into()
}

/// SHA3-256 hash
pub fn sha3_256_hash(message: &[u8]) -> Hash {
    let mut hasher = Sha3_256::new();
    hasher.update(message);
    let result = hasher.finalize();
    result.into()
}

/// Keccak-256 hash (Ethereum compatible)
pub fn keccak256_hash(message: &[u8]) -> Hash {
    use tiny_keccak::{Hasher, Keccak};

    let mut hasher = Keccak::v256();
    let mut result = [0u8; 32];
    hasher.update(message);
    hasher.finalize(&mut result);
    result
}

/// Poseidon hash (ZK-friendly)
pub fn poseidon_hash(_message: &[u8]) -> Hash {
    // Placeholder for Poseidon implementation
    // In production, use a proper Poseidon library
    [0u8; 32]
}

/// Merkle tree
#[derive(Debug, Clone)]
pub struct MerkleTree {
    leaves: Vec<Hash>,
    layers: Vec<Vec<Hash>>,
}

impl MerkleTree {
    /// Create new Merkle tree
    pub fn new(leaves: Vec<Hash>) -> Self {
        let mut tree = Self {
            leaves: leaves.clone(),
            layers: vec![leaves],
        };
        tree.build();
        tree
    }

    /// Build tree layers
    fn build(&mut self) {
        let mut current_layer = self.leaves.clone();

        while current_layer.len() > 1 {
            let mut next_layer = Vec::new();

            for chunk in current_layer.chunks(2) {
                let combined = if chunk.len() == 2 {
                    let mut combined = chunk[0].to_vec();
                    combined.extend_from_slice(&chunk[1]);
                    blake3_hash(&combined)
                } else {
                    chunk[0]
                };
                next_layer.push(combined);
            }

            self.layers.push(next_layer.clone());
            current_layer = next_layer;
        }
    }

    /// Get root hash
    pub fn root(&self) -> Option<Hash> {
        self.layers.last()?.first().copied()
    }

    /// Generate proof for leaf
    pub fn proof(&self, index: usize) -> Option<MerkleProof> {
        let mut proof = Vec::new();
        let mut current_index = index;

        for layer in &self.layers[..self.layers.len() - 1] {
            let sibling_index = if current_index.is_multiple_of(2) {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < layer.len() {
                proof.push(MerkleProofElement {
                    hash: layer[sibling_index],
                    is_left: sibling_index < current_index,
                });
            }

            current_index /= 2;
        }

        Some(MerkleProof { elements: proof })
    }

    /// Verify proof
    pub fn verify_proof(&self, leaf: Hash, proof: &MerkleProof) -> bool {
        let mut current = leaf;

        for element in &proof.elements {
            let combined = if element.is_left {
                let mut c = element.hash.to_vec();
                c.extend_from_slice(&current);
                c
            } else {
                let mut c = current.to_vec();
                c.extend_from_slice(&element.hash);
                c
            };

            current = blake3_hash(&combined);
        }

        self.root() == Some(current)
    }
}

/// Merkle proof
#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub elements: Vec<MerkleProofElement>,
}

/// Merkle proof element
#[derive(Debug, Clone)]
pub struct MerkleProofElement {
    pub hash: Hash,
    pub is_left: bool,
}
