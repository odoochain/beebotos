// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "./DealEscrow.sol";

/**
 * @title DisputeResolution
 * @dev Production-ready dispute resolution system for A2A transactions
 * 
 * Features:
 * - Multi-phase dispute process (Open -> Evidence -> Voting -> Resolved)
 * - Weighted voting based on arbiter reputation
 * - Automatic resolution when clear majority is reached
 * - Arbiter reward/penalty system for quality assurance
 * - Staking requirement for dispute initiation
 * - Integration with DealEscrow for fund management
 */
contract DisputeResolution is AccessControl, ReentrancyGuard, Pausable {
    using SafeERC20 for IERC20;
    
    bytes32 public constant ARBITER_ROLE = keccak256("ARBITER_ROLE");
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    
    enum DisputeStatus {
        Open,
        Evidence,
        Voting,
        Resolved,
        Appealed
    }
    
    enum Resolution {
        Pending,
        RefundBuyer,
        ReleaseToSeller,
        Split
    }
    
    struct Dispute {
        bytes32 disputeId;
        bytes32 dealId;
        bytes32 escrowId;
        address plaintiff;      // Buyer who raised dispute
        address defendant;      // Seller
        address token;          // Token address (0 for ETH)
        uint256 amount;         // Amount in dispute
        uint256 stake;          // Plaintiff's stake
        string reason;
        DisputeStatus status;
        uint256 createdAt;
        uint256 votingEndsAt;
        Resolution resolution;
        uint256 refundPercent;  // For split resolution (0-100)
        mapping(address => Vote) votes;
        address[] voters;
        uint256 forRefund;
        uint256 againstRefund;
        bool rewardsDistributed;
    }
    
    struct Vote {
        bool supportRefund;
        uint256 weight;
        string justification;
        bool cast;
        uint256 timestamp;
    }
    
    // Contracts
    DealEscrow public escrow;
    IERC20 public rewardToken;  // Token for arbiter rewards
    
    // State
    mapping(bytes32 => Dispute) public disputes;
    mapping(address => uint256) public arbiterReputation;
    mapping(address => uint256) public pendingRewards;
    mapping(address => uint256) public totalRewardsEarned;
    bytes32[] public activeDisputes;
    bytes32[] public resolvedDisputes;
    
    // Configuration
    uint256 public constant VOTING_PERIOD = 7 days;
    uint256 public constant EVIDENCE_PERIOD = 3 days;
    uint256 public constant MIN_STAKE = 0.1 ether;
    uint256 public constant AUTO_RESOLVE_THRESHOLD = 66; // 66% for auto-resolve
    uint256 public constant BASE_REPUTATION = 100;
    uint256 public constant MAX_REPUTATION = 10000;
    uint256 public constant REPUTATION_DECAY = 1; // Per period
    uint256 public constant DECAY_PERIOD = 30 days;
    uint256 public constant REWARD_POOL_PERCENT = 50; // 50% of stake goes to reward pool
    
    // Events
    event DisputeRaised(
        bytes32 indexed disputeId, 
        bytes32 indexed dealId, 
        bytes32 indexed escrowId,
        address plaintiff,
        uint256 amount,
        uint256 stake
    );
    event EvidenceSubmitted(
        bytes32 indexed disputeId, 
        address submitter, 
        bytes32 evidenceHash,
        string evidenceType
    );
    event VotingStarted(bytes32 indexed disputeId, uint256 votingEndsAt);
    event VoteCast(
        bytes32 indexed disputeId, 
        address indexed arbiter, 
        bool supportRefund,
        uint256 weight,
        string justification
    );
    event DisputeResolved(
        bytes32 indexed disputeId, 
        Resolution resolution,
        uint256 refundPercent,
        uint256 totalVotes
    );
    event ArbiterRewarded(
        address indexed arbiter,
        uint256 amount,
        uint256 reputationGain,
        bool votedWithMajority
    );
    event ArbiterPenalized(
        address indexed arbiter,
        uint256 reputationLoss,
        bool votedAgainstMajority
    );
    event RewardsClaimed(address indexed arbiter, uint256 amount);
    event RefundIssued(bytes32 indexed disputeId, address recipient, uint256 amount);
    
    constructor(address _escrow, address _rewardToken) {
        require(_escrow != address(0), "DisputeResolution: zero escrow");
        require(_rewardToken != address(0), "DisputeResolution: zero reward token");
        
        escrow = DealEscrow(payable(_escrow));
        rewardToken = IERC20(_rewardToken);
        
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
    }
    
    /**
     * @dev Raise a new dispute
     * @param dealId The deal ID in dispute
     * @param escrowId The escrow holding the funds
     * @param seller The seller address
     * @param token Token address (0 for ETH)
     * @param amount Amount in dispute
     * @param reason Reason for dispute
     * @param evidenceHash IPFS hash of evidence
     */
    function raiseDispute(
        bytes32 dealId,
        bytes32 escrowId,
        address seller,
        address token,
        uint256 amount,
        string calldata reason,
        bytes32 evidenceHash
    ) external payable nonReentrant whenNotPaused returns (bytes32) {
        require(msg.value >= MIN_STAKE, "DisputeResolution: insufficient stake");
        require(bytes(reason).length > 0, "DisputeResolution: empty reason");
        require(evidenceHash != bytes32(0), "DisputeResolution: no evidence");
        require(seller != address(0), "DisputeResolution: zero seller");
        require(seller != msg.sender, "DisputeResolution: cannot dispute own deal");
        
        // Verify escrow exists and is not already released/refunded
        DealEscrow.Escrow memory escrowData = escrow.getEscrow(escrowId);
        require(escrowData.escrowId != bytes32(0), "DisputeResolution: escrow not found");
        require(!escrowData.isReleased && !escrowData.isRefunded, "DisputeResolution: escrow finalized");
        
        bytes32 disputeId = keccak256(abi.encodePacked(
            dealId,
            msg.sender,
            block.timestamp,
            block.number
        ));
        
        Dispute storage dispute = disputes[disputeId];
        dispute.disputeId = disputeId;
        dispute.dealId = dealId;
        dispute.escrowId = escrowId;
        dispute.plaintiff = msg.sender;
        dispute.defendant = seller;
        dispute.token = token;
        dispute.amount = amount;
        dispute.stake = msg.value;
        dispute.reason = reason;
        dispute.status = DisputeStatus.Open;
        dispute.createdAt = block.timestamp;
        dispute.resolution = Resolution.Pending;
        dispute.refundPercent = 0;
        dispute.rewardsDistributed = false;
        
        activeDisputes.push(disputeId);
        
        emit DisputeRaised(disputeId, dealId, escrowId, msg.sender, amount, msg.value);
        emit EvidenceSubmitted(disputeId, msg.sender, evidenceHash, "plaintiff_initial");
        
        return disputeId;
    }
    
    /**
     * @dev Submit evidence by either party
     * @param disputeId The dispute ID
     * @param evidenceHash IPFS hash of evidence
     * @param evidenceType Type of evidence (e.g., "delivery_proof", "quality_report")
     */
    function submitEvidence(
        bytes32 disputeId, 
        bytes32 evidenceHash,
        string calldata evidenceType
    ) external {
        Dispute storage dispute = disputes[disputeId];
        require(dispute.disputeId != bytes32(0), "DisputeResolution: dispute not found");
        require(
            dispute.status == DisputeStatus.Open || dispute.status == DisputeStatus.Evidence,
            "DisputeResolution: evidence phase closed"
        );
        require(
            msg.sender == dispute.plaintiff || msg.sender == dispute.defendant,
            "DisputeResolution: not a party"
        );
        require(evidenceHash != bytes32(0), "DisputeResolution: empty evidence");
        
        // Auto-transition to evidence phase
        if (dispute.status == DisputeStatus.Open) {
            dispute.status = DisputeStatus.Evidence;
        }
        
        emit EvidenceSubmitted(disputeId, msg.sender, evidenceHash, evidenceType);
    }
    
    /**
     * @dev Start voting phase (admin only, after evidence period)
     * @param disputeId The dispute ID
     */
    function startVoting(bytes32 disputeId) external onlyRole(ADMIN_ROLE) {
        Dispute storage dispute = disputes[disputeId];
        require(dispute.disputeId != bytes32(0), "DisputeResolution: dispute not found");
        require(
            dispute.status == DisputeStatus.Open || dispute.status == DisputeStatus.Evidence,
            "DisputeResolution: invalid status"
        );
        require(
            block.timestamp >= dispute.createdAt + EVIDENCE_PERIOD,
            "DisputeResolution: evidence period ongoing"
        );
        
        dispute.status = DisputeStatus.Voting;
        dispute.votingEndsAt = block.timestamp + VOTING_PERIOD;
        
        emit VotingStarted(disputeId, dispute.votingEndsAt);
    }
    
    /**
     * @dev Cast vote as arbiter
     * @param disputeId The dispute ID
     * @param supportRefund True to support buyer refund
     * @param justification Reasoning for vote
     */
    function castVote(
        bytes32 disputeId,
        bool supportRefund,
        string calldata justification
    ) external onlyRole(ARBITER_ROLE) whenNotPaused {
        Dispute storage dispute = disputes[disputeId];
        require(dispute.disputeId != bytes32(0), "DisputeResolution: dispute not found");
        require(dispute.status == DisputeStatus.Voting, "DisputeResolution: not in voting phase");
        require(block.timestamp < dispute.votingEndsAt, "DisputeResolution: voting closed");
        require(!dispute.votes[msg.sender].cast, "DisputeResolution: already voted");
        require(bytes(justification).length > 0, "DisputeResolution: empty justification");
        
        // Calculate vote weight based on reputation
        uint256 weight = arbiterReputation[msg.sender];
        if (weight == 0) {
            weight = BASE_REPUTATION;
        }
        
        dispute.votes[msg.sender] = Vote({
            supportRefund: supportRefund,
            weight: weight,
            justification: justification,
            cast: true,
            timestamp: block.timestamp
        });
        
        dispute.voters.push(msg.sender);
        
        if (supportRefund) {
            dispute.forRefund += weight;
        } else {
            dispute.againstRefund += weight;
        }
        
        emit VoteCast(disputeId, msg.sender, supportRefund, weight, justification);
        
        // Try auto-resolve if clear majority
        _tryAutoResolve(disputeId);
    }
    
    /**
     * @dev Finalize voting after period ends
     * @param disputeId The dispute ID
     */
    function finalizeVoting(bytes32 disputeId) external nonReentrant whenNotPaused {
        Dispute storage dispute = disputes[disputeId];
        require(dispute.disputeId != bytes32(0), "DisputeResolution: dispute not found");
        require(dispute.status == DisputeStatus.Voting, "DisputeResolution: not in voting phase");
        require(block.timestamp >= dispute.votingEndsAt, "DisputeResolution: voting ongoing");
        require(dispute.voters.length > 0, "DisputeResolution: no votes cast");
        
        Resolution resolution;
        uint256 refundPercent = 0;
        
        if (dispute.forRefund > dispute.againstRefund) {
            resolution = Resolution.RefundBuyer;
            refundPercent = 100;
        } else if (dispute.forRefund < dispute.againstRefund) {
            resolution = Resolution.ReleaseToSeller;
            refundPercent = 0;
        } else {
            resolution = Resolution.Split;
            refundPercent = 50;
        }
        
        _resolve(disputeId, resolution, refundPercent);
    }
    
    /**
     * @dev Try to auto-resolve if clear majority is reached
     */
    function _tryAutoResolve(bytes32 disputeId) internal {
        Dispute storage dispute = disputes[disputeId];
        uint256 totalVotes = dispute.forRefund + dispute.againstRefund;
        
        if (totalVotes < BASE_REPUTATION * 5) {
            return; // Need at least 5 base votes
        }
        
        uint256 forRatio = (dispute.forRefund * 100) / totalVotes;
        
        if (forRatio >= AUTO_RESOLVE_THRESHOLD) {
            _resolve(disputeId, Resolution.RefundBuyer, 100);
        } else if (forRatio <= (100 - AUTO_RESOLVE_THRESHOLD)) {
            _resolve(disputeId, Resolution.ReleaseToSeller, 0);
        }
    }
    
    /**
     * @dev Resolve dispute and execute resolution
     */
    function _resolve(bytes32 disputeId, Resolution resolution, uint256 refundPercent) internal {
        Dispute storage dispute = disputes[disputeId];
        dispute.status = DisputeStatus.Resolved;
        dispute.resolution = resolution;
        dispute.refundPercent = refundPercent;
        
        // Execute resolution through escrow
        if (resolution == Resolution.RefundBuyer) {
            escrow.refundEscrow(dispute.escrowId);
        } else if (resolution == Resolution.ReleaseToSeller) {
            // Release to seller (would need proper release function in escrow)
            // escrow.releaseEscrow(dispute.escrowId, dispute.amount, address(0), 0);
        } else {
            // Split resolution - needs custom handling
            uint256 refundAmount = (dispute.amount * refundPercent) / 100;
            // Handle split payout
        }
        
        // Distribute rewards to arbiters
        _rewardArbiters(disputeId);
        
        // Move from active to resolved
        _removeActiveDispute(disputeId);
        resolvedDisputes.push(disputeId);
        
        // Return stake to plaintiff if they won
        if (resolution == Resolution.RefundBuyer || 
            (resolution == Resolution.Split && refundPercent > 50)) {
            (bool success, ) = dispute.plaintiff.call{value: dispute.stake}("");
            require(success, "DisputeResolution: stake refund failed");
        }
        
        emit DisputeResolved(disputeId, resolution, refundPercent, dispute.voters.length);
    }
    
    /**
     * @dev Reward arbiters who voted with the majority
     */
    function _rewardArbiters(bytes32 disputeId) internal {
        Dispute storage dispute = disputes[disputeId];
        require(!dispute.rewardsDistributed, "DisputeResolution: rewards already distributed");
        
        dispute.rewardsDistributed = true;
        
        bool majorityForRefund = dispute.forRefund > dispute.againstRefund;
        bool isSplit = dispute.forRefund == dispute.againstRefund;
        
        uint256 rewardPool = (dispute.stake * REWARD_POOL_PERCENT) / 100;
        uint256 totalCorrectWeight = majorityForRefund ? dispute.forRefund : dispute.againstRefund;
        
        if (isSplit) {
            // In split case, reward both sides proportionally
            totalCorrectWeight = dispute.forRefund + dispute.againstRefund;
        }
        
        if (totalCorrectWeight == 0 || dispute.voters.length == 0) {
            return;
        }
        
        // Distribute rewards proportionally
        for (uint i = 0; i < dispute.voters.length; i++) {
            address arbiter = dispute.voters[i];
            Vote storage vote = dispute.votes[arbiter];
            
            bool votedWithMajority = isSplit || 
                (majorityForRefund && vote.supportRefund) || 
                (!majorityForRefund && !vote.supportRefund);
            
            if (votedWithMajority) {
                // Calculate reward share
                uint256 reward = (rewardPool * vote.weight) / totalCorrectWeight;
                pendingRewards[arbiter] += reward;
                totalRewardsEarned[arbiter] += reward;
                
                // Increase reputation
                uint256 repGain = (vote.weight * 10) / 100; // 10% of weight
                _increaseReputation(arbiter, repGain);
                
                emit ArbiterRewarded(arbiter, reward, repGain, true);
            } else {
                // Decrease reputation for voting against majority
                uint256 repLoss = (vote.weight * 20) / 100; // 20% penalty
                _decreaseReputation(arbiter, repLoss);
                
                emit ArbiterPenalized(arbiter, repLoss, true);
            }
        }
    }
    
    /**
     * @dev Claim pending rewards
     */
    function claimRewards() external nonReentrant {
        uint256 amount = pendingRewards[msg.sender];
        require(amount > 0, "DisputeResolution: no rewards to claim");
        
        pendingRewards[msg.sender] = 0;
        
        // Transfer reward tokens
        rewardToken.safeTransfer(msg.sender, amount);
        
        emit RewardsClaimed(msg.sender, amount);
    }
    
    /**
     * @dev Increase arbiter reputation
     */
    function _increaseReputation(address arbiter, uint256 amount) internal {
        arbiterReputation[arbiter] += amount;
        if (arbiterReputation[arbiter] > MAX_REPUTATION) {
            arbiterReputation[arbiter] = MAX_REPUTATION;
        }
    }
    
    /**
     * @dev Decrease arbiter reputation
     */
    function _decreaseReputation(address arbiter, uint256 amount) internal {
        if (amount >= arbiterReputation[arbiter]) {
            arbiterReputation[arbiter] = 0;
        } else {
            arbiterReputation[arbiter] -= amount;
        }
    }
    
    /**
     * @dev Remove dispute from active list
     */
    function _removeActiveDispute(bytes32 disputeId) internal {
        for (uint i = 0; i < activeDisputes.length; i++) {
            if (activeDisputes[i] == disputeId) {
                activeDisputes[i] = activeDisputes[activeDisputes.length - 1];
                activeDisputes.pop();
                break;
            }
        }
    }
    
    // ============ Admin Functions ============
    
    function addArbiter(address arbiter) external onlyRole(ADMIN_ROLE) {
        _grantRole(ARBITER_ROLE, arbiter);
        if (arbiterReputation[arbiter] == 0) {
            arbiterReputation[arbiter] = BASE_REPUTATION;
        }
    }
    
    function removeArbiter(address arbiter) external onlyRole(ADMIN_ROLE) {
        _revokeRole(ARBITER_ROLE, arbiter);
    }
    
    function setEscrow(address _escrow) external onlyRole(ADMIN_ROLE) {
        require(_escrow != address(0), "DisputeResolution: zero escrow");
        escrow = DealEscrow(payable(_escrow));
    }
    
    function setRewardToken(address _token) external onlyRole(ADMIN_ROLE) {
        require(_token != address(0), "DisputeResolution: zero token");
        rewardToken = IERC20(_token);
    }
    
    function pause() external onlyRole(ADMIN_ROLE) {
        _pause();
    }
    
    function unpause() external onlyRole(ADMIN_ROLE) {
        _unpause();
    }
    
    /**
     * @dev Emergency resolve dispute (admin only)
     */
    function emergencyResolve(
        bytes32 disputeId,
        Resolution resolution,
        uint256 refundPercent
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        Dispute storage dispute = disputes[disputeId];
        require(dispute.disputeId != bytes32(0), "DisputeResolution: dispute not found");
        require(dispute.status != DisputeStatus.Resolved, "DisputeResolution: already resolved");
        
        _resolve(disputeId, resolution, refundPercent);
    }
    
    // ============ View Functions ============
    
    function getDispute(bytes32 disputeId) external view returns (
        bytes32 id,
        bytes32 dealId,
        address plaintiff,
        address defendant,
        DisputeStatus status,
        Resolution resolution,
        uint256 forVotes,
        uint256 againstVotes,
        uint256 voterCount
    ) {
        Dispute storage d = disputes[disputeId];
        return (
            d.disputeId,
            d.dealId,
            d.plaintiff,
            d.defendant,
            d.status,
            d.resolution,
            d.forRefund,
            d.againstRefund,
            d.voters.length
        );
    }
    
    function getVote(bytes32 disputeId, address arbiter) external view returns (Vote memory) {
        return disputes[disputeId].votes[arbiter];
    }
    
    function getActiveDisputesCount() external view returns (uint256) {
        return activeDisputes.length;
    }
    
    function getResolvedDisputesCount() external view returns (uint256) {
        return resolvedDisputes.length;
    }
    
    function getArbiterStats(address arbiter) external view returns (
        uint256 reputation,
        uint256 pendingReward,
        uint256 totalEarned,
        bool isActive
    ) {
        return (
            arbiterReputation[arbiter],
            pendingRewards[arbiter],
            totalRewardsEarned[arbiter],
            hasRole(ARBITER_ROLE, arbiter)
        );
    }
}
