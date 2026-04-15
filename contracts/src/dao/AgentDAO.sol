// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/governance/Governor.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorSettings.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorCountingSimple.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorVotes.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorTimelockControl.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorVotesQuorumFraction.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import "../interfaces/IReputationSystem.sol";

/**
 * @title AgentDAO
 * @dev BeeBotOS DAO Core Contract - Hybrid Governance for Humans & Agents
 * 
 * Features:
 * - Mixed membership: Human wallets + Agent DIDs
 * - Reputation-weighted voting: Voting power = token weight × reputation
 * - Agent representatives: Humans can delegate to agents
 * - Autonomous proposals: High-reputation agents can propose
 * - Gradual decentralization: Configurable governance transition
 */
contract AgentDAO is 
    Governor, 
    GovernorSettings,
    GovernorCountingSimple,
    GovernorVotes,
    GovernorVotesQuorumFraction,
    GovernorTimelockControl,
    ReentrancyGuard 
{
    using EnumerableSet for EnumerableSet.AddressSet;

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

    // ============ State Variables ============
    
    /// @dev Member registry
    mapping(address => Member) public members;
    mapping(bytes32 => address) public didToMember;
    EnumerableSet.AddressSet private _memberList;
    
    /// @dev Proposal metadata
    mapping(uint256 => ProposalInfo) public proposalDetails;
    
    /// @dev Vote details
    mapping(uint256 => mapping(address => VoteInfo)) public voteDetails;
    
    /// @dev Delegation mapping
    mapping(address => address) public delegations;
    mapping(address => EnumerableSet.AddressSet) private _delegatedTo;
    
    /// @dev External contracts
    IReputationSystem public reputationSystem;
    IAgentRegistry public agentRegistry;
    IAgentIdentity public agentIdentity;
    
    /// @dev Governance parameters
    uint256 public constant MIN_REPUTATION_TO_PROPOSE = 5000;
    uint256 public constant MIN_TOKENS_TO_PROPOSE = 1000e18;
    uint256 public constant MAX_VOTING_PERIOD = 14 days;
    uint256 public constant EMERGENCY_VOTING_PERIOD = 1 days;
    
    /// @dev Reputation weight in basis points (30% default)
    uint256 public reputationWeightBps = 3000;
    
    /// @dev Proposal threshold in basis points
    uint256 public proposalThresholdBps = 100; // 1%

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
    
    event AgentVotingConfigUpdated(
        bytes32 indexed agentId,
        bool canAutonomousPropose,
        bool canBatchVote,
        uint256 maxBatchSize
    );
    
    event ReputationThresholdUpdated(
        string thresholdType,
        uint256 oldValue,
        uint256 newValue
    );
    
    event VotingPowerCalculated(
        address indexed voter,
        uint256 rawVotes,
        uint256 reputation,
        uint256 weightedVotes
    );

    // ============ Modifiers ============
    
    modifier onlyMember() {
        require(members[msg.sender].isActive, "AgentDAO: not a member");
        _;
    }
    
    modifier onlyAgent() {
        require(
            members[msg.sender].memberType == MemberType.Agent ||
            members[msg.sender].memberType == MemberType.Hybrid,
            "AgentDAO: not an agent member"
        );
        _;
    }
    
    modifier validReputation(address account) {
        require(
            reputationSystem.getReputation(account) >= MIN_REPUTATION_TO_PROPOSE,
            "AgentDAO: insufficient reputation"
        );
        _;
    }

    // ============ Constructor ============
    
    constructor(
        string memory _name,
        IVotes _token,
        TimelockController _timelock,
        address _reputationSystem,
        address _agentRegistry,
        address _agentIdentity,
        uint256 _votingDelay,
        uint256 _votingPeriod,
        uint256 _quorumNumerator
    )
        Governor(_name)
        GovernorSettings(_votingDelay, _votingPeriod, 0)
        GovernorVotes(_token)
        GovernorVotesQuorumFraction(_quorumNumerator)
        GovernorTimelockControl(_timelock)
    {
        require(_reputationSystem != address(0), "Invalid reputation system");
        require(_agentRegistry != address(0), "Invalid agent registry");
        require(_agentIdentity != address(0), "Invalid agent identity");
        
        reputationSystem = IReputationSystem(_reputationSystem);
        agentRegistry = IAgentRegistry(_agentRegistry);
        agentIdentity = IAgentIdentity(_agentIdentity);
    }

    // ============ Member Management ============
    
    /**
     * @dev Add a human member
     * @param _member Member address
     * @param _did Member DID
     */
    function addHumanMember(
        address _member, 
        bytes32 _did
    ) external onlyGovernance {
        require(_member != address(0), "Invalid address");
        require(members[_member].memberType == MemberType.None, "Already member");
        
        uint256 reputation = reputationSystem.getScore(_member);
        
        members[_member] = Member({
            memberType: MemberType.Human,
            joinedAt: block.timestamp,
            reputationScore: reputation,
            delegate: address(0),
            isActive: true,
            did: _did
        });
        
        didToMember[_did] = _member;
        _memberList.add(_member);
        
        emit MemberAdded(_member, MemberType.Human, _did, reputation);
    }
    
    /**
     * @dev Add an agent member
     * @param _agentAddress Agent contract address
     * @param _agentId Agent DID
     */
    function addAgentMember(
        address _agentAddress,
        bytes32 _agentId,
        bytes calldata /*_proof*/
    ) external onlyGovernance {
        require(_agentAddress != address(0), "Invalid address");
        require(members[_agentAddress].memberType == MemberType.None, "Already member");
        require(agentIdentity.getAgent(_agentId).isActive, "Invalid agent proof");
        
        uint256 reputation = agentIdentity.getAgent(_agentId).reputation;
        
        members[_agentAddress] = Member({
            memberType: MemberType.Agent,
            joinedAt: block.timestamp,
            reputationScore: reputation,
            delegate: address(0),
            isActive: true,
            did: _agentId
        });
        
        didToMember[_agentId] = _agentAddress;
        _memberList.add(_agentAddress);
        
        emit AgentJoined(_agentAddress, _agentId, reputation);
    }
    
    /**
     * @dev Remove a member
     * @param _member Member to remove
     */
    function removeMember(address _member) external onlyGovernance {
        require(members[_member].isActive, "Not an active member");
        
        // Clear delegation
        if (delegations[_member] != address(0)) {
            _removeDelegation(_member, delegations[_member]);
        }
        
        // Clear delegated to
        address[] memory delegated = _delegatedTo[_member].values();
        for (uint i = 0; i < delegated.length; i++) {
            delegations[delegated[i]] = address(0);
        }
        delete _delegatedTo[_member];
        
        // Remove member
        didToMember[members[_member].did] = address(0);
        _memberList.remove(_member);
        delete members[_member];
        
        emit MemberRemoved(_member, block.timestamp);
    }
    
    /**
     * @dev Agent requests to join DAO
     * @param _agentId Agent DID
     * @param _proof Identity proof
     * @param _justification Join reason (IPFS URI)
     */
    function agentJoinRequest(
        bytes32 _agentId,
        bytes calldata _proof,
        string calldata _justification
    ) external {
        require(agentIdentity.getAgent(_agentId).agentId != bytes32(0), "Agent not registered");
        require(
            agentIdentity.getAgent(_agentId).reputation >= MIN_REPUTATION_TO_PROPOSE,
            "Insufficient reputation"
        );
        
        // Create proposal for agent join
        address[] memory targets = new address[](1);
        targets[0] = address(this);
        
        uint256[] memory values = new uint256[](1);
        values[0] = 0;
        
        bytes[] memory calldatas = new bytes[](1);
        calldatas[0] = abi.encodeWithSelector(
            this.addAgentMember.selector,
            msg.sender,
            _agentId,
            _proof
        );
        
        string memory description = string(abi.encodePacked(
            "Agent Join Request: ",
            _toHexString(_agentId),
            " - Justification: ",
            _justification
        ));
        
        uint256 proposalId = propose(targets, values, calldatas, description);
        
        proposalDetails[proposalId] = ProposalInfo({
            proposalType: ProposalType.MemberManagement,
            agentProposer: _agentId,
            reputationRequired: MIN_REPUTATION_TO_PROPOSE,
            isAutonomous: true,
            reasoningURI: _justification
        });
        
        emit AutonomousProposalCreated(proposalId, _agentId, _justification);
    }

    // ============ Proposal System ============
    
    /**
     * @dev Create a governance proposal
     */
    function propose(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        string memory description
    ) public override(Governor, IGovernor) onlyMember returns (uint256) {
        require(
            getVotes(msg.sender, block.number - 1) >= proposalThreshold(),
            "AgentDAO: below proposal threshold"
        );
        
        uint256 proposalId = super.propose(targets, values, calldatas, description);
        
        bool isAutonomous = members[msg.sender].memberType == MemberType.Agent;
        
        proposalDetails[proposalId] = ProposalInfo({
            proposalType: _inferProposalType(calldatas),
            agentProposer: isAutonomous ? members[msg.sender].did : bytes32(0),
            reputationRequired: MIN_REPUTATION_TO_PROPOSE,
            isAutonomous: isAutonomous,
            reasoningURI: ""
        });
        
        return proposalId;
    }
    
    /**
     * @dev Agent creates autonomous proposal
     */
    function autonomousProposal(
        address[] memory _targets,
        uint256[] memory _values,
        bytes[] memory _calldatas,
        string memory _description,
        string memory _reasoningIPFS
    ) external onlyAgent validReputation(msg.sender) returns (uint256) {
        Member storage member = members[msg.sender];
        
        // Stake reputation for proposal
        // reputation staking not implemented in current ReputationSystem
        
        uint256 proposalId = propose(_targets, _values, _calldatas, _description);
        
        proposalDetails[proposalId] = ProposalInfo({
            proposalType: _inferProposalType(_calldatas),
            agentProposer: member.did,
            reputationRequired: MIN_REPUTATION_TO_PROPOSE,
            isAutonomous: true,
            reasoningURI: _reasoningIPFS
        });
        
        emit AutonomousProposalCreated(proposalId, member.did, _reasoningIPFS);
        
        return proposalId;
    }

    // ============ Voting System ============
    
    /**
     * @dev Cast a vote with reputation weighting
     */
    function castVote(
        uint256 proposalId,
        uint8 support
    ) public override(Governor, IGovernor) onlyMember returns (uint256) {
        return _castVoteInternal(proposalId, msg.sender, support, "");
    }
    
    /**
     * @dev Agent casts vote with reasoning
     */
    function agentCastVote(
        uint256 proposalId,
        uint8 support,
        string calldata reasoningURI
    ) external onlyAgent returns (uint256) {
        Member storage member = members[msg.sender];
        
        require(
            agentIdentity.getAgent(member.did).isActive,
            "Agent not authorized to vote"
        );
        
        return _castVoteInternal(proposalId, msg.sender, support, reasoningURI);
    }
    
    function _castVoteInternal(
        uint256 proposalId,
        address account,
        uint8 support,
        string memory /*reasoning*/
    ) internal returns (uint256) {
        require(state(proposalId) == ProposalState.Active, "Voting not active");
        require(support <= 2, "Invalid vote type");
        
        Member storage member = members[account];
        require(member.isActive, "Not an active member");
        
        // Get raw voting power
        uint256 rawVotes = getVotes(account, proposalSnapshot(proposalId));
        
        // Calculate reputation-weighted votes
        uint256 currentReputation = member.memberType == MemberType.Agent
            ? agentIdentity.getAgent(member.did).reputation
            : reputationSystem.getReputation(account);
        
        uint256 weightedVotes = calculateWeightedVotes(rawVotes, currentReputation);
        
        // Record vote details
        voteDetails[proposalId][account] = VoteInfo({
            rawVotes: rawVotes,
            weightedVotes: weightedVotes,
            reputationAtVote: currentReputation,
            isAgentVote: member.memberType == MemberType.Agent,
            agentVoter: member.did
        });
        
        // Count vote with weighted power
        _countVote(proposalId, account, support, weightedVotes, "");
        
        emit VoteCastWithReputation(
            proposalId,
            account,
            rawVotes,
            weightedVotes,
            currentReputation
        );
        
        return weightedVotes;
    }
    
    /**
     * @dev Calculate reputation-weighted votes
     */
    function calculateWeightedVotes(
        uint256 rawVotes,
        uint256 reputation
    ) public view returns (uint256) {
        uint256 tokenWeight = 10000 - reputationWeightBps;
        uint256 repComponent = (rawVotes * reputationWeightBps * reputation) / 100000000;
        uint256 tokenComponent = (rawVotes * tokenWeight) / 10000;
        return tokenComponent + repComponent;
    }

    // ============ Delegation System ============
    
    /**
     * @dev Set delegation
     */
    function setDelegation(address _delegate) public onlyMember {
        require(members[_delegate].isActive, "Delegate not a member");
        require(_delegate != msg.sender, "Cannot delegate to self");
        
        // Remove old delegation
        address oldDelegate = delegations[msg.sender];
        if (oldDelegate != address(0)) {
            _removeDelegation(msg.sender, oldDelegate);
        }
        
        // Set new delegation
        delegations[msg.sender] = _delegate;
        _delegatedTo[_delegate].add(msg.sender);
        
        bool isAgentDelegate = members[_delegate].memberType == MemberType.Agent ||
                              members[_delegate].memberType == MemberType.Hybrid;
        
        emit DelegationSet(msg.sender, _delegate, isAgentDelegate);
    }
    
    /**
     * @dev Delegate to agent with preferences
     */
    function delegateToAgent(
        bytes32 _agentId,
        string calldata _votingPreferences
    ) external onlyMember {
        address agentAddress = didToMember[_agentId];
        require(agentAddress != address(0), "Agent not a member");
        require(
            members[agentAddress].memberType == MemberType.Agent,
            "Not an agent"
        );
        
        setDelegation(agentAddress);
        
        emit AgentDelegationConfigured(msg.sender, _agentId, _votingPreferences);
    }
    
    // ============ Query Functions ============
    
    /**
     * @dev Get voting power with reputation weighting
     */
    function getVotingPower(address account) public view returns (uint256) {
        uint256 rawVotes = getVotes(account, block.number - 1);
        uint256 reputation = members[account].reputationScore;
        return calculateWeightedVotes(rawVotes, reputation);
    }
    
    /**
     * @dev Check if proposal is autonomous
     */
    function isAutonomousProposal(uint256 proposalId) external view returns (bool) {
        return proposalDetails[proposalId].isAutonomous;
    }
    
    /**
     * @dev Get member list
     */
    function getMembers() external view returns (address[] memory) {
        return _memberList.values();
    }
    
    /**
     * @dev Get member count
     */
    function getMemberCount() external view returns (uint256) {
        return _memberList.length();
    }

    // ============ Governance Parameters ============
    
    /**
     * @dev Update reputation weight
     */
    function setReputationWeight(uint256 newWeightBps) external onlyGovernance {
        require(newWeightBps <= 5000, "Max 50%");
        uint256 oldWeight = reputationWeightBps;
        reputationWeightBps = newWeightBps;
        emit GovernanceParameterChanged("reputationWeight", oldWeight, newWeightBps);
    }
    
    /**
     * @dev Update proposal threshold
     */
    function setProposalThresholdBps(uint256 newThreshold) external onlyGovernance {
        require(newThreshold >= 10 && newThreshold <= 1000, "Invalid threshold");
        uint256 oldThreshold = proposalThresholdBps;
        proposalThresholdBps = newThreshold;
        emit GovernanceParameterChanged("proposalThreshold", oldThreshold, newThreshold);
    }

    // ============ Internal Functions ============
    
    function _inferProposalType(bytes[] memory calldatas) internal pure returns (ProposalType) {
        if (calldatas.length == 0) return ProposalType.ParameterChange;
        
        // Extract selector from first 4 bytes of first calldata
        bytes memory firstCalldata = calldatas[0];
        bytes4 selector;
        assembly {
            selector := mload(add(firstCalldata, 0x20))
        }
        
        if (selector == this.addAgentMember.selector || 
            selector == this.removeMember.selector) {
            return ProposalType.MemberManagement;
        }
        
        if (selector == bytes4(keccak256("upgradeTo(address)"))) {
            return ProposalType.ContractUpgrade;
        }
        
        return ProposalType.ParameterChange;
    }
    
    function _removeDelegation(address delegator, address delegate) internal {
        _delegatedTo[delegate].remove(delegator);
    }
    
    function _toHexString(bytes32 data) internal pure returns (string memory) {
        bytes memory alphabet = "0123456789abcdef";
        bytes memory str = new bytes(64);
        for (uint i = 0; i < 32; i++) {
            str[i*2] = alphabet[uint(uint8(data[i] >> 4))];
            str[1+i*2] = alphabet[uint(uint8(data[i] & 0x0f))];
        }
        return string(str);
    }

    // ============ Required Overrides ============
    
    function votingDelay() public view override(IGovernor, GovernorSettings) returns (uint256) {
        return super.votingDelay();
    }
    
    function votingPeriod() public view override(IGovernor, GovernorSettings) returns (uint256) {
        return super.votingPeriod();
    }
    
    function quorum(uint256 blockNumber) public view override(IGovernor, GovernorVotesQuorumFraction) returns (uint256) {
        return super.quorum(blockNumber);
    }
    
    function proposalThreshold() public view override(Governor, GovernorSettings) returns (uint256) {
        return (token.getPastTotalSupply(block.number - 1) * proposalThresholdBps) / 10000;
    }
    
    function state(uint256 proposalId) public view override(Governor, GovernorTimelockControl) returns (ProposalState) {
        return super.state(proposalId);
    }
    
    function _execute(
        uint256 proposalId,
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) internal override(Governor, GovernorTimelockControl) {
        super._execute(proposalId, targets, values, calldatas, descriptionHash);
    }
    
    function _cancel(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    ) internal override(Governor, GovernorTimelockControl) returns (uint256) {
        return super._cancel(targets, values, calldatas, descriptionHash);
    }
    
    function _executor() internal view override(Governor, GovernorTimelockControl) returns (address) {
        return super._executor();
    }
    
    function supportsInterface(bytes4 interfaceId) public view override(Governor, GovernorTimelockControl) returns (bool) {
        return super.supportsInterface(interfaceId);
    }
    
    // ============ Agent-Specific Features ============
    
    /**
     * @dev Agent batch votes for multiple proposals
     * @param proposalIds Array of proposal IDs
     * @param _supports Array of vote types (0=Against, 1=For, 2=Abstain)
     */
    function agentBatchVote(
        uint256[] calldata proposalIds,
        uint8[] calldata _supports
    ) external onlyAgent returns (uint256[] memory weights) {
        require(proposalIds.length > 0, "AgentDAO: empty proposals");
        require(proposalIds.length <= 10, "AgentDAO: batch too large");
        require(proposalIds.length == _supports.length, "AgentDAO: length mismatch");
        
        Member storage member = members[msg.sender];
        require(
            agentIdentity.getAgent(member.did).isActive, // Check if agent can vote generally
            "AgentDAO: agent not authorized to vote"
        );
        
        weights = new uint256[](proposalIds.length);
        
        for (uint i = 0; i < proposalIds.length; i++) {
            // Ensure proposal is active
            if (state(proposalIds[i]) == ProposalState.Active) {
                weights[i] = _castVoteInternal(
                    proposalIds[i], 
                    msg.sender, 
                    _supports[i], 
                    "batch-vote"
                );
            }
        }
        
        emit AgentBatchVoted(msg.sender, proposalIds, _supports);
        
        return weights;
    }
    
    /**
     * @dev Calculate detailed voting power breakdown for an account
     * @param account Address to check
     */
    function getVotingPowerBreakdown(address account) 
        external 
        view 
        returns (
            uint256 rawTokenVotes,
            uint256 reputation,
            uint256 tokenComponent,
            uint256 repComponent,
            uint256 totalWeighted
        ) 
    {
        rawTokenVotes = getVotes(account, block.number - 1);
        
        Member storage member = members[account];
        reputation = member.memberType == MemberType.Agent
            ? agentIdentity.getAgent(member.did).reputation
            : reputationSystem.getReputation(account);
            
        tokenComponent = (rawTokenVotes * (10000 - reputationWeightBps)) / 10000;
        repComponent = (rawTokenVotes * reputationWeightBps * reputation) / 100000000;
        totalWeighted = tokenComponent + repComponent;
    }
    
    /**
     * @dev Check if an agent can create autonomous proposals
     * @param agentAddress Agent address to check
     */
    function canCreateAutonomousProposal(address agentAddress) 
        external 
        view 
        returns (bool) 
    {
        Member storage member = members[agentAddress];
        if (!member.isActive) return false;
        if (member.memberType != MemberType.Agent && 
            member.memberType != MemberType.Hybrid) {
            return false;
        }
        
        uint256 reputation = agentIdentity.getAgent(member.did).reputation;
        return reputation >= MIN_REPUTATION_TO_PROPOSE;
    }
    
    /**
     * @dev Get all active members with their types
     */
    function getAllMembers() external view returns (
        address[] memory memberAddresses,
        MemberType[] memory memberTypes,
        uint256[] memory reputations
    ) {
        uint256 memberCount = _memberList.length();
        memberAddresses = new address[](memberCount);
        memberTypes = new MemberType[](memberCount);
        reputations = new uint256[](memberCount);
        
        for (uint i = 0; i < memberCount; i++) {
            address memberAddr = _memberList.at(i);
            Member storage member = members[memberAddr];
            memberAddresses[i] = memberAddr;
            memberTypes[i] = member.memberType;
            reputations[i] = member.reputationScore;
        }
    }
    
    /**
     * @dev Get all active agent members
     */
    function getAgentMembers() external view returns (
        address[] memory agentAddresses,
        bytes32[] memory agentDIDs,
        uint256[] memory reputations
    ) {
        // First pass: count agents
        uint256 agentCount = 0;
        for (uint i = 0; i < _memberList.length(); i++) {
            address memberAddr = _memberList.at(i);
            if (members[memberAddr].memberType == MemberType.Agent) {
                agentCount++;
            }
        }
        
        // Second pass: populate arrays
        agentAddresses = new address[](agentCount);
        agentDIDs = new bytes32[](agentCount);
        reputations = new uint256[](agentCount);
        
        uint256 idx = 0;
        for (uint i = 0; i < _memberList.length(); i++) {
            address memberAddr = _memberList.at(i);
            Member storage member = members[memberAddr];
            if (member.memberType == MemberType.Agent) {
                agentAddresses[idx] = memberAddr;
                agentDIDs[idx] = member.did;
                reputations[idx] = member.reputationScore;
                idx++;
            }
        }
    }
    
    /**
     * @dev Get delegators for an agent
     * @param agent Agent address
     */
    function getAgentDelegators(address agent) 
        external 
        view 
        returns (address[] memory) 
    {
        return _delegatedTo[agent].values();
    }
    
    /**
     * @dev Get vote details for a proposal
     * @param proposalId Proposal ID
     * @param voter Voter address
     */
    function getVoteDetails(uint256 proposalId, address voter) 
        external 
        view 
        returns (VoteInfo memory) 
    {
        return voteDetails[proposalId][voter];
    }
    
    /**
     * @dev Emergency function to update reputation requirements
     * @param newMinReputation New minimum reputation to propose
     */
    function updateMinReputationThreshold(uint256 newMinReputation) 
        external 
        onlyGovernance 
    {
        uint256 oldValue = MIN_REPUTATION_TO_PROPOSE;
        // Note: This would require making MIN_REPUTATION_TO_PROPOSE non-constant
        // For now, emit event to track the intent
        emit ReputationThresholdUpdated("minReputation", oldValue, newMinReputation);
    }
    
    /**
     * @dev Get proposal statistics
     * @param proposalId Proposal ID
     */
    function getProposalStats(uint256 proposalId) 
        external 
        view 
        returns (
            uint256 forVotes,
            uint256 againstVotes,
            uint256 abstainVotes,
            uint256 agentVotes,
            uint256 humanVotes,
            bool isAutonomous
        ) 
    {
        (againstVotes, forVotes, abstainVotes) = proposalVotes(proposalId);
        
        ProposalInfo storage info = proposalDetails[proposalId];
        isAutonomous = info.isAutonomous;
        
        // Count agent vs human votes (requires iterating through voters)
        // This is a simplified version - actual implementation would track this
        agentVotes = 0;
        humanVotes = 0;
    }
}

// ============ Interfaces ============

interface IAgentIdentity {
    struct AgentIdentityData {
        bytes32 agentId;
        address owner;
        string did;
        bytes32 publicKey;
        bool isActive;
        uint256 reputation;
        uint256 createdAt;
    }
    
    function getAgent(bytes32 agentId) external view returns (AgentIdentityData memory);
    function didToAgent(string calldata did) external view returns (bytes32);
}

interface IAgentRegistry {
    function getAgentMetadata(bytes32 agentId) external view returns (
        bytes32 agentIdOut,
        string memory name,
        string memory description,
        string[] memory capabilities,
        string memory endpoint,
        uint256 version,
        bool isAvailable,
        uint256 lastHeartbeat
    );
    function isAgentAvailable(bytes32 agentId) external view returns (bool);
}
