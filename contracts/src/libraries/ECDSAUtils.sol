// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title ECDSAUtils
 * @notice Utility functions for ECDSA signatures
 */
library ECDSAUtils {
    
    function recoverSigner(bytes32 messageHash, bytes memory signature) internal pure returns (address) {
        require(signature.length == 65, "Invalid signature length");
        
        bytes32 r;
        bytes32 s;
        uint8 v;
        
        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }
        
        return ecrecover(messageHash, v, r, s);
    }
    
    function verifySignature(bytes32 messageHash, bytes memory signature, address expectedSigner) internal pure returns (bool) {
        return recoverSigner(messageHash, signature) == expectedSigner;
    }
    
    function toEthSignedMessageHash(bytes32 messageHash) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash));
    }
}
