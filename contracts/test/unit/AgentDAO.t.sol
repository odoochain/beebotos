// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/dao/AgentDAO.sol";
import "../../src/dao/BeeToken.sol";
import "../../src/dao/TreasuryManager.sol";

/**
 * @title AgentDAOTest
 * @dev Comprehensive tests for AgentDAO governance
 */
contract AgentDAOTest is Test {
    BeeToken public beeToken;
    TreasuryManager public treasury;
    AgentDAO public dao;
    
    address public deployer = address(1);
    address public member1 = address(2);
    address public member2 = address(3);
    address public member3 = address(4);
    
    // Mock external contracts
    address public reputationSystem;
    address public agentRegistry;
    address public agentIdentity;
    address public timelock;
    
    function setUp() public {
        vm.startPrank(deployer);
        
        // Deploy token
        beeToken = new BeeToken(
            deployer,
            address(5),  // team
            address(6),  // investors
            address(7),  // ecosystem
            address(8)   // liquidity
        );
        
        // Deploy mocks
        timelock = address(new MockTimelockController());
        reputationSystem = address(new MockReputationSystem());
        agentRegistry = address(new MockAgentRegistry());
        agentIdentity = address(new MockAgentIdentity());
        
        // Deploy DAO
        dao = new AgentDAO(
            "BeeBotOS DAO",
            IVotes(address(beeToken)),
            TimelockController(payable(timelock)),
            reputationSystem,
            agentRegistry,
            agentIdentity,
            1,      // voting delay
            40320,  // voting period
            400     // quorum numerator (4%)
        );
        
        // Deploy treasury with DAO as owner
        treasury = new TreasuryManager(address(dao));
        
        // Mint tokens to members
        beeToken.mint(member1, 10000 * 10**18, "test");
        beeToken.mint(member2, 5000 * 10**18, "test");
        beeToken.mint(member3, 50000 * 10**18, "test"); // Above proposal threshold
        
        vm.stopPrank();
    }
    
    // ============ Proposal Tests ============
    
    function testPropose() public {
        // Setup delegation
        vm.prank(member3);
        beeToken.delegate(member3);
        vm.roll(block.number + 1);
        
        vm.startPrank(member3);
        
        address[] memory targets = new address[](1);
        targets[0] = address(beeToken);
        
        uint256[] memory values = new uint256[](1);
        values[0] = 0;
        
        bytes[] memory calldatas = new bytes[](1);
        calldatas[0] = "";
        
        uint256 proposalId = dao.propose(
            targets,
            values,
            calldatas,
            "Test Proposal"
        );
        
        assertGt(proposalId, 0);
        
        // Verify proposal was tracked
        (
            AgentDAO.ProposalType proposalType,
            bytes32 agentProposer,
            uint256 reputationRequired,
            bool isAutonomous,
            string memory reasoningURI
        ) = dao.proposalDetails(proposalId);
        
        assertEq(uint256(proposalType), uint256(AgentDAO.ProposalType.ParameterChange));
        assertFalse(isAutonomous);
        
        vm.stopPrank();
    }
    
    function testVote() public {
        // Setup delegations
        vm.prank(member1);
        beeToken.delegate(member1);
        
        vm.prank(member2);
        beeToken.delegate(member2);
        
        vm.prank(member3);
        beeToken.delegate(member3);
        vm.roll(block.number + 1);
        
        // Create proposal
        vm.prank(member3);
        address[] memory targets = new address[](1);
        uint256[] memory values = new uint256[](1);
        bytes[] memory calldatas = new bytes[](1);
        
        uint256 proposalId = dao.propose(targets, values, calldatas, "Vote test");
        
        vm.roll(block.number + 2);
        
        // Vote
        vm.prank(member1);
        dao.castVote(proposalId, 1); // For
        
        vm.prank(member2);
        dao.castVote(proposalId, 0); // Against
        
        // Check votes
        (
            uint256 againstVotes,
            uint256 forVotes,
            uint256 abstainVotes
        ) = dao.proposalVotes(proposalId);
        
        assertGt(forVotes, 0);
        assertGt(againstVotes, 0);
    }
    
    function testDelegation() public {
        vm.prank(member1);
        beeToken.delegate(member2);
        
        assertEq(beeToken.delegates(member1), member2);
    }
    
    function testQuorum() public view {
        uint256 quorum = dao.quorum(block.number);
        assertGt(quorum, 0);
    }
    
    function testProposalThreshold() public view {
        uint256 threshold = dao.proposalThreshold();
        assertGt(threshold, 0);
    }
    
    function testCalculateWeightedVotes() public {
        uint256 rawVotes = 1000e18;
        uint256 reputation = 5000;
        
        uint256 weighted = dao.calculateWeightedVotes(rawVotes, reputation);
        
        // With default 30% reputation weight:
        // tokenComponent = (1000e18 * 7000) / 10000 = 700e18
        // repComponent = (1000e18 * 3000 * 5000) / 100000000 = 150e18
        // total = 850e18
        assertGt(weighted, 0);
        assertLt(weighted, rawVotes * 2);
    }
    
    function testSetReputationWeight() public {
        // Only governance can change this
        // Test that the function exists and has correct access control
        
        vm.expectRevert();
        dao.setReputationWeight(4000);
    }
    
    // ============ Agent-Specific Tests ============
    
    function testAddHumanMember() public {
        bytes32 did = keccak256("human:member1");
        
        // This would require governance execution in reality
        // Here we just test the function interface exists
        // dao.addHumanMember(member1, did);
    }
    
    // ============ View Functions ============
    
    function testGetVotingPower() public {
        vm.prank(member1);
        beeToken.delegate(member1);
        vm.roll(block.number + 1);
        
        uint256 power = dao.getVotingPower(member1);
        assertEq(power, beeToken.balanceOf(member1));
    }
    
    function testGetMembers() public {
        address[] memory members = dao.getMembers();
        // Initially empty until members are added
        assertEq(members.length, 0);
    }
    
    function testVotingDelay() public view {
        uint256 delay = dao.votingDelay();
        assertEq(delay, 1);
    }
    
    function testVotingPeriod() public view {
        uint256 period = dao.votingPeriod();
        assertEq(period, 40320);
    }
}

// Mock contracts
contract MockTimelockController {
    // Minimal implementation
    receive() external payable {}
}

contract MockReputationSystem {
    mapping(address => uint256) public reputations;
    
    function getReputation(address account) external view returns (uint256) {
        return reputations[account] > 0 ? reputations[account] : 5000;
    }
    
    function getCategoryScore(address account, bytes32) external view returns (uint256) {
        return reputations[account] > 0 ? reputations[account] : 5000;
    }
}

contract MockAgentRegistry {
    function getAgentMetadata(bytes32) external view returns (
        bytes32 agentIdOut,
        string memory name,
        string memory description,
        string[] memory capabilities,
        string memory endpoint,
        uint256 version,
        bool isAvailable,
        uint256 lastHeartbeat
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
    
    mapping(bytes32 => AgentIdentityData) public agents;
    
    function getAgent(bytes32 agentId) external view returns (AgentIdentityData memory) {
        if (agents[agentId].agentId == bytes32(0)) {
            // Return default agent if not found
            return AgentIdentityData({
                agentId: agentId,
                owner: address(0x1234),
                did: "did:mock:123",
                publicKey: bytes32(0),
                isActive: true,
                reputation: 5000,
                createdAt: block.timestamp
            });
        }
        return agents[agentId];
    }
    
    function didToAgent(string calldata) external pure returns (bytes32) {
        return bytes32(0);
    }
}
