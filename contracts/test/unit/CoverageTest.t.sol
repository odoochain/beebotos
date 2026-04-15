// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/dao/AgentDAO.sol";
import "../../src/dao/BeeToken.sol";
import "../../src/dao/VestingWallet.sol";
import "../../src/dao/TreasuryManager.sol";
import "../../src/interfaces/ITreasuryManager.sol";
import "../../src/core/AgentIdentity.sol";
import "../../src/core/AgentRegistry.sol";
import "../../src/interfaces/IERC8004.sol";
import "../../src/skills/SkillNFT.sol";
import "../../src/a2a/A2ACommerce.sol";
import "../../src/a2a/DealEscrow.sol";
import "../../src/a2a/DisputeResolution.sol";
import "../../src/payment/CrossChainBridge.sol";
import "../../src/interfaces/IA2ACommerce.sol";

/**
 * @title CoverageTest
 * @dev Comprehensive test suite for high coverage
 */
contract CoverageTest is Test {
    // Contracts
    BeeToken public token;
    VestingWallet public vesting;
    TreasuryManager public treasury;
    AgentDAO public dao;
    AgentIdentity public identity;
    AgentRegistry public registry;
    SkillNFT public skillNFT;
    A2ACommerce public a2a;
    DealEscrow public escrow;
    DisputeResolution public dispute;
    CrossChainBridge public bridge;
    
    // Addresses
    address public owner = address(1);
    address public user1 = address(2);
    address public user2 = address(3);
    address public agent = address(4);
    address public validator = address(5);
    
    // Mocks
    address public timelock;
    address public reputation;
    
    function setUp() public {
        vm.startPrank(owner);
        
        // Deploy token
        token = new BeeToken(owner, user1, user2, agent, address(6));
        
        // Deploy vesting
        vesting = new VestingWallet(address(token));
        token.transfer(address(vesting), 1_000_000e18);
        
        // Deploy identity
        identity = new AgentIdentity();
        identity.initialize();
        
        // Deploy registry
        registry = new AgentRegistry();
        registry.initialize(address(identity));
        
        // Deploy skill NFT
        skillNFT = new SkillNFT();
        skillNFT.initialize();
        
        // Deploy treasury
        treasury = new TreasuryManager(owner);
        token.transfer(address(treasury), 1_000_000e18);
        vm.deal(address(treasury), 100 ether);
        
        // Deploy escrow
        escrow = new DealEscrow();
        escrow.initialize(address(a2a), owner, 250);
        
        // Deploy A2A commerce
        a2a = new A2ACommerce();
        a2a.initialize(address(escrow));
        
        // Update escrow with correct A2A address
        escrow.setA2ACommerce(address(a2a));
        
        // Deploy dispute resolution
        dispute = new DisputeResolution(address(escrow), address(token));
        
        // Deploy bridge with validators
        address[] memory validators = new address[](1);
        validators[0] = validator;
        bridge = new CrossChainBridge(validators, 1);
        vm.deal(address(bridge), 100 ether);
        
        vm.stopPrank();
    }
    
    // ============ Token Tests ============
    
    function testTokenLockAndUnlock() public {
        // Transfer tokens to user
        vm.prank(owner);
        token.transfer(user1, 1000e18);
        
        // Lock tokens
        vm.prank(user1);
        uint256 lockId = token.lock(500e18, 30 days);
        assertEq(lockId, 0);
        
        // Try to unlock before expiry (should fail)
        vm.prank(user1);
        vm.expectRevert("BeeToken: lock not expired");
        token.unlock(lockId);
        
        // Fast forward and unlock
        vm.warp(block.timestamp + 31 days);
        vm.prank(user1);
        token.unlock(lockId);
        
        assertEq(token.balanceOf(user1), 1000e18);
    }
    
    function testTokenExtendLock() public {
        vm.prank(owner);
        token.transfer(user1, 1000e18);
        
        vm.prank(user1);
        uint256 lockId = token.lock(500e18, 30 days);
        
        uint256 originalEnd = block.timestamp + 30 days;
        
        vm.prank(user1);
        token.extendLock(lockId, 30 days);
        
        (,,uint256 endTime,,) = token.userLocks(user1, lockId);
        assertEq(endTime, originalEnd + 30 days);
    }
    
    // ============ Vesting Tests ============
    
    function testVestingLifecycle() public {
        vm.startPrank(owner);
        
        uint256 scheduleId = vesting.createVestingSchedule(
            user1,
            1000e18,
            block.timestamp,
            30 days,
            90 days,
            true
        );
        
        vm.stopPrank();
        
        // Before cliff
        vm.warp(block.timestamp + 15 days);
        assertEq(vesting.vestedAmount(scheduleId), 0);
        
        // After cliff, halfway through
        vm.warp(block.timestamp + 45 days); // 60 days total
        uint256 vested = vesting.vestedAmount(scheduleId);
        assertApproxEqRel(vested, 500e18, 0.1e18);
        
        // Full vesting
        vm.warp(block.timestamp + 30 days);
        vm.prank(user1);
        vesting.release(scheduleId);
        
        assertEq(token.balanceOf(user1), 1000e18);
    }
    
    function testVestingRevoke() public {
        vm.startPrank(owner);
        
        uint256 scheduleId = vesting.createVestingSchedule(
            user1,
            1000e18,
            block.timestamp,
            0,
            90 days,
            true
        );
        
        vm.warp(block.timestamp + 45 days);
        
        vesting.revoke(scheduleId);
        
        vm.stopPrank();
        
        // User should have received 50%
        assertApproxEqRel(token.balanceOf(user1), 500e18, 0.1e18);
    }
    
    // ============ Treasury Tests ============
    
    function testTreasuryBudgetLifecycle() public {
        vm.startPrank(owner);
        
        uint256 budgetId = treasury.createBudget(
            user1,
            10000e18,
            address(token),
            block.timestamp,
            block.timestamp + 30 days,
            ITreasuryManager.BudgetType.Development
        );
        
        treasury.grantRole(treasury.BUDGET_MANAGER(), user2);
        vm.stopPrank();
        
        vm.prank(user2);
        treasury.spendFromBudget(budgetId, 5000e18, "Test spend");
        
        assertEq(token.balanceOf(user1), 5000e18);
        
        // Update budget
        vm.prank(owner);
        treasury.updateBudget(budgetId, 15000e18, block.timestamp + 60 days);
        
        // Deactivate
        vm.prank(owner);
        treasury.deactivateBudget(budgetId);
        
        (,,,,,,,bool isActive) = treasury.budgets(budgetId);
        assertFalse(isActive);
    }
    
    function testTreasuryStreaming() public {
        vm.startPrank(owner);
        
        uint256 streamId = treasury.createStreamingPayment(
            user1,
            address(token),
            1000e18,
            30 days
        );
        
        vm.stopPrank();
        
        vm.warp(block.timestamp + 15 days);
        
        vm.prank(user1);
        uint256 released = treasury.releaseStream(streamId);
        
        assertApproxEqRel(released, 500e18, 0.1e18);
    }
    
    // ============ Identity Tests ============
    
    function testIdentityLifecycle() public {
        vm.prank(owner);
        
        bytes32 agentId = identity.registerAgent(
            "did:beebot:agent1",
            bytes32("publickey123")
        );
        
        assertTrue(identity.totalAgents() > 0);
        
        IERC8004.AgentIdentity memory agentData = identity.getAgent(agentId);
        assertEq(agentData.owner, owner);
        
        // Grant capability
        identity.grantCapability(agentId, keccak256("skill.trading"));
        assertTrue(identity.hasCapability(agentId, keccak256("skill.trading")));
        
        // Revoke capability
        identity.revokeCapability(agentId, keccak256("skill.trading"));
        assertFalse(identity.hasCapability(agentId, keccak256("skill.trading")));
        
        // Deactivate
        identity.deactivateAgent(agentId);
        IERC8004.AgentIdentity memory agentData2 = identity.getAgent(agentId);
        assertFalse(agentData2.isActive);
    }
    
    // ============ Registry Tests ============
    
    function testRegistryLifecycle() public {
        // First register identity
        vm.prank(user1);
        bytes32 agentId = identity.registerAgent("did:beebot:agent1", bytes32("pk"));
        
        // Then register metadata
        vm.prank(user1);
        registry.registerMetadata(
            agentId,
            "Trading Agent",
            "AI trading assistant",
            new string[](0),
            "https://api.example.com/agent1"
        );
        
        // Heartbeat
        vm.warp(block.timestamp + 30 minutes);
        vm.prank(user1);
        registry.heartbeat(agentId);
        
        // Check availability
        assertTrue(registry.isAgentAvailable(agentId));
        
        // Set unavailable
        vm.prank(user1);
        registry.setAvailability(agentId, false);
        assertFalse(registry.isAgentAvailable(agentId));
        
        // Remove agent
        vm.prank(user1);
        registry.removeAgent(agentId);
    }
    
    // ============ SkillNFT Tests ============
    
    function testSkillNFTMintAndTransfer() public {
        vm.prank(user1);
        uint256 tokenId = skillNFT.mintSkill(
            "Trading Skill",
            "1.0.0",
            "ipfs://metadata",
            true
        );
        
        assertEq(skillNFT.ownerOf(tokenId), user1);
        
        // Check royalty info
        (address receiver, uint256 royaltyAmount) = skillNFT.royaltyInfo(tokenId, 10000);
        assertEq(receiver, user1);
        assertEq(royaltyAmount, 250); // 2.5%
        
        // Transfer
        vm.prank(user1);
        skillNFT.transferFrom(user1, user2, tokenId);
        assertEq(skillNFT.ownerOf(tokenId), user2);
    }
    
    function testSkillNFTNonTransferable() public {
        vm.prank(user1);
        uint256 tokenId = skillNFT.mintSkill(
            "Exclusive Skill",
            "1.0.0",
            "ipfs://metadata",
            false // Not transferable
        );
        
        vm.prank(user1);
        vm.expectRevert("SkillNFT: skill not transferable");
        skillNFT.transferFrom(user1, user2, tokenId);
    }
    
    function testSkillNFTRoyaltyUpdate() public {
        vm.prank(user1);
        uint256 tokenId = skillNFT.mintSkill("Skill", "1.0", "ipfs://", true);
        
        vm.prank(user1);
        skillNFT.setTokenRoyalty(tokenId, 500); // 5%
        
        (, uint256 royalty) = skillNFT.royaltyInfo(tokenId, 10000);
        assertEq(royalty, 500);
    }
    
    // ============ A2A Commerce Tests ============
    
    function testA2AServiceListing() public {
        vm.prank(user1);
        bytes32 serviceId = a2a.listService("ipfs://service", 1000e18, address(token));
        
        A2ACommerce.ServiceListing memory service = a2a.getService(serviceId);
        assertEq(service.provider, user1);
        assertEq(service.price, 1000e18);
        
        // Update service
        vm.prank(user1);
        a2a.updateService(serviceId, 1500e18, true);
        
        service = a2a.getService(serviceId);
        assertEq(service.price, 1500e18);
    }
    
    function testA2ADealCreationAndCompletion() public {
        // List service
        vm.prank(user1);
        bytes32 serviceId = a2a.listService("ipfs://service", 1000e18, address(0));
        
        // Create deal
        vm.prank(user2);
        bytes32 dealId = a2a.createDeal(serviceId, block.timestamp + 7 days);
        
        // Fund deal
        vm.deal(user2, 10000e18);
        vm.prank(user2);
        a2a.fundDeal{value: 1000e18}(dealId);
        
        // Complete deal
        A2ACommerce.Deal memory deal = a2a.getDeal(dealId);
        assertEq(uint256(deal.status), uint256(IA2ACommerce.DealStatus.Funded));
    }
    
    // ============ Cross-Chain Bridge Tests ============
    
    function testBridgeOut() public {
        vm.deal(user1, 10000e18);
        
        // Add supported chain
        vm.prank(owner);
        bridge.addSupportedChain(137); // Polygon
        
        vm.prank(user1);
        bytes32 requestId = bridge.bridgeOut{value: 1000e18}(
            address(0),
            1000e18,
            137,
            bytes32(uint256(1)),
            user2
        );
        
        (
            bytes32 reqId,
            address sender,
            address recipient,
            uint256 amount,
            ,
            uint256 targetChain,
            ,
            CrossChainBridge.BridgeState state,
            ,
        ) = bridge.requests(requestId);
        
        assertEq(sender, user1);
        assertEq(recipient, user2);
        assertEq(targetChain, 137);
        assertEq(uint256(state), uint256(CrossChainBridge.BridgeState.Locked));
    }
    
    function testBridgeValidatorManagement() public {
        address newValidator = address(100);
        
        vm.prank(owner);
        bridge.addValidator(newValidator);
        
        assertTrue(bridge.validators(newValidator));
        
        vm.prank(owner);
        bridge.removeValidator(newValidator);
        
        assertFalse(bridge.validators(newValidator));
    }
    
    // ============ Dispute Resolution Tests ============
    
    function testDisputeLifecycle() public {
        // Setup: create a deal with escrow first
        vm.prank(owner);
        escrow.setA2ACommerce(address(this)); // Set ourselves as A2A for testing
        
        // Create escrow
        bytes32 escrowId = escrow.createEscrow{
            value: 1000e18
        }(
            bytes32(0),
            user1, // buyer
            user2, // seller
            address(0),
            1000e18
        );
        
        // Raise dispute
        vm.deal(user1, 10000e18);
        vm.prank(user1);
        bytes32 disputeId = dispute.raiseDispute{
            value: 0.1 ether
        }(
            bytes32(0),
            escrowId,
            user2,
            address(0),
            1000e18,
            "Service not delivered",
            bytes32("evidence123")
        );
        
        // Add arbiter
        vm.prank(owner);
        dispute.addArbiter(validator);
        
        // Start voting
        vm.prank(owner);
        dispute.startVoting(disputeId);
        
        // Cast vote
        vm.prank(validator);
        dispute.castVote(disputeId, true, "Buyer is right");
        
        // Finalize voting
        vm.warp(block.timestamp + 8 days);
        dispute.finalizeVoting(disputeId);
    }
    
    // ============ Pause/Unpause Tests ============
    
    function testPauseIdentity() public {
        vm.prank(owner);
        identity.pause();
        
        vm.expectRevert();
        identity.registerAgent("did:test", bytes32("pk"));
        
        vm.prank(owner);
        identity.unpause();
    }
    
    function testPauseA2A() public {
        vm.prank(owner);
        a2a.pause();
        
        vm.expectRevert();
        a2a.listService("ipfs://", 100, address(0));
        
        vm.prank(owner);
        a2a.unpause();
    }
    
    function testPauseSkillNFT() public {
        vm.prank(owner);
        skillNFT.pause();
        
        vm.prank(user1);
        vm.expectRevert();
        skillNFT.mintSkill("Skill", "1.0", "ipfs://", true);
        
        vm.prank(owner);
        skillNFT.unpause();
    }
    
    // ============ Gas Measurement Tests ============
    
    function testGas_TokenLock() public {
        vm.prank(owner);
        token.transfer(user1, 1000e18);
        
        vm.prank(user1);
        uint256 gasBefore = gasleft();
        token.lock(500e18, 30 days);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for token lock", gasUsed);
    }
    
    function testGas_VestingCreate() public {
        vm.prank(owner);
        uint256 gasBefore = gasleft();
        vesting.createVestingSchedule(
            user1,
            1000e18,
            block.timestamp,
            30 days,
            90 days,
            true
        );
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for vesting create", gasUsed);
    }
    
    function testGas_IdentityRegister() public {
        vm.prank(user1);
        uint256 gasBefore = gasleft();
        identity.registerAgent("did:test", bytes32("pk"));
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for identity register", gasUsed);
    }
    
    function testGas_SkillMint() public {
        vm.prank(user1);
        uint256 gasBefore = gasleft();
        skillNFT.mintSkill("Skill", "1.0", "ipfs://", true);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for skill mint", gasUsed);
    }
}

