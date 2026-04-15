// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "./AgentIdentity.sol";

/**
 * @title AgentRegistry
 * @notice Registry for agent metadata and service endpoints with pause and upgrade protection
 */
contract AgentRegistry is OwnableUpgradeable, PausableUpgradeable, ReentrancyGuardUpgradeable, UUPSUpgradeable {
    
    struct AgentMetadata {
        bytes32 agentId;
        string name;
        string description;
        string[] capabilities;
        string endpoint;
        uint256 version;
        bool isAvailable;
        uint256 lastHeartbeat;
    }
    
    AgentIdentity public identityContract;
    mapping(bytes32 => AgentMetadata) public metadata;
    mapping(string => bytes32[]) public capabilityIndex;
    bytes32[] public availableAgents;
    mapping(bytes32 => uint256) public agentIndex; // Track index in availableAgents for removal
    
    address private immutable __self;
    
    uint256 public constant HEARTBEAT_TIMEOUT = 1 hours;
    
    constructor() {
        __self = address(this);
    }
    
    event MetadataUpdated(bytes32 indexed agentId, string name);
    event Heartbeat(bytes32 indexed agentId, uint256 timestamp);
    event AvailabilityChanged(bytes32 indexed agentId, bool isAvailable);
    event AgentRemoved(bytes32 indexed agentId);
    
    function initialize(address identityAddress) public initializer {
        require(identityAddress != address(0), "AgentRegistry: zero address");
        
        __Ownable_init();
        __Pausable_init();
        __ReentrancyGuard_init();
        __UUPSUpgradeable_init();
        identityContract = AgentIdentity(identityAddress);
    }
    
    function registerMetadata(
        bytes32 agentId,
        string calldata name,
        string calldata description,
        string[] calldata capabilities,
        string calldata endpoint
    ) external whenNotPaused {
        AgentIdentity.AgentIdentity memory identity = identityContract.getAgent(agentId);
        require(identity.owner == msg.sender, "Not owner");
        require(identity.isActive, "Agent not active");
        require(bytes(name).length > 0, "Name required");
        require(bytes(endpoint).length > 0, "Endpoint required");
        
        AgentMetadata storage meta = metadata[agentId];
        meta.agentId = agentId;
        meta.name = name;
        meta.description = description;
        meta.capabilities = capabilities;
        meta.endpoint = endpoint;
        meta.version = 1;
        meta.isAvailable = true;
        meta.lastHeartbeat = block.timestamp;
        
        // Track index for removal
        agentIndex[agentId] = availableAgents.length;
        availableAgents.push(agentId);
        
        for (uint i = 0; i < capabilities.length; i++) {
            capabilityIndex[capabilities[i]].push(agentId);
        }
        
        emit MetadataUpdated(agentId, name);
    }
    
    function updateMetadata(
        bytes32 agentId,
        string calldata name,
        string calldata description,
        string[] calldata capabilities,
        string calldata endpoint
    ) external whenNotPaused {
        AgentMetadata storage meta = metadata[agentId];
        require(meta.agentId != bytes32(0), "Agent not registered");
        
        AgentIdentity.AgentIdentity memory identity = identityContract.getAgent(agentId);
        require(identity.owner == msg.sender, "Not owner");
        
        meta.name = name;
        meta.description = description;
        meta.capabilities = capabilities;
        meta.endpoint = endpoint;
        meta.version++;
        
        emit MetadataUpdated(agentId, name);
    }
    
    function heartbeat(bytes32 agentId) external whenNotPaused {
        require(metadata[agentId].agentId != bytes32(0), "Agent not registered");
        metadata[agentId].lastHeartbeat = block.timestamp;
        emit Heartbeat(agentId, block.timestamp);
    }
    
    function setAvailability(bytes32 agentId, bool isAvailable) external whenNotPaused {
        require(metadata[agentId].agentId != bytes32(0), "Agent not registered");
        
        AgentIdentity.AgentIdentity memory identity = identityContract.getAgent(agentId);
        require(identity.owner == msg.sender || msg.sender == owner(), "Not authorized");
        
        metadata[agentId].isAvailable = isAvailable;
        emit AvailabilityChanged(agentId, isAvailable);
    }
    
    function removeAgent(bytes32 agentId) external nonReentrant {
        AgentMetadata storage meta = metadata[agentId];
        require(meta.agentId != bytes32(0), "Agent not registered");
        
        AgentIdentity.AgentIdentity memory identity = identityContract.getAgent(agentId);
        require(identity.owner == msg.sender || msg.sender == owner(), "Not authorized");
        
        // Remove from availableAgents array (swap and pop)
        uint256 index = agentIndex[agentId];
        uint256 lastIndex = availableAgents.length - 1;
        if (index != lastIndex) {
            bytes32 lastAgent = availableAgents[lastIndex];
            availableAgents[index] = lastAgent;
            agentIndex[lastAgent] = index;
        }
        availableAgents.pop();
        delete agentIndex[agentId];
        
        // Mark as unavailable
        meta.isAvailable = false;
        
        emit AgentRemoved(agentId);
    }
    
    function findAgentsByCapability(string calldata capability) 
        external 
        view 
        returns (bytes32[] memory) 
    {
        return capabilityIndex[capability];
    }
    
    function isAgentAvailable(bytes32 agentId) external view returns (bool) {
        AgentMetadata memory meta = metadata[agentId];
        if (meta.agentId == bytes32(0)) return false;
        if (!meta.isAvailable) return false;
        if (block.timestamp - meta.lastHeartbeat > HEARTBEAT_TIMEOUT) return false;
        return true;
    }
    
    function getAgentMetadata(bytes32 agentId) 
        external 
        view 
        returns (AgentMetadata memory) 
    {
        return metadata[agentId];
    }
    
    function getAvailableAgentsCount() external view returns (uint256) {
        return availableAgents.length;
    }
    
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
