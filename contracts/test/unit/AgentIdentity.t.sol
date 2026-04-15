// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/core/AgentIdentity.sol";

/**
 * @title AgentIdentityTest
 * @dev Comprehensive tests for AgentIdentity (Target: 90%+ coverage)
 */
contract AgentIdentityTest is Test {
    AgentIdentity public identity;
    
    address public owner = address(1);
    address public updater = address(2);
    address public user1 = address(3);
    address public user2 = address(4);
    
    string constant DID1 = "did:beebot:agent1";
    string constant DID2 = "did:beebot:agent2";
    bytes32 constant PUBLIC_KEY1 = keccak256("pubkey1");
    bytes32 constant PUBLIC_KEY2 = keccak256("pubkey2");
    bytes32 constant CAPABILITY = keccak256("trading");
    
    function setUp() public {
        vm.prank(owner);
        identity = new AgentIdentity();
        identity.initialize();
        
        vm.prank(owner);
        identity.setAuthorizedUpdater(updater, true);
    }
    
    // ============ Initialization Tests ============
    
    function testInitialization() public view {
        assertEq(identity.owner(), owner);
        assertTrue(identity.authorizedUpdaters(updater));
    }
    
    function testCannotInitializeTwice() public {
        vm.prank(owner);
        vm.expectRevert("AgentIdentity: already initialized");
        identity.initialize();
    }
    
    // ============ Register Agent Tests ============
    
    function testRegisterAgent() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        assertTrue(agentId != bytes32(0));
        
        AgentIdentity.AgentIdentity memory agent = identity.getAgent(agentId);
        assertEq(agent.agentId, agentId);
        assertEq(agent.owner, user1);
        assertEq(agent.did, DID1);
        assertEq(agent.publicKey, PUBLIC_KEY1);
        assertTrue(agent.isActive);
        assertEq(agent.reputation, 100);
        assertGt(agent.createdAt, 0);
    }
    
    function testRegisterAgentEmitsEvent() public {
        vm.prank(user1);
        vm.expectEmit(true, true, false, false);
        emit AgentIdentity.AgentRegistered(bytes32(0), user1, DID1);
        identity.registerAgent(DID1, PUBLIC_KEY1);
    }
    
    function testRegisterAgentEmptyDID() public {
        vm.prank(user1);
        vm.expectRevert("DID required");
        identity.registerAgent("", PUBLIC_KEY1);
    }
    
    function testRegisterAgentEmptyPublicKey() public {
        vm.prank(user1);
        vm.expectRevert("Public key required");
        identity.registerAgent(DID1, bytes32(0));
    }
    
    function testRegisterAgentDuplicateDID() public {
        vm.prank(user1);
        identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user2);
        vm.expectRevert("DID already registered");
        identity.registerAgent(DID1, PUBLIC_KEY2);
    }
    
    function testRegisterAgentIncrementsTotalAgents() public {
        uint256 totalBefore = identity.totalAgents();
        
        vm.prank(user1);
        identity.registerAgent(DID1, PUBLIC_KEY1);
        
        assertEq(identity.totalAgents(), totalBefore + 1);
    }
    
    function testRegisterAgentWhenPaused() public {
        vm.prank(owner);
        identity.pause();
        
        vm.prank(user1);
        vm.expectRevert();
        identity.registerAgent(DID1, PUBLIC_KEY1);
    }
    
    function testRegisterAgentThroughProxyOnly() public {
        // Direct call to implementation should fail
        AgentIdentity impl = new AgentIdentity();
        impl.initialize();
        
        vm.prank(user1);
        vm.expectRevert("AgentIdentity: must be called through proxy");
        impl.registerAgent(DID1, PUBLIC_KEY1);
    }
    
    function testGeneratedAgentIdIsUnique() public {
        vm.prank(user1);
        bytes32 agentId1 = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user2);
        bytes32 agentId2 = identity.registerAgent(DID2, PUBLIC_KEY2);
        
        assertTrue(agentId1 != agentId2);
    }
    
    // ============ Update Reputation Tests ============
    
    function testUpdateReputation() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(updater);
        identity.updateReputation(agentId, 500);
        
        AgentIdentity.AgentIdentity memory agent = identity.getAgent(agentId);
        assertEq(agent.reputation, 500);
    }
    
    function testUpdateReputationNotActive() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        identity.deactivateAgent(agentId);
        
        vm.prank(updater);
        vm.expectRevert("Agent not active");
        identity.updateReputation(agentId, 500);
    }
    
    function testUpdateReputationWhenPaused() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(owner);
        identity.pause();
        
        vm.prank(updater);
        vm.expectRevert();
        identity.updateReputation(agentId, 500);
    }
    
    function testUpdateReputationNotAuthorized() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        address randomUser = address(999);
        vm.prank(randomUser);
        vm.expectRevert("AgentIdentity: not authorized updater");
        identity.updateReputation(agentId, 500);
    }
    
    function testUpdateReputationEmitsEvent() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(updater);
        vm.expectEmit(true, false, false, false);
        emit AgentIdentity.AgentUpdated(agentId, "reputation");
        identity.updateReputation(agentId, 500);
    }
    
    function testOwnerCanUpdateReputation() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(owner);
        identity.updateReputation(agentId, 500);
        
        AgentIdentity.AgentIdentity memory agent = identity.getAgent(agentId);
        assertEq(agent.reputation, 500);
    }
    
    // ============ Deactivate Agent Tests ============
    
    function testDeactivateAgent() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        identity.deactivateAgent(agentId);
        
        AgentIdentity.AgentIdentity memory agent = identity.getAgent(agentId);
        assertFalse(agent.isActive);
    }
    
    function testDeactivateAgentEmitsEvent() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        vm.expectEmit(true, false, false, false);
        emit AgentIdentity.AgentDeactivated(agentId);
        identity.deactivateAgent(agentId);
    }
    
    function testDeactivateAgentNotOwner() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user2);
        vm.expectRevert("Not agent owner");
        identity.deactivateAgent(agentId);
    }
    
    function testDeactivateAgentWhenPaused() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(owner);
        identity.pause();
        
        vm.prank(user1);
        vm.expectRevert();
        identity.deactivateAgent(agentId);
    }
    
    // ============ Capability Tests ============
    
    function testGrantCapability() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        identity.grantCapability(agentId, CAPABILITY);
        
        assertTrue(identity.hasCapability(agentId, CAPABILITY));
    }
    
    function testGrantCapabilityEmitsEvent() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        vm.expectEmit(true, true, false, false);
        emit AgentIdentity.CapabilityGranted(agentId, CAPABILITY);
        identity.grantCapability(agentId, CAPABILITY);
    }
    
    function testGrantCapabilityNotOwner() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user2);
        vm.expectRevert("Not agent owner");
        identity.grantCapability(agentId, CAPABILITY);
    }
    
    function testGrantCapabilityWhenPaused() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(owner);
        identity.pause();
        
        vm.prank(user1);
        vm.expectRevert();
        identity.grantCapability(agentId, CAPABILITY);
    }
    
    function testRevokeCapability() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        identity.grantCapability(agentId, CAPABILITY);
        
        vm.prank(user1);
        identity.revokeCapability(agentId, CAPABILITY);
        
        assertFalse(identity.hasCapability(agentId, CAPABILITY));
    }
    
    function testRevokeCapabilityEmitsEvent() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        identity.grantCapability(agentId, CAPABILITY);
        
        vm.prank(user1);
        vm.expectEmit(true, true, false, false);
        emit AgentIdentity.CapabilityRevoked(agentId, CAPABILITY);
        identity.revokeCapability(agentId, CAPABILITY);
    }
    
    function testRevokeCapabilityNotOwner() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        identity.grantCapability(agentId, CAPABILITY);
        
        vm.prank(user2);
        vm.expectRevert("Not agent owner");
        identity.revokeCapability(agentId, CAPABILITY);
    }
    
    // ============ Get Owner Agents Tests ============
    
    function testGetOwnerAgents() public {
        vm.prank(user1);
        bytes32 agentId1 = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        bytes32 agentId2 = identity.registerAgent(DID2, PUBLIC_KEY2);
        
        bytes32[] memory agents = identity.getOwnerAgents(user1);
        
        assertEq(agents.length, 2);
        assertTrue(agents[0] == agentId1 || agents[1] == agentId1);
        assertTrue(agents[0] == agentId2 || agents[1] == agentId2);
    }
    
    function testGetOwnerAgentsEmpty() public view {
        bytes32[] memory agents = identity.getOwnerAgents(address(999));
        assertEq(agents.length, 0);
    }
    
    // ============ Authorized Updater Tests ============
    
    function testSetAuthorizedUpdater() public {
        address newUpdater = address(999);
        
        vm.prank(owner);
        identity.setAuthorizedUpdater(newUpdater, true);
        
        assertTrue(identity.authorizedUpdaters(newUpdater));
    }
    
    function testSetAuthorizedUpdaterRemove() public {
        vm.prank(owner);
        identity.setAuthorizedUpdater(updater, false);
        
        assertFalse(identity.authorizedUpdaters(updater));
    }
    
    function testSetAuthorizedUpdaterEmitsEvent() public {
        address newUpdater = address(999);
        
        vm.prank(owner);
        vm.expectEmit(true, false, false, true);
        emit AgentIdentity.UpdaterAuthorized(newUpdater, true);
        identity.setAuthorizedUpdater(newUpdater, true);
    }
    
    function testSetAuthorizedUpdaterZeroAddress() public {
        vm.prank(owner);
        vm.expectRevert("AgentIdentity: zero address");
        identity.setAuthorizedUpdater(address(0), true);
    }
    
    function testOnlyOwnerCanSetUpdater() public {
        vm.prank(user1);
        vm.expectRevert();
        identity.setAuthorizedUpdater(address(999), true);
    }
    
    // ============ Pause/Unpause Tests ============
    
    function testPause() public {
        vm.prank(owner);
        identity.pause();
        
        assertTrue(identity.paused());
    }
    
    function testUnpause() public {
        vm.prank(owner);
        identity.pause();
        
        vm.prank(owner);
        identity.unpause();
        
        assertFalse(identity.paused());
    }
    
    function testOnlyOwnerCanPause() public {
        vm.prank(user1);
        vm.expectRevert();
        identity.pause();
    }
    
    // ============ DID to Agent Mapping Tests ============
    
    function testDidToAgentMapping() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        assertEq(identity.didToAgent(DID1), agentId);
    }
    
    // ============ Gas Measurement Tests ============
    
    function testGas_RegisterAgent() public {
        vm.prank(user1);
        uint256 gasBefore = gasleft();
        identity.registerAgent(DID1, PUBLIC_KEY1);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for registerAgent", gasUsed);
    }
    
    function testGas_UpdateReputation() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(updater);
        uint256 gasBefore = gasleft();
        identity.updateReputation(agentId, 500);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for updateReputation", gasUsed);
    }
    
    function testGas_GrantCapability() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        vm.prank(user1);
        uint256 gasBefore = gasleft();
        identity.grantCapability(agentId, CAPABILITY);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for grantCapability", gasUsed);
    }
    
    // ============ Edge Cases ============
    
    function testGetAgentNonExistent() public view {
        bytes32 fakeAgentId = keccak256("fake");
        AgentIdentity.AgentIdentity memory agent = identity.getAgent(fakeAgentId);
        assertEq(agent.agentId, bytes32(0));
    }
    
    function testMultipleCapabilities() public {
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        bytes32[] memory capabilities = new bytes32[](5);
        for (uint i = 0; i < 5; i++) {
            capabilities[i] = keccak256(abi.encodePacked("cap", i));
            vm.prank(user1);
            identity.grantCapability(agentId, capabilities[i]);
        }
        
        for (uint i = 0; i < 5; i++) {
            assertTrue(identity.hasCapability(agentId, capabilities[i]));
        }
    }
    
    function testRegisterManyAgents() public {
        uint256 totalBefore = identity.totalAgents();
        uint256 agentsToRegister = 50;
        
        for (uint i = 0; i < agentsToRegister; i++) {
            address user = address(uint160(1000 + i));
            
            vm.prank(user);
            identity.registerAgent(
                string(abi.encodePacked("did:beebot:agent", vm.toString(i))),
                keccak256(abi.encodePacked("pubkey", i))
            );
        }
        
        assertEq(identity.totalAgents(), totalBefore + agentsToRegister);
    }
    
    function testFullLifecycle() public {
        // Register
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent(DID1, PUBLIC_KEY1);
        
        // Update reputation
        vm.prank(updater);
        identity.updateReputation(agentId, 500);
        
        // Grant capability
        vm.prank(user1);
        identity.grantCapability(agentId, CAPABILITY);
        
        // Deactivate
        vm.prank(user1);
        identity.deactivateAgent(agentId);
        
        AgentIdentity.AgentIdentity memory agent = identity.getAgent(agentId);
        assertFalse(agent.isActive);
        assertEq(agent.reputation, 500);
    }
}
