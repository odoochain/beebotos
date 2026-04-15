// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/core/AgentRegistry.sol";
import "../../src/core/AgentIdentity.sol";

/**
 * @title AgentRegistryTest
 * @dev Comprehensive tests for AgentRegistry (Target: 90%+ coverage)
 */
contract AgentRegistryTest is Test {
    AgentRegistry public registry;
    AgentIdentity public identity;
    
    address public owner = address(1);
    address public user1 = address(2);
    address public user2 = address(3);
    
    bytes32 public agentId1;
    bytes32 public agentId2;
    
    string constant DID1 = "did:beebot:agent1";
    string constant DID2 = "did:beebot:agent2";
    bytes32 constant PUBLIC_KEY1 = keccak256("pubkey1");
    bytes32 constant PUBLIC_KEY2 = keccak256("pubkey2");
    
    function setUp() public {
        vm.startPrank(owner);
        
        // Deploy and initialize identity
        identity = new AgentIdentity();
        identity.initialize();
        
        // Deploy and initialize registry
        registry = new AgentRegistry();
        registry.initialize(address(identity));
        
        vm.stopPrank();
        
        // Register agents through identity
        vm.prank(user1);
        agentId1 = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user2);
        agentId2 = identity.registerAgent(DID2, PUBLIC_KEY2);
    }
    
    // ============ Initialization Tests ============
    
    function testInitialization() public view {
        assertEq(address(registry.identityContract()), address(identity));
        assertEq(registry.owner(), owner);
    }
    
    function testCannotInitializeTwice() public {
        vm.prank(owner);
        vm.expectRevert("AgentRegistry: already initialized");
        registry.initialize(address(identity));
    }
    
    function testCannotInitializeWithZeroAddress() public {
        AgentRegistry newRegistry = new AgentRegistry();
        vm.prank(owner);
        vm.expectRevert("AgentRegistry: zero address");
        newRegistry.initialize(address(0));
    }
    
    // ============ Register Metadata Tests ============
    
    function testRegisterMetadata() public {
        string[] memory capabilities = new string[](2);
        capabilities[0] = "trading";
        capabilities[1] = "analysis";
        
        vm.prank(user1);
        registry.registerMetadata(
            agentId1,
            "Agent One",
            "A test agent",
            capabilities,
            "https://api.agent1.com"
        );
        
        AgentRegistry.AgentMetadata memory meta = registry.getAgentMetadata(agentId1);
        assertEq(meta.agentId, agentId1);
        assertEq(meta.name, "Agent One");
        assertEq(meta.description, "A test agent");
        assertEq(meta.capabilities.length, 2);
        assertEq(meta.endpoint, "https://api.agent1.com");
        assertEq(meta.version, 1);
        assertTrue(meta.isAvailable);
        assertGt(meta.lastHeartbeat, 0);
    }
    
    function testRegisterMetadataOnlyOwner() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        // user2 tries to register metadata for user1's agent
        vm.prank(user2);
        vm.expectRevert("Not owner");
        registry.registerMetadata(
            agentId1,
            "Agent One",
            "A test agent",
            capabilities,
            "https://api.agent1.com"
        );
    }
    
    function testRegisterMetadataOnlyActiveAgent() public {
        // Deactivate agent first
        vm.prank(user1);
        identity.deactivateAgent(agentId1);
        
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        vm.expectRevert("Agent not active");
        registry.registerMetadata(
            agentId1,
            "Agent One",
            "A test agent",
            capabilities,
            "https://api.agent1.com"
        );
    }
    
    function testRegisterMetadataEmptyName() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        vm.expectRevert("Name required");
        registry.registerMetadata(
            agentId1,
            "",
            "A test agent",
            capabilities,
            "https://api.agent1.com"
        );
    }
    
    function testRegisterMetadataEmptyEndpoint() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        vm.expectRevert("Endpoint required");
        registry.registerMetadata(
            agentId1,
            "Agent One",
            "A test agent",
            capabilities,
            ""
        );
    }
    
    function testRegisterMetadataEmitsEvent() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        vm.expectEmit(true, false, false, true);
        emit AgentRegistry.MetadataUpdated(agentId1, "Agent One");
        registry.registerMetadata(
            agentId1,
            "Agent One",
            "A test agent",
            capabilities,
            "https://api.agent1.com"
        );
    }
    
    // ============ Update Metadata Tests ============
    
    function testUpdateMetadata() public {
        // First register
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        // Then update
        string[] memory newCapabilities = new string[](2);
        newCapabilities[0] = "trading";
        newCapabilities[1] = "analytics";
        
        vm.prank(user1);
        registry.updateMetadata(
            agentId1,
            "Agent One Updated",
            "Updated description",
            newCapabilities,
            "https://api-new.com"
        );
        
        AgentRegistry.AgentMetadata memory meta = registry.getAgentMetadata(agentId1);
        assertEq(meta.name, "Agent One Updated");
        assertEq(meta.version, 2);
    }
    
    function testUpdateMetadataNotRegistered() public {
        bytes32 fakeAgentId = keccak256("fake");
        string[] memory capabilities = new string[](1);
        
        vm.prank(user1);
        vm.expectRevert("Agent not registered");
        registry.updateMetadata(fakeAgentId, "Name", "Desc", capabilities, "https://api.com");
    }
    
    function testUpdateMetadataNotOwner() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(user2);
        vm.expectRevert("Not owner");
        registry.updateMetadata(agentId1, "Hacked", "Desc", capabilities, "https://evil.com");
    }
    
    // ============ Heartbeat Tests ============
    
    function testHeartbeat() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        uint256 heartbeatBefore = registry.getAgentMetadata(agentId1).lastHeartbeat;
        
        // Fast forward
        vm.warp(block.timestamp + 1 hours);
        
        vm.prank(user1);
        registry.heartbeat(agentId1);
        
        uint256 heartbeatAfter = registry.getAgentMetadata(agentId1).lastHeartbeat;
        assertGt(heartbeatAfter, heartbeatBefore);
    }
    
    function testHeartbeatEmitsEvent() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(user1);
        vm.expectEmit(true, false, false, true);
        emit AgentRegistry.Heartbeat(agentId1, block.timestamp);
        registry.heartbeat(agentId1);
    }
    
    function testHeartbeatNotRegistered() public {
        bytes32 fakeAgentId = keccak256("fake");
        
        vm.prank(user1);
        vm.expectRevert("Agent not registered");
        registry.heartbeat(fakeAgentId);
    }
    
    function testHeartbeatWhenPaused() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(owner);
        registry.pause();
        
        vm.prank(user1);
        vm.expectRevert();
        registry.heartbeat(agentId1);
    }
    
    // ============ Set Availability Tests ============
    
    function testSetAvailability() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        // Set unavailable
        vm.prank(user1);
        registry.setAvailability(agentId1, false);
        
        assertFalse(registry.getAgentMetadata(agentId1).isAvailable);
        
        // Set available again
        vm.prank(user1);
        registry.setAvailability(agentId1, true);
        
        assertTrue(registry.getAgentMetadata(agentId1).isAvailable);
    }
    
    function testSetAvailabilityByOwner() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        // Contract owner can also set availability
        vm.prank(owner);
        registry.setAvailability(agentId1, false);
        
        assertFalse(registry.getAgentMetadata(agentId1).isAvailable);
    }
    
    function testSetAvailabilityNotAuthorized() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        address randomUser = address(999);
        vm.prank(randomUser);
        vm.expectRevert("Not authorized");
        registry.setAvailability(agentId1, false);
    }
    
    function testSetAvailabilityNotRegistered() public {
        bytes32 fakeAgentId = keccak256("fake");
        
        vm.prank(user1);
        vm.expectRevert("Agent not registered");
        registry.setAvailability(fakeAgentId, false);
    }
    
    function testSetAvailabilityEmitsEvent() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(user1);
        vm.expectEmit(true, false, false, true);
        emit AgentRegistry.AvailabilityChanged(agentId1, false);
        registry.setAvailability(agentId1, false);
    }
    
    function testSetAvailabilityWhenPaused() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(owner);
        registry.pause();
        
        vm.prank(user1);
        vm.expectRevert();
        registry.setAvailability(agentId1, false);
    }
    
    // ============ Remove Agent Tests ============
    
    function testRemoveAgent() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(user1);
        registry.removeAgent(agentId1);
        
        assertFalse(registry.getAgentMetadata(agentId1).isAvailable);
    }
    
    function testRemoveAgentByOwner() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(owner);
        registry.removeAgent(agentId1);
        
        assertFalse(registry.getAgentMetadata(agentId1).isAvailable);
    }
    
    function testRemoveAgentNotAuthorized() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        address randomUser = address(999);
        vm.prank(randomUser);
        vm.expectRevert("Not authorized");
        registry.removeAgent(agentId1);
    }
    
    function testRemoveAgentEmitsEvent() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(user1);
        vm.expectEmit(true, false, false, false);
        emit AgentRegistry.AgentRemoved(agentId1);
        registry.removeAgent(agentId1);
    }
    
    function testRemoveAgentRemovesFromAvailableList() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        uint256 countBefore = registry.getAvailableAgentsCount();
        
        vm.prank(user1);
        registry.removeAgent(agentId1);
        
        uint256 countAfter = registry.getAvailableAgentsCount();
        assertEq(countAfter, countBefore - 1);
    }
    
    // ============ Find Agents by Capability Tests ============
    
    function testFindAgentsByCapability() public {
        string[] memory capabilities1 = new string[](2);
        capabilities1[0] = "trading";
        capabilities1[1] = "analysis";
        
        string[] memory capabilities2 = new string[](1);
        capabilities2[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities1, "https://api1.com");
        
        vm.prank(user2);
        registry.registerMetadata(agentId2, "Agent Two", "Desc", capabilities2, "https://api2.com");
        
        bytes32[] memory tradingAgents = registry.findAgentsByCapability("trading");
        assertEq(tradingAgents.length, 2);
        
        bytes32[] memory analysisAgents = registry.findAgentsByCapability("analysis");
        assertEq(analysisAgents.length, 1);
        assertEq(analysisAgents[0], agentId1);
    }
    
    // ============ Is Agent Available Tests ============
    
    function testIsAgentAvailable() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        assertTrue(registry.isAgentAvailable(agentId1));
        
        // Fast forward past heartbeat timeout
        vm.warp(block.timestamp + registry.HEARTBEAT_TIMEOUT() + 1);
        
        assertFalse(registry.isAgentAvailable(agentId1));
    }
    
    function testIsAgentAvailableNotRegistered() public view {
        bytes32 fakeAgentId = keccak256("fake");
        assertFalse(registry.isAgentAvailable(fakeAgentId));
    }
    
    function testIsAgentAvailableNotAvailable() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(user1);
        registry.setAvailability(agentId1, false);
        
        assertFalse(registry.isAgentAvailable(agentId1));
    }
    
    // ============ Pause/Unpause Tests ============
    
    function testPause() public {
        vm.prank(owner);
        registry.pause();
        
        assertTrue(registry.paused());
    }
    
    function testUnpause() public {
        vm.prank(owner);
        registry.pause();
        
        vm.prank(owner);
        registry.unpause();
        
        assertFalse(registry.paused());
    }
    
    function testOnlyOwnerCanPause() public {
        vm.prank(user1);
        vm.expectRevert();
        registry.pause();
    }
    
    // ============ Upgrade Tests ============
    
    function testUpgradeAuthorization() public {
        // Only owner can authorize upgrade
        AgentRegistry newImplementation = new AgentRegistry();
        
        vm.prank(owner);
        // Should not revert
        registry.upgradeTo(address(newImplementation));
    }
    
    function testOnlyOwnerCanUpgrade() public {
        AgentRegistry newImplementation = new AgentRegistry();
        
        vm.prank(user1);
        vm.expectRevert();
        registry.upgradeTo(address(newImplementation));
    }
    
    // ============ Gas Measurement Tests ============
    
    function testGas_RegisterMetadata() public {
        string[] memory capabilities = new string[](3);
        capabilities[0] = "trading";
        capabilities[1] = "analysis";
        capabilities[2] = "forecasting";
        
        vm.prank(user1);
        uint256 gasBefore = gasleft();
        registry.registerMetadata(
            agentId1,
            "Agent One",
            "A test agent",
            capabilities,
            "https://api.agent1.com"
        );
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for registerMetadata", gasUsed);
    }
    
    function testGas_Heartbeat() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent One", "Desc", capabilities, "https://api.com");
        
        vm.prank(user1);
        uint256 gasBefore = gasleft();
        registry.heartbeat(agentId1);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for heartbeat", gasUsed);
    }
    
    function testGas_FindAgentsByCapability() public {
        // Register multiple agents
        for (uint i = 0; i < 10; i++) {
            address user = address(uint160(100 + i));
            
            // Register agent in identity
            vm.prank(user);
            bytes32 agentId = identity.registerAgent(
                string(abi.encodePacked("did:beebot:agent", vm.toString(i))),
                keccak256(abi.encodePacked("pubkey", i))
            );
            
            string[] memory capabilities = new string[](1);
            capabilities[0] = "trading";
            
            vm.prank(user);
            registry.registerMetadata(agentId, "Agent", "Desc", capabilities, "https://api.com");
        }
        
        uint256 gasBefore = gasleft();
        registry.findAgentsByCapability("trading");
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for findAgentsByCapability (10 agents)", gasUsed);
    }
    
    // ============ Edge Cases ============
    
    function testRegisterManyCapabilities() public {
        string[] memory capabilities = new string[](10);
        for (uint i = 0; i < 10; i++) {
            capabilities[i] = string(abi.encodePacked("cap", vm.toString(i)));
        }
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent", "Desc", capabilities, "https://api.com");
        
        AgentRegistry.AgentMetadata memory meta = registry.getAgentMetadata(agentId1);
        assertEq(meta.capabilities.length, 10);
    }
    
    function testMultipleAgentsSameOwner() public {
        // Register second agent for user1
        vm.prank(user1);
        bytes32 agentId3 = identity.registerAgent("did:beebot:agent3", keccak256("pubkey3"));
        
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent 1", "Desc", capabilities, "https://api1.com");
        
        vm.prank(user1);
        registry.registerMetadata(agentId3, "Agent 3", "Desc", capabilities, "https://api3.com");
        
        assertEq(registry.getAvailableAgentsCount(), 2);
    }
    
    function testRegisterUpdateRemoveFlow() public {
        string[] memory capabilities = new string[](1);
        capabilities[0] = "trading";
        
        // Register
        vm.prank(user1);
        registry.registerMetadata(agentId1, "Agent", "Desc", capabilities, "https://api.com");
        assertTrue(registry.getAgentMetadata(agentId1).isAvailable);
        
        // Update
        vm.prank(user1);
        registry.updateMetadata(agentId1, "Updated", "Updated Desc", capabilities, "https://new.com");
        assertEq(registry.getAgentMetadata(agentId1).name, "Updated");
        
        // Remove
        vm.prank(user1);
        registry.removeAgent(agentId1);
        assertFalse(registry.getAgentMetadata(agentId1).isAvailable);
    }
}
