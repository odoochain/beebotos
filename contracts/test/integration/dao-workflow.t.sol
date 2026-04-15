// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/dao/AgentDAO.sol";
import "../../src/dao/BeeToken.sol";
import "../../src/dao/TreasuryManager.sol";
import "../../src/interfaces/ITreasuryManager.sol";

/**
 * @title DAOIntegrationTest
 * @dev Integration tests for full DAO workflow
 */
contract DAOIntegrationTest is Test {
    AgentDAO public dao;
    BeeToken public token;
    TreasuryManager public treasury;
    
    address public admin = address(1);
    address public proposer = address(2);
    address public voter1 = address(3);
    address public voter2 = address(4);
    address public voter3 = address(5);
    
    uint256 public constant INITIAL_SUPPLY = 10_000_000e18;
    uint256 public constant PROPOSAL_THRESHOLD = 100_000e18;
    
    // Timelock for DAO
    address public timelock;
    
    function setUp() public {
        vm.startPrank(admin);
        
        // Deploy token
        token = new BeeToken(
            admin,      // treasury allocation
            address(6), // team
            address(7), // investors
            address(8), // ecosystem
            address(9)  // liquidity
        );
        
        // Deploy timelock (using admin as proxy for timelock in tests)
        timelock = address(new MockTimelock());
        
        // Deploy DAO with proper constructor parameters
        dao = new AgentDAO(
            "BeeBotOS DAO",
            IVotes(address(token)),
            TimelockController(payable(timelock)),
            address(new MockReputation()),
            address(new MockRegistry()),
            address(new MockAgentIdentity()),
            1,          // voting delay (1 block)
            40320,      // voting period (~1 week)
            400         // quorum numerator (4%)
        );
        
        // Deploy treasury
        treasury = new TreasuryManager(address(dao));
        
        // Distribute tokens and set up delegates
        token.transfer(proposer, 200_000e18);
        token.transfer(voter1, 500_000e18);
        token.transfer(voter2, 300_000e18);
        token.transfer(voter3, 100_000e18);
        
        vm.stopPrank();
    }
    
    // ============ Proposal Lifecycle Tests ============
    
    function testCreateProposal() public {
        // Setup: proposer delegates to self
        vm.prank(proposer);
        token.delegate(proposer);
        
        // Move forward to activate delegation
        vm.roll(block.number + 1);
        
        vm.startPrank(proposer);
        
        // Create proposal
        address[] memory targets = new address[](1);
        targets[0] = address(treasury);
        
        uint256[] memory values = new uint256[](1);
        values[0] = 0;
        
        bytes[] memory calldatas = new bytes[](1);
        calldatas[0] = abi.encodeWithSelector(
            TreasuryManager.createBudget.selector,
            address(0xdead),
            1,
            1000000e18,
            block.timestamp,
            block.timestamp + 30 days,
            ITreasuryManager.BudgetType.Development
        );
        
        uint256 proposalId = dao.propose(
            targets,
            values,
            calldatas,
            "Test proposal for development budget"
        );
        
        assertGt(proposalId, 0);
        
        // Check proposal state is Pending
        assertEq(uint256(dao.state(proposalId)), uint256(IGovernor.ProposalState.Pending));
        
        vm.stopPrank();
    }
    
    function testFullProposalLifecycle() public {
        // Setup delegates
        vm.prank(proposer);
        token.delegate(proposer);
        
        vm.prank(voter1);
        token.delegate(voter1);
        
        vm.prank(voter2);
        token.delegate(voter2);
        
        // Move forward to activate delegations
        vm.roll(block.number + 1);
        
        // Create proposal
        vm.prank(proposer);
        address[] memory targets = new address[](1);
        targets[0] = address(treasury);
        
        uint256[] memory values = new uint256[](1);
        bytes[] memory calldatas = new bytes[](1);
        calldatas[0] = abi.encodeWithSelector(
            TreasuryManager.createBudget.selector,
            beneficiary(),
            1_000_000e18,
            block.timestamp,
            block.timestamp + 30 days,
            ITreasuryManager.BudgetType.Development
        );
        
        uint256 proposalId = dao.propose(
            targets,
            values,
            calldatas,
            "Full lifecycle test proposal"
        );
        
        // Move past voting delay to Active
        vm.roll(block.number + 2);
        assertEq(uint256(dao.state(proposalId)), uint256(IGovernor.ProposalState.Active));
        
        // Cast votes
        vm.prank(voter1);
        dao.castVote(proposalId, 1); // For
        
        vm.prank(voter2);
        dao.castVote(proposalId, 1); // For
        
        vm.prank(voter3);
        dao.castVote(proposalId, 0); // Against
        
        // Move past voting period
        vm.roll(block.number + 40321);
        
        // Should be Succeeded
        assertEq(uint256(dao.state(proposalId)), uint256(IGovernor.ProposalState.Succeeded));
        
        // Queue proposal
        bytes32 descriptionHash = keccak256(bytes("Full lifecycle test proposal"));
        dao.queue(targets, values, calldatas, descriptionHash);
        
        // Should be Queued
        assertEq(uint256(dao.state(proposalId)), uint256(IGovernor.ProposalState.Queued));
    }
    
    function testProposalDefeated() public {
        // Setup
        vm.prank(proposer);
        token.delegate(proposer);
        
        vm.prank(voter1);
        token.delegate(voter1);
        
        vm.prank(voter2);
        token.delegate(voter2);
        
        vm.roll(block.number + 1);
        
        // Create proposal
        vm.prank(proposer);
        address[] memory targets = new address[](1);
        targets[0] = address(treasury);
        uint256[] memory values = new uint256[](1);
        bytes[] memory calldatas = new bytes[](1);
        
        uint256 proposalId = dao.propose(targets, values, calldatas, "Defeated proposal");
        
        vm.roll(block.number + 2);
        
        // Vote against
        vm.prank(voter1);
        dao.castVote(proposalId, 0); // Against
        
        vm.prank(voter2);
        dao.castVote(proposalId, 0); // Against
        
        vm.roll(block.number + 40321);
        
        // Should be Defeated
        assertEq(uint256(dao.state(proposalId)), uint256(IGovernor.ProposalState.Defeated));
    }
    
    function testProposalCanceled() public {
        // Setup
        vm.prank(proposer);
        token.delegate(proposer);
        vm.roll(block.number + 1);
        
        // Create proposal
        vm.prank(proposer);
        address[] memory targets = new address[](1);
        uint256[] memory values = new uint256[](1);
        bytes[] memory calldatas = new bytes[](1);
        
        uint256 proposalId = dao.propose(targets, values, calldatas, "Canceled proposal");
        
        // Cancel (proposer can cancel before voting starts)
        vm.prank(proposer);
        bytes32 descriptionHash = keccak256(bytes("Canceled proposal"));
        dao.cancel(targets, values, calldatas, descriptionHash);
        
        // Should be Canceled
        assertEq(uint256(dao.state(proposalId)), uint256(IGovernor.ProposalState.Canceled));
    }
    
    function testDelegationAndVotingPower() public {
        // Setup delegation
        vm.prank(voter1);
        token.delegate(voter2);
        
        vm.roll(block.number + 1);
        
        // Voter2 should have voter1's voting power
        uint256 voter2Power = dao.getVotes(voter2, block.number - 1);
        assertEq(voter2Power, 500_000e18);
        
        // Voter1 should have no voting power
        uint256 voter1Power = dao.getVotes(voter1, block.number - 1);
        assertEq(voter1Power, 0);
    }
    
    function testCannotProposeWithoutThreshold() public {
        // Voter3 has less than proposal threshold
        vm.prank(voter3);
        token.delegate(voter3);
        vm.roll(block.number + 1);
        
        vm.prank(voter3);
        address[] memory targets = new address[](1);
        uint256[] memory values = new uint256[](1);
        bytes[] memory calldatas = new bytes[](1);
        
        vm.expectRevert();
        dao.propose(targets, values, calldatas, "Under threshold proposal");
    }
    
    // ============ Helper Functions ============
    
    function beneficiary() internal pure returns (address) {
        return address(0xdead);
    }
}

