// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title IReputationSystem
 * @dev Interface for ReputationSystem with archival support
 */
interface IReputationSystem {
    
    // ============ Events ============
    
    event ReputationUpdated(
        address indexed account, 
        int256 delta, 
        uint256 newScore,
        string reason
    );
    event CategoryScoreUpdated(
        address indexed account, 
        bytes32 indexed category, 
        uint256 score,
        int256 delta
    );
    event HistoryArchived(
        address indexed account,
        uint256 indexed checkpointId,
        uint256 archivedCount,
        uint256 preservedCount
    );
    event CheckpointCreated(
        address indexed account,
        uint256 indexed checkpointId,
        uint256 score,
        uint256 timestamp
    );
    event DecayApplied(
        address indexed account,
        uint256 oldScore,
        uint256 newScore,
        uint256 periods
    );
    
    // ============ Core Functions ============
    
    /**
     * @dev Update reputation with reason
     */
    function updateReputation(address account, int256 delta, string calldata reason) external;
    
    /**
     * @dev Legacy update without reason
     */
    function updateReputation(address account, int256 delta) external;
    
    /**
     * @dev Batch update reputation
     */
    function batchUpdateReputation(
        address[] calldata accounts,
        int256[] calldata deltas,
        string[] calldata reasons
    ) external;
    
    /**
     * @dev Update category-specific score
     */
    function updateCategoryScore(address account, bytes32 category, int256 delta) external;
    
    /**
     * @dev Apply decay to an account
     */
    function applyDecay(address account) external;
    
    /**
     * @dev Manually create checkpoint
     */
    function createCheckpoint(address account, string calldata reason) external;
    
    /**
     * @dev Archive history for an account
     */
    function archiveHistory(address account) external;
    
    // ============ View Functions ============
    
    /**
     * @dev Get current reputation score
     */
    function getReputation(address account) external view returns (uint256);
    
    /**
     * @dev Get score (alias for getReputation)
     */
    function getScore(address account) external view returns (uint256);
    
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
        );
    
    /**
     * @dev Get paginated reputation history
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
        );
    
    /**
     * @dev Get legacy history (full array)
     */
    function getReputationHistory(address account) 
        external 
        view 
        returns (uint256[] memory scores, uint256[] memory timestamps);
    
    /**
     * @dev Get category score
     */
    function getCategoryScore(address account, bytes32 category) 
        external 
        view 
        returns (uint256);
    
    /**
     * @dev Get all category scores
     */
    function getAllCategoryScores(address account, bytes32[] calldata categories) 
        external 
        view 
        returns (uint256[] memory scores);
    
    /**
     * @dev Calculate voting power
     */
    function calculateVotingPower(address account) external view returns (uint256);
    
    /**
     * @dev Get reputation statistics
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
        );
    
    // ============ Structs ============
    
    struct Checkpoint {
        uint256 score;
        uint256 timestamp;
        uint256 blockNumber;
        string reason;
    }
    
    /**
     * @dev Get archived history
     */
    function getArchivedHistory(address account, uint256 checkpointId) 
        external 
        view 
        returns (Checkpoint[] memory);
    
    /**
     * @dev Get all checkpoint IDs
     */
    function getCheckpointIds(address account) external view returns (uint256[] memory);
    
    // ============ Admin Functions ============
    
    function setAuthorizedUpdater(address updater, bool authorized) external;
    function pause() external;
    function unpause() external;
}
