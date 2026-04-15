// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "../interfaces/IReputationSystem.sol";

/**
 * @title ReputationSystem
 * @notice Production-ready reputation tracking with archival mechanism
 * 
 * Features:
 * - Reputation history with checkpoint archival
 * - Configurable history limit to prevent gas issues
 * - Automatic archival when history grows too large
 * - Pagination support for history queries
 * - Category-based reputation tracking
 * - Voting power calculation with decay
 */
contract ReputationSystem is IReputationSystem, OwnableUpgradeable, PausableUpgradeable, ReentrancyGuardUpgradeable, UUPSUpgradeable {
    
    // ============ Structs ============
    
    struct ReputationData {
        uint256 currentScore;
        uint256 lastUpdateTime;
        // Active history (recent updates)
        uint256[] history;
        uint256[] timestamps;
        // Category scores
        mapping(bytes32 => uint256) categoryScores;
        // Checkpoint data for archived history
        uint256 lastCheckpointTime;
        uint256 checkpointCount;
    }
    
    struct ArchivedHistory {
        Checkpoint[] checkpoints;
        bool isArchived;
    }
    
    // ============ State Variables ============
    
    mapping(address => ReputationData) private reputations;
    mapping(address => mapping(uint256 => ArchivedHistory)) private archivedHistories;
    mapping(address => bool) public authorizedUpdaters;
    mapping(address => uint256) public lastDecayTime;
    
    // Configuration
    uint256 public constant MAX_REPUTATION = 10000;
    uint256 public constant MIN_REPUTATION = 0;
    uint256 public constant DECAY_RATE = 1; // 1% per period
    uint256 public constant DECAY_PERIOD = 7 days;
    uint256 public constant HISTORY_LIMIT = 100; // Max active history entries
    uint256 public constant CHECKPOINT_INTERVAL = 30 days;
    uint256 public constant MAX_DECAY_PERIODS = 52; // Max 52 periods (~1 year) per call
    
    address private immutable __self;
    
    // ============ Events ============
    
    constructor() {
        __self = address(this);
    }
    
    event UpdaterAuthorized(address indexed updater, bool authorized);
    
    // ============ Modifiers ============
    
    modifier onlyAuthorized() {
        require(
            authorizedUpdaters[msg.sender] || owner() == msg.sender,
            "ReputationSystem: not authorized"
        );
        _;
    }
    
    // ============ Initialization ============
    
    function initialize() public initializer {
        __Ownable_init();
        __Pausable_init();
        __ReentrancyGuard_init();
        __UUPSUpgradeable_init();
    }
    
    // ============ Core Functions ============
    
    /**
     * @dev Update reputation with optional reason
     * @param account Account to update
     * @param delta Change in reputation (positive or negative)
     * @param reason Reason for update (IPFS hash or description)
     */
    function updateReputation(
        address account, 
        int256 delta,
        string calldata reason
    ) external onlyAuthorized whenNotPaused {
        _updateReputation(account, delta, reason);
    }
    
    /**
     * @dev Backward compatible update without reason
     */
    function updateReputation(address account, int256 delta) 
        external 
        override 
        onlyAuthorized 
        whenNotPaused 
    {
        _updateReputation(account, delta, "");
    }
    
    function _updateReputation(
        address account, 
        int256 delta,
        string memory reason
    ) internal {
        require(account != address(0), "ReputationSystem: zero address");
        
        ReputationData storage rep = reputations[account];
        
        // Apply decay before update
        _applyDecay(account);
        
        uint256 oldScore = rep.currentScore;
        
        if (delta > 0) {
            rep.currentScore = oldScore + uint256(delta);
            if (rep.currentScore > MAX_REPUTATION) {
                rep.currentScore = MAX_REPUTATION;
            }
        } else {
            uint256 decrease = uint256(-delta);
            if (decrease >= oldScore) {
                rep.currentScore = MIN_REPUTATION;
            } else {
                rep.currentScore = oldScore - decrease;
            }
        }
        
        rep.lastUpdateTime = block.timestamp;
        
        // Add to history
        rep.history.push(rep.currentScore);
        rep.timestamps.push(block.timestamp);
        
        // Check if archiving needed
        if (rep.history.length > HISTORY_LIMIT) {
            _archiveHistory(account);
        }
        
        // Create checkpoint if needed
        if (block.timestamp >= rep.lastCheckpointTime + CHECKPOINT_INTERVAL) {
            _createCheckpoint(account, reason);
        }
        
        emit ReputationUpdated(account, delta, rep.currentScore, reason);
    }
    
    /**
     * @dev Batch update reputation for multiple accounts
     */
    function batchUpdateReputation(
        address[] calldata accounts,
        int256[] calldata deltas,
        string[] calldata reasons
    ) external onlyAuthorized whenNotPaused nonReentrant {
        require(
            accounts.length == deltas.length && deltas.length == reasons.length,
            "ReputationSystem: length mismatch"
        );
        require(accounts.length <= 100, "ReputationSystem: batch too large"); // Prevent gas exhaustion
        
        for (uint i = 0; i < accounts.length; i++) {
            _updateReputation(accounts[i], deltas[i], reasons[i]);
        }
    }
    
    /**
     * @dev Update category-specific score
     */
    function updateCategoryScore(
        address account, 
        bytes32 category, 
        int256 delta
    ) external onlyAuthorized whenNotPaused {
        require(account != address(0), "ReputationSystem: zero address");
        require(category != bytes32(0), "ReputationSystem: empty category");
        
        ReputationData storage rep = reputations[account];
        uint256 current = rep.categoryScores[category];
        
        if (delta > 0) {
            rep.categoryScores[category] = current + uint256(delta);
        } else {
            uint256 decrease = uint256(-delta);
            rep.categoryScores[category] = decrease >= current ? 0 : current - decrease;
        }
        
        emit CategoryScoreUpdated(
            account, 
            category, 
            rep.categoryScores[category],
            delta
        );
    }
    
    // ============ Archival Functions ============
    
    /**
     * @dev Archive old history to save gas on future operations
     * Keeps most recent 20 entries, archives the rest
     */
    function _archiveHistory(address account) internal {
        ReputationData storage rep = reputations[account];
        uint256 totalHistory = rep.history.length;
        
        if (totalHistory <= 20) return;
        
        uint256 toArchive = totalHistory - 20;
        uint256 checkpointId = rep.checkpointCount++;
        
        ArchivedHistory storage archive = archivedHistories[account][checkpointId];
        archive.isArchived = true;
        
        // Create checkpoints from old history
        for (uint i = 0; i < toArchive; i++) {
            archive.checkpoints.push(Checkpoint({
                score: rep.history[i],
                timestamp: rep.timestamps[i],
                blockNumber: block.number, // Note: should store actual block
                reason: "archived"
            }));
        }
        
        // Shift remaining history to front
        for (uint i = 0; i < 20; i++) {
            rep.history[i] = rep.history[totalHistory - 20 + i];
            rep.timestamps[i] = rep.timestamps[totalHistory - 20 + i];
        }
        
        // Trim arrays
        while (rep.history.length > 20) {
            rep.history.pop();
            rep.timestamps.pop();
        }
        
        emit HistoryArchived(account, checkpointId, toArchive, 20);
    }
    
    /**
     * @dev Manually trigger history archival
     */
    function archiveHistory(address account) external onlyOwner {
        _archiveHistory(account);
    }
    
    /**
     * @dev Create a checkpoint for current reputation
     */
    function _createCheckpoint(address account, string memory reason) internal {
        ReputationData storage rep = reputations[account];
        uint256 checkpointId = rep.checkpointCount++;
        
        archivedHistories[account][checkpointId].checkpoints.push(Checkpoint({
            score: rep.currentScore,
            timestamp: block.timestamp,
            blockNumber: block.number,
            reason: reason
        }));
        
        rep.lastCheckpointTime = block.timestamp;
        
        emit CheckpointCreated(account, checkpointId, rep.currentScore, block.timestamp);
    }
    
    /**
     * @dev Manually create checkpoint
     */
    function createCheckpoint(address account, string calldata reason) 
        external 
        onlyAuthorized 
    {
        _createCheckpoint(account, reason);
    }
    
    // ============ Decay Functions ============
    
    /**
     * @dev Apply time-based decay to reputation
     * Decay is 1% per DECAY_PERIOD (7 days)
     * Optimized to prevent gas exhaustion with large period counts
     */
    function _applyDecay(address account) internal {
        uint256 lastDecay = lastDecayTime[account];
        if (lastDecay == 0) {
            lastDecayTime[account] = block.timestamp;
            return;
        }
        
        uint256 timePassed = block.timestamp - lastDecay;
        uint256 periods = timePassed / DECAY_PERIOD;
        
        if (periods == 0) return;
        
        // Limit periods to prevent gas exhaustion
        // If more periods, they'll be processed in subsequent calls
        uint256 periodsToProcess = periods > MAX_DECAY_PERIODS ? MAX_DECAY_PERIODS : periods;
        
        ReputationData storage rep = reputations[account];
        uint256 oldScore = rep.currentScore;
        
        // Apply compound decay using optimized calculation
        // Instead of loop, use pre-computed powers for common periods
        uint256 newScore = _calculateDecayedScore(oldScore, periodsToProcess);
        
        rep.currentScore = newScore;
        // Only update lastDecayTime by the processed periods
        lastDecayTime[account] = lastDecay + (periodsToProcess * DECAY_PERIOD);
        
        emit DecayApplied(account, oldScore, newScore, periodsToProcess);
    }
    
    /**
     * @dev Calculate decayed score using optimized power calculation
     * Formula: score * (0.99 ^ periods)
     */
    function _calculateDecayedScore(uint256 score, uint256 periods) 
        internal 
        pure 
        returns (uint256) 
    {
        if (periods == 0 || score == 0) return score;
        
        // Cap periods to prevent underflow to 0
        if (periods > 500) {
            // After 500 periods at 1% decay, score would be ~0.6% of original
            // Essentially 0 for practical purposes
            return MIN_REPUTATION;
        }
        
        // Use exponentiation by squaring for efficiency
        uint256 decayFactor = 100 - DECAY_RATE; // 99
        uint256 result = score;
        uint256 power = periods;
        
        while (power > 0) {
            if (power % 2 == 1) {
                result = (result * decayFactor) / 100;
                if (result <= MIN_REPUTATION) return MIN_REPUTATION;
            }
            decayFactor = (decayFactor * decayFactor) / 100;
            power /= 2;
        }
        
        return result;
    }
    
    /**
     * @dev Trigger decay calculation for an account
     */
    function applyDecay(address account) external {
        _applyDecay(account);
    }
    
    // ============ View Functions ============
    
    function getReputation(address account) 
        external 
        view 
        override 
        returns (uint256) 
    {
        return reputations[account].currentScore;
    }
    
    /**
     * @dev Get score (alias for getReputation)
     */
    function getScore(address account) 
        external 
        view 
        returns (uint256) 
    {
        return reputations[account].currentScore;
    }
    
    /**
     * @dev Get full reputation data
     */
    function getReputationData(address account) 
        external 
        view 
        returns (
            uint256 currentScore,
            uint256 lastUpdateTime,
            uint256 lastDecay,
            uint256 checkpointCount,
            uint256 historyCount
        ) 
    {
        ReputationData storage rep = reputations[account];
        return (
            rep.currentScore,
            rep.lastUpdateTime,
            lastDecayTime[account],
            rep.checkpointCount,
            rep.history.length
        );
    }
    
    /**
     * @dev Get paginated reputation history
     * @param account Account to query
     * @param offset Start index
     * @param limit Maximum entries to return
     */
    function getReputationHistoryPaginated(
        address account,
        uint256 offset,
        uint256 limit
    ) 
        external 
        view 
        returns (
            uint256[] memory scores,
            uint256[] memory timestamps,
            uint256 total
        ) 
    {
        ReputationData storage rep = reputations[account];
        total = rep.history.length;
        
        if (offset >= total) {
            return (new uint256[](0), new uint256[](0), total);
        }
        
        uint256 end = offset + limit;
        if (end > total) {
            end = total;
        }
        
        uint256 resultLength = end - offset;
        scores = new uint256[](resultLength);
        timestamps = new uint256[](resultLength);
        
        for (uint i = 0; i < resultLength; i++) {
            scores[i] = rep.history[offset + i];
            timestamps[i] = rep.timestamps[offset + i];
        }
    }
    
    /**
     * @dev Get archived history for an account
     */
    function getArchivedHistory(
        address account,
        uint256 checkpointId
    ) 
        external 
        view 
        returns (Checkpoint[] memory) 
    {
        return archivedHistories[account][checkpointId].checkpoints;
    }
    
    /**
     * @dev Get all checkpoint IDs for an account
     */
    function getCheckpointIds(address account) 
        external 
        view 
        returns (uint256[] memory) 
    {
        ReputationData storage rep = reputations[account];
        uint256[] memory ids = new uint256[](rep.checkpointCount);
        for (uint i = 0; i < rep.checkpointCount; i++) {
            ids[i] = i;
        }
        return ids;
    }
    
    function getReputationHistory(address account) 
        external 
        view 
        override 
        returns (uint256[] memory scores, uint256[] memory timestamps) 
    {
        ReputationData storage rep = reputations[account];
        return (rep.history, rep.timestamps);
    }
    
    function getCategoryScore(address account, bytes32 category) 
        external 
        view 
        returns (uint256) 
    {
        return reputations[account].categoryScores[category];
    }
    
    /**
     * @dev Get all category scores for an account
     */
    function getAllCategoryScores(
        address account,
        bytes32[] calldata categories
    ) 
        external 
        view 
        returns (uint256[] memory scores) 
    {
        scores = new uint256[](categories.length);
        for (uint i = 0; i < categories.length; i++) {
            scores[i] = reputations[account].categoryScores[categories[i]];
        }
    }
    
    /**
     * @dev Calculate voting power with decay consideration
     * Uses square root scaling for Sybil resistance
     * Optimized to prevent gas exhaustion with large period counts
     */
    function calculateVotingPower(address account) 
        external 
        view 
        returns (uint256) 
    {
        uint256 rep = reputations[account].currentScore;
        
        // Calculate potential decay with period limit to prevent gas exhaustion
        uint256 lastDecay = lastDecayTime[account];
        if (lastDecay > 0 && rep > MIN_REPUTATION) {
            uint256 timePassed = block.timestamp - lastDecay;
            uint256 periods = timePassed / DECAY_PERIOD;
            
            // Limit periods to prevent gas exhaustion (same logic as _applyDecay)
            if (periods > MAX_DECAY_PERIODS) {
                periods = MAX_DECAY_PERIODS;
            }
            
            // Use optimized decay calculation instead of loop
            rep = _calculateDecayedScore(rep, periods);
        }
        
        // Square root scaling: sqrt(rep * 1e18)
        return sqrt(rep * 1e18);
    }
    
    /**
     * @dev Get detailed reputation stats
     */
    function getReputationStats(address account)
        external
        view
        returns (
            uint256 currentScore,
            uint256 maxScore,
            uint256 minScore,
            uint256 averageScore,
            uint256 updateCount,
            uint256 timeSinceLastUpdate
        )
    {
        ReputationData storage rep = reputations[account];
        currentScore = rep.currentScore;
        maxScore = currentScore;
        minScore = currentScore;
        uint256 sum = 0;
        
        for (uint i = 0; i < rep.history.length; i++) {
            uint256 score = rep.history[i];
            if (score > maxScore) maxScore = score;
            if (score < minScore) minScore = score;
            sum += score;
        }
        
        updateCount = rep.history.length;
        averageScore = updateCount > 0 ? sum / updateCount : currentScore;
        timeSinceLastUpdate = block.timestamp - rep.lastUpdateTime;
    }
    
    // ============ Admin Functions ============
    
    function setAuthorizedUpdater(address updater, bool authorized) 
        external 
        onlyOwner 
    {
        require(updater != address(0), "ReputationSystem: zero address");
        authorizedUpdaters[updater] = authorized;
        emit UpdaterAuthorized(updater, authorized);
    }
    
    function pause() external onlyOwner {
        _pause();
    }
    
    function unpause() external onlyOwner {
        _unpause();
    }
    
    function _authorizeUpgrade(address newImplementation) 
        internal 
        override 
        onlyOwner 
    {}
    
    // ============ Utility Functions ============
    
    function sqrt(uint256 x) internal pure returns (uint256) {
        if (x == 0) return 0;
        uint256 z = (x + 1) / 2;
        uint256 y = x;
        while (z < y) {
            y = z;
            z = (x / z + z) / 2;
        }
        return y;
    }
    
    // Storage gap for upgrade safety
    uint256[50] private __gap;
}
