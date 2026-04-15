// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/governance/IGovernor.sol";

/**
 * @title IAgentDAO
 * @notice Interface for the AgentDAO governance contract
 * @dev Extends OpenZeppelin IGovernor with agent-specific features
 * Note: Using abstract contract instead of interface because IGovernor is an abstract contract
 */
abstract contract IAgentDAO is IGovernor {
    
    // ============ Enums ============
    
    enum MemberType {
        None,
        Human,
        Agent,
        Hybrid
    }
    
    enum ProposalType {
        ParameterChange,
        TreasurySpend,
        MemberManagement,
        ContractUpgrade,
        StrategyExecution,
        EmergencyAction
    }
    
    // ============ Structs ============
    
    struct Member {
        MemberType memberType;
        uint256 joinedAt;
        uint256 reputationScore;
        address delegate;
        bool isActive;
        bytes32 did;
    }
    
    struct ProposalInfo {
        ProposalType proposalType;
        bytes32 agentProposer;
        uint256 reputationRequired;
        bool isAutonomous;
        string reasoningURI;
    }
    
    struct VoteInfo {
        uint256 rawVotes;
        uint256 weightedVotes;
        uint256 reputationAtVote;
        bool isAgentVote;
        bytes32 agentVoter;
    }
    
    // ============ Events ============
    
    event MemberAdded(
        address indexed member, 
        MemberType memberType, 
        bytes32 did,
        uint256 reputationScore
    );
    
    event MemberRemoved(address indexed member, uint256 timestamp);
    
    event AgentJoined(
        address indexed agentAddress,
        bytes32 indexed agentId,
        uint256 reputationScore
    );
    
    event AutonomousProposalCreated(
        uint256 indexed proposalId,
        bytes32 indexed agentId,
        string reasoningURI
    );
    
    event VoteCastWithReputation(
        uint256 indexed proposalId,
        address indexed voter,
        uint256 rawWeight,
        uint256 weightedWeight,
        uint256 reputationScore
    );
    
    event DelegationSet(
        address indexed delegator,
        address indexed delegate,
        bool isAgentDelegate
    );
    
    event GovernanceParameterChanged(
        string parameter,
        uint256 oldValue,
        uint256 newValue
    );
    
    event AgentDelegationConfigured(
        address indexed delegator,
        bytes32 indexed agentId,
        string preferencesURI
    );
    
    event AgentBatchVoted(
        address indexed agent,
        uint256[] proposalIds,
        uint8[] _supports
    );
    
    // ============ Member Management ============
    
    /**
     * @notice Add a human member
     * @param _member Member address
     * @param _did Member DID
     */
    function addHumanMember(address _member, bytes32 _did) external virtual;
    
    /**
     * @notice Add an agent member
     * @param _agentAddress Agent contract address
     * @param _agentId Agent DID
     * @param _proof Identity verification proof
     */
    function addAgentMember(
        address _agentAddress,
        bytes32 _agentId,
        bytes calldata _proof
    ) external virtual;
    
    /**
     * @notice Remove a member
     * @param _member Member to remove
     */
    function removeMember(address _member) external virtual;
    
    /**
     * @notice Request to join DAO as an agent
     * @param _agentId Agent DID
     * @param _proof Identity proof
     * @param _justification Join reason (IPFS URI)
     */
    function agentJoinRequest(
        bytes32 _agentId,
        bytes calldata _proof,
        string calldata _justification
    ) external virtual;
    
    /**
     * @notice Get member details
     * @param _member Member address
     * @return Member struct
     */
    function members(address _member) external virtual view returns (Member memory);
    
    /**
     * @notice Check if address is an active member
     */
    function isMember(address _account) external virtual view returns (bool);
    
    /**
     * @notice Get all members
     */
    function getMembers() external virtual view returns (address[] memory);
    
    /**
     * @notice Get member count
     */
    function getMemberCount() external virtual view returns (uint256);
    
    /**
     * @notice Get all agent members
     */
    function getAgentMembers() external virtual view returns (
        address[] memory agentAddresses,
        bytes32[] memory agentDIDs,
        uint256[] memory reputations
    );
    
    // ============ Proposal Functions ============
    
    // Note: propose() is inherited from IGovernor
    
    /**
     * @notice Create an autonomous proposal as an agent
     */
    function autonomousProposal(
        address[] memory _targets,
        uint256[] memory _values,
        bytes[] memory _calldatas,
        string memory _description,
        string memory _reasoningIPFS
    ) external virtual returns (uint256);
    
    /**
     * @notice Check if proposal is autonomous
     */
    function isAutonomousProposal(uint256 proposalId) external virtual view returns (bool);
    
    /**
     * @notice Get proposal details
     */
    function proposalDetails(uint256 proposalId) external virtual view returns (ProposalInfo memory);
    
    // ============ Voting Functions ============
    
    // Note: castVote() is inherited from IGovernor
    
    /**
     * @notice Agent casts vote with reasoning
     */
    function agentCastVote(
        uint256 proposalId,
        uint8 support,
        string calldata reasoningURI
    ) external virtual returns (uint256);
    
    /**
     * @notice Batch vote for multiple proposals
     */
    function agentBatchVote(
        uint256[] calldata proposalIds,
        uint8[] calldata _supports
    ) external virtual returns (uint256[] memory weights);
    
    /**
     * @notice Get vote details
     */
    function getVoteDetails(uint256 proposalId, address voter) external virtual view returns (VoteInfo memory);
    
    /**
     * @notice Calculate voting power with reputation
     */
    function getVotingPower(address account) external virtual view returns (uint256);
    
    /**
     * @notice Get detailed voting power breakdown
     */
    function getVotingPowerBreakdown(address account) external virtual view returns (
        uint256 rawTokenVotes,
        uint256 reputation,
        uint256 tokenComponent,
        uint256 repComponent,
        uint256 totalWeighted
    );
    
    // ============ Delegation ============
    
    /**
     * @notice Set delegation
     */
    function setDelegation(address _delegate) external virtual;
    
    /**
     * @notice Delegate to an agent with preferences
     */
    function delegateToAgent(
        bytes32 _agentId,
        string calldata _votingPreferences
    ) external virtual;
    
    /**
     * @notice Get current delegation
     */
    function delegations(address delegator) external virtual view returns (address);
    
    /**
     * @notice Get delegators for an agent
     */
    function getAgentDelegators(address agent) external virtual view returns (address[] memory);
    
    // ============ Governance Parameters ============
    
    /**
     * @notice Update reputation weight
     */
    function setReputationWeight(uint256 newWeightBps) external virtual;
    
    /**
     * @notice Update proposal threshold
     */
    function setProposalThresholdBps(uint256 newThreshold) external virtual;
    
    /**
     * @notice Check if agent can create autonomous proposals
     */
    function canCreateAutonomousProposal(address agentAddress) external virtual view returns (bool);
    
    // ============ Constants ============
    
    function MIN_REPUTATION_TO_PROPOSE() external virtual pure returns (uint256);
    function MIN_TOKENS_TO_PROPOSE() external virtual pure returns (uint256);
    function MAX_VOTING_PERIOD() external virtual pure returns (uint256);
    function EMERGENCY_VOTING_PERIOD() external virtual pure returns (uint256);
    function reputationWeightBps() external virtual view returns (uint256);
    function proposalThresholdBps() external virtual view returns (uint256);
    
    // ============ External Contracts ============
    
    function reputationSystem() external virtual view returns (address);
    function agentRegistry() external virtual view returns (address);
    function agentIdentity() external virtual view returns (address);
}