// Mock contracts for testing
contract MockTimelock {
    // Minimal timelock implementation for tests
}

contract MockReputation {
    mapping(address => uint256) public reputations;
    
    function getReputation(address account) external view returns (uint256) {
        return reputations[account] > 0 ? reputations[account] : 100; // Default 100
    }
    
    function getCategoryScore(address, bytes32) external pure returns (uint256) {
        return 100;
    }
}

contract MockRegistry {
    function getAgentMetadata(bytes32) external view returns (
        bytes32, string memory, string memory, string[] memory, string memory, uint256, bool, uint256
    ) {
        return (bytes32(0), "", "", new string[](0), "", 0, true, block.timestamp);
    }
    
    function isAgentAvailable(bytes32) external pure returns (bool) {
        return true;
    }
}

contract MockAgentIdentity {
    struct AgentIdentityData {
        bytes32 agentId;
        address owner;
        string did;
        bytes32 publicKey;
        bool isActive;
        uint256 reputation;
        uint256 createdAt;
    }
    
    function getAgent(bytes32 agentId) external view returns (AgentIdentityData memory) {
        return AgentIdentityData({
            agentId: agentId,
            owner: address(0x1234),
            did: "did:mock:123",
            publicKey: bytes32(0),
            isActive: true,
            reputation: 100,
            createdAt: block.timestamp
        });
    }
    
    function didToAgent(string calldata) external pure returns (bytes32) {
        return bytes32(0);
    }
}


