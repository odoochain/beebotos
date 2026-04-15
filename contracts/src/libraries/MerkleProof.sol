// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title MerkleProof
 * @dev Library for verifying Merkle tree proofs
 */
library MerkleProof {
    /**
     * @dev Verifies a Merkle proof
     * @param root The Merkle root
     * @param leaf The leaf node to verify
     * @param proof The proof path
     * @return True if the proof is valid
     */
    function verify(
        bytes32 root,
        bytes32 leaf,
        bytes32[] memory proof
    ) internal pure returns (bool) {
        return processProof(proof, leaf) == root;
    }
    
    /**
     * @dev Processes a proof and returns the computed root
     */
    function processProof(
        bytes32[] memory proof,
        bytes32 leaf
    ) internal pure returns (bytes32) {
        bytes32 computedHash = leaf;
        for (uint256 i = 0; i < proof.length; i++) {
            computedHash = _hashPair(computedHash, proof[i]);
        }
        return computedHash;
    }
    
    /**
     * @dev Sorts and hashes two bytes32 values
     */
    function _hashPair(bytes32 a, bytes32 b) private pure returns (bytes32) {
        return a < b
            ? keccak256(abi.encodePacked(a, b))
            : keccak256(abi.encodePacked(b, a));
    }
    
    /**
     * @dev Multi-proof verification
     */
    function multiProofVerify(
        bytes32 root,
        bytes32[] memory leaves,
        bytes32[] memory proof,
        bool[] memory proofFlags
    ) internal pure returns (bool) {
        return processMultiProof(proof, proofFlags, leaves) == root;
    }
    
    /**
     * @dev Processes a multi-proof
     */
    function processMultiProof(
        bytes32[] memory proof,
        bool[] memory proofFlags,
        bytes32[] memory leaves
    ) internal pure returns (bytes32) {
        uint256 leavesLen = leaves.length;
        uint256 totalHashes = proofFlags.length;
        
        bytes32[] memory hashes = new bytes32[](totalHashes);
        uint256 leafPos = 0;
        uint256 hashPos = 0;
        uint256 proofPos = 0;
        
        for (uint256 i = 0; i < totalHashes; i++) {
            bytes32 a = leafPos < leavesLen
                ? leaves[leafPos++]
                : hashes[hashPos++];
            
            bytes32 b = proofFlags[i]
                ? (leafPos < leavesLen ? leaves[leafPos++] : hashes[hashPos++])
                : proof[proofPos++];
            
            hashes[i] = _hashPair(a, b);
        }
        
        return hashes[totalHashes - 1];
    }
}
