// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IERC8004 {
    struct AgentIdentity {
        bytes32 agentId;
        address owner;
        string did;
        bytes32 publicKey;
        bool isActive;
        uint256 reputation;
        uint256 createdAt;
    }
    
    function registerAgent(string calldata did, bytes32 publicKey) external returns (bytes32);
    function getAgent(bytes32 agentId) external view returns (AgentIdentity memory);
    function updateReputation(bytes32 agentId, uint256 newReputation) external;
}
