//! Merkle Tree

use crate::types::HashOutput;

/// Merkle tree
pub struct MerkleTree {
    leaves: Vec<HashOutput>,
    root: HashOutput,
}

impl MerkleTree {
    pub fn new(leaves: Vec<HashOutput>) -> Self {
        let root = Self::compute_root(&leaves);
        Self { leaves, root }
    }

    fn compute_root(leaves: &[HashOutput]) -> HashOutput {
        // Simplified root computation
        leaves.first().copied().unwrap_or_default()
    }

    pub fn root(&self) -> &HashOutput {
        &self.root
    }

    pub fn proof(&self, index: usize) -> Option<MerkleProof> {
        if index < self.leaves.len() {
            Some(MerkleProof { path: vec![] })
        } else {
            None
        }
    }
}

/// Merkle proof
pub struct MerkleProof {
    pub path: Vec<HashOutput>,
}
