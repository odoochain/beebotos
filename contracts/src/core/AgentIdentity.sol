// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "../interfaces/IERC8004.sol";

/**
 * @title AgentIdentity
 * @notice Decentralized identity registry for AI agents with emergency pause
 */
contract AgentIdentity is IERC8004, OwnableUpgradeable, PausableUpgradeable, ReentrancyGuardUpgradeable, UUPSUpgradeable {
    
    struct Agent {
        bytes32 agentId;
        address owner;
        string did;
        bytes32 publicKey;
        bool isActive;
        uint256 reputation;
        uint256 createdAt;
        mapping(bytes32 => bool) capabilities;
    }
    
    mapping(bytes32 => Agent) public agents;
    mapping(address => bytes32[]) public ownerAgents;
    mapping(string => bytes32) public didToAgent;
    
    uint256 public totalAgents;
    bytes32[] public allAgentIds;
    
    // Authorized reputation updaters
    mapping(address => bool) public authorizedUpdaters;
    
    address private immutable __self;
    
    event UpdaterAuthorized(address indexed updater, bool authorized);
    
    constructor() {
        __self = address(this);
    }
    
    event AgentRegistered(bytes32 indexed agentId, address indexed owner, string did);
    event AgentUpdated(bytes32 indexed agentId, string field);
    event AgentDeactivated(bytes32 indexed agentId);
    event CapabilityGranted(bytes32 indexed agentId, bytes32 capability);
    event CapabilityRevoked(bytes32 indexed agentId, bytes32 capability);
    
    modifier onlyAgentOwner(bytes32 agentId) {
        require(agents[agentId].owner == msg.sender, "Not agent owner");
        _;
    }
    
    modifier onlyActiveAgent(bytes32 agentId) {
        require(agents[agentId].isActive, "Agent not active");
        _;
    }
    
    modifier onlyAuthorizedUpdater() {
        require(
            authorizedUpdaters[msg.sender] || msg.sender == owner(),
            "AgentIdentity: not authorized updater"
        );
        _;
    }
    
    function initialize() public initializer {
        __Ownable_init();
        __Pausable_init();
        __ReentrancyGuard_init();
        __UUPSUpgradeable_init();
    }
    
    /// @dev Counter for additional entropy in ID generation
    uint256 private _registrationNonce;
    
    /// @dev Previous block hash for additional entropy (if available)
    bytes32 private _prevBlockHash;
    
    /// @dev Last registration block number to prevent same-block manipulation
    uint256 private _lastRegistrationBlock;
    
    function registerAgent(string calldata did, bytes32 publicKey) 
        external 
        override 
        whenNotPaused 
        returns (bytes32) 
    {
        require(bytes(did).length > 0, "DID required");
        require(publicKey != bytes32(0), "Public key required");
        require(didToAgent[did] == bytes32(0), "DID already registered");
        
        // Generate agentId with multiple entropy sources
        // Uses: blockhash (if available), prevrandao, timestamp, sender nonce
        // Note: For higher security requirements, consider using Chainlink VRF
        _registrationNonce++;
        
        // Get previous block hash for additional entropy (only if not same block)
        bytes32 additionalEntropy = blockhash(block.number - 1);
        if (block.number == _lastRegistrationBlock || additionalEntropy == bytes32(0)) {
            // Fallback to prevrandao if blockhash unavailable or same block
            additionalEntropy = bytes32(block.prevrandao);
        }
        _lastRegistrationBlock = block.number;
        
        bytes32 agentId = keccak256(abi.encodePacked(
            did,
            msg.sender,
            publicKey,
            block.timestamp,
            block.number,
            block.prevrandao,
            additionalEntropy,
            _registrationNonce,
            address(this),
            gasleft()  // Add gasleft() for additional entropy
        ));
        
        // Ensure no collision (extremely unlikely with keccak256, but for safety)
        require(agents[agentId].agentId == bytes32(0), "AgentIdentity: ID collision");
        
        Agent storage agent = agents[agentId];
        agent.agentId = agentId;
        agent.owner = msg.sender;
        agent.did = did;
        agent.publicKey = publicKey;
        agent.isActive = true;
        agent.reputation = 100;
        agent.createdAt = block.timestamp;
        
        ownerAgents[msg.sender].push(agentId);
        didToAgent[did] = agentId;
        allAgentIds.push(agentId);
        totalAgents++;
        
        emit AgentRegistered(agentId, msg.sender, did);
        return agentId;
    }
    
    function getAgent(bytes32 agentId) 
        external 
        view 
        override 
        returns (AgentIdentity memory) 
    {
        Agent storage agent = agents[agentId];
        return AgentIdentity({
            agentId: agent.agentId,
            owner: agent.owner,
            did: agent.did,
            publicKey: agent.publicKey,
            isActive: agent.isActive,
            reputation: agent.reputation,
            createdAt: agent.createdAt
        });
    }
    
    function updateReputation(bytes32 agentId, uint256 newReputation) 
        external 
        override 
        onlyActiveAgent(agentId) 
        whenNotPaused 
        onlyAuthorizedUpdater
    {
        agents[agentId].reputation = newReputation;
        emit AgentUpdated(agentId, "reputation");
    }
    
    function setAuthorizedUpdater(address updater, bool authorized) external onlyOwner {
        require(updater != address(0), "AgentIdentity: zero address");
        authorizedUpdaters[updater] = authorized;
        emit UpdaterAuthorized(updater, authorized);
    }
    
    function deactivateAgent(bytes32 agentId) 
        external 
        onlyAgentOwner(agentId) 
        whenNotPaused 
    {
        agents[agentId].isActive = false;
        emit AgentDeactivated(agentId);
    }
    
    function grantCapability(bytes32 agentId, bytes32 capability) 
        external 
        onlyAgentOwner(agentId) 
        whenNotPaused 
    {
        agents[agentId].capabilities[capability] = true;
        emit CapabilityGranted(agentId, capability);
    }
    
    function revokeCapability(bytes32 agentId, bytes32 capability) 
        external 
        onlyAgentOwner(agentId) 
        whenNotPaused 
    {
        agents[agentId].capabilities[capability] = false;
        emit CapabilityRevoked(agentId, capability);
    }
    
    function hasCapability(bytes32 agentId, bytes32 capability) 
        external 
        view 
        returns (bool) 
    {
        return agents[agentId].capabilities[capability];
    }
    
    function getOwnerAgents(address owner) external view returns (bytes32[] memory) {
        return ownerAgents[owner];
    }
    
    // Emergency pause functions
    function pause() external onlyOwner {
        _pause();
    }
    
    function unpause() external onlyOwner {
        _unpause();
    }
    
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
    
    // Storage gap for upgrade safety
    uint256[50] private __gap;
}
