// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/Pausable.sol";

/**
 * @title ReputationPoints
 * @dev Non-transferable reputation points for DAO governance
 */
contract ReputationPoints is AccessControl, Pausable {
    bytes32 public constant ISSUER_ROLE = keccak256("ISSUER_ROLE");
    bytes32 public constant REVOKER_ROLE = keccak256("REVOKER_ROLE");
    
    struct Reputation {
        uint256 score;
        uint256 issuedAt;
        uint256 lastUpdated;
        mapping(bytes32 => uint256) contributions;
    }
    
    mapping(address => Reputation) public reputations;
    mapping(address => bool) public authorizedAgents;
    
    // Decay parameters
    uint256 public decayRate = 5; // 5% decay per period
    uint256 public decayPeriod = 90 days;
    uint256 public constant MAX_DECAY_PERIODS = 52; // Max 52 periods (~13 years) per call
    
    // Contribution types
    bytes32 public constant CONTRIB_PROPOSAL = keccak256("PROPOSAL");
    bytes32 public constant CONTRIB_VOTE = keccak256("VOTE");
    bytes32 public constant CONTRIB_TASK = keccak256("TASK");
    bytes32 public constant CONTRIB_SKILL = keccak256("SKILL");
    
    event ReputationIssued(address indexed account, uint256 amount, bytes32 contributionType);
    event ReputationRevoked(address indexed account, uint256 amount, string reason);
    event DecayApplied(address indexed account, uint256 oldScore, uint256 newScore);
    
    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ISSUER_ROLE, msg.sender);
        _grantRole(REVOKER_ROLE, msg.sender);
    }
    
    /**
     * @dev Issue reputation points
     */
    function issueReputation(
        address account,
        uint256 amount,
        bytes32 contributionType
    )
        public
        onlyRole(ISSUER_ROLE)
        whenNotPaused
    {
        require(account != address(0), "Invalid address");
        require(amount > 0, "Invalid amount");
        
        Reputation storage rep = reputations[account];
        
        // Apply decay before adding new points
        _applyDecay(account);
        
        rep.score += amount;
        rep.contributions[contributionType] += amount;
        rep.lastUpdated = block.timestamp;
        
        if (rep.issuedAt == 0) {
            rep.issuedAt = block.timestamp;
        }
        
        emit ReputationIssued(account, amount, contributionType);
    }
    
    /**
     * @dev Batch issue reputation
     */
    function batchIssueReputation(
        address[] calldata accounts,
        uint256[] calldata amounts,
        bytes32[] calldata contributionTypes
    )
        external
        onlyRole(ISSUER_ROLE)
        whenNotPaused
    {
        require(
            accounts.length == amounts.length &&
            accounts.length == contributionTypes.length,
            "Length mismatch"
        );
        
        for (uint256 i = 0; i < accounts.length; i++) {
            issueReputation(accounts[i], amounts[i], contributionTypes[i]);
        }
    }
    
    /**
     * @dev Revoke reputation points
     */
    function revokeReputation(
        address account,
        uint256 amount,
        string calldata reason
    )
        external
        onlyRole(REVOKER_ROLE)
        whenNotPaused
    {
        Reputation storage rep = reputations[account];
        require(rep.score >= amount, "Insufficient reputation");
        
        rep.score -= amount;
        rep.lastUpdated = block.timestamp;
        
        emit ReputationRevoked(account, amount, reason);
    }
    
    /**
     * @dev Apply decay to reputation score
     */
    function applyDecay(address account) external whenNotPaused {
        _applyDecay(account);
    }
    
    function _applyDecay(address account) internal {
        Reputation storage rep = reputations[account];
        if (rep.score == 0 || rep.lastUpdated == 0) return;
        
        uint256 periods = (block.timestamp - rep.lastUpdated) / decayPeriod;
        if (periods == 0) return;
        
        // Limit periods to prevent gas exhaustion
        uint256 periodsToProcess = periods > MAX_DECAY_PERIODS ? MAX_DECAY_PERIODS : periods;
        
        uint256 oldScore = rep.score;
        
        // Apply compound decay using optimized calculation
        rep.score = _calculateDecayedScore(oldScore, periodsToProcess);
        
        // Only update lastUpdated by processed periods
        rep.lastUpdated = rep.lastUpdated + (periodsToProcess * decayPeriod);
        
        emit DecayApplied(account, oldScore, rep.score);
    }
    
    /**
     * @dev Calculate decayed score using optimized power calculation
     * Formula: score * (0.95 ^ periods)
     */
    function _calculateDecayedScore(uint256 score, uint256 periods) 
        internal 
        view 
        returns (uint256) 
    {
        if (periods == 0 || score == 0) return score;
        
        // Cap periods to prevent underflow to 0
        if (periods > 500) {
            // After 500 periods at 5% decay, score would be essentially 0
            return 0;
        }
        
        // Use exponentiation by squaring for efficiency
        uint256 decayFactor = 100 - decayRate; // 95
        uint256 result = score;
        uint256 power = periods;
        
        while (power > 0) {
            if (power % 2 == 1) {
                result = (result * decayFactor) / 100;
                if (result == 0) return 0;
            }
            decayFactor = (decayFactor * decayFactor) / 100;
            power /= 2;
        }
        
        return result;
    }
    
    /**
     * @dev Get effective reputation score (with decay applied)
     * Optimized to prevent gas exhaustion
     */
    function getReputation(address account) external view returns (uint256) {
        Reputation storage rep = reputations[account];
        if (rep.score == 0 || rep.lastUpdated == 0) return 0;
        
        uint256 periods = (block.timestamp - rep.lastUpdated) / decayPeriod;
        if (periods == 0) return rep.score;
        
        // Limit periods for view function as well
        if (periods > MAX_DECAY_PERIODS) {
            periods = MAX_DECAY_PERIODS;
        }
        
        return _calculateDecayedScore(rep.score, periods);
    }
    
    /**
     * @dev Get contribution breakdown
     */
    function getContributions(address account)
        external
        view
        returns (
            uint256 proposals,
            uint256 votes,
            uint256 tasks,
            uint256 skills
        )
    {
        Reputation storage rep = reputations[account];
        return (
            rep.contributions[CONTRIB_PROPOSAL],
            rep.contributions[CONTRIB_VOTE],
            rep.contributions[CONTRIB_TASK],
            rep.contributions[CONTRIB_SKILL]
        );
    }
    
    /**
     * @dev Set decay parameters
     */
    function setDecayParameters(
        uint256 newDecayRate,
        uint256 newDecayPeriod
    )
        external
        onlyRole(DEFAULT_ADMIN_ROLE)
    {
        require(newDecayRate <= 50, "Decay rate too high");
        decayRate = newDecayRate;
        decayPeriod = newDecayPeriod;
    }
    
    /**
     * @dev Authorize an agent to earn reputation
     */
    function authorizeAgent(address agent) external onlyRole(DEFAULT_ADMIN_ROLE) {
        authorizedAgents[agent] = true;
    }
    
    /**
     * @dev Revoke agent authorization
     */
    function revokeAgent(address agent) external onlyRole(DEFAULT_ADMIN_ROLE) {
        authorizedAgents[agent] = false;
    }
    
    /**
     * @dev Check if an account is an authorized agent
     */
    function isAuthorizedAgent(address account) external view returns (bool) {
        return authorizedAgents[account];
    }
    
    /**
     * @dev Pause the contract
     */
    function pause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        _pause();
    }
    
    /**
     * @dev Unpause the contract
     */
    function unpause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        _unpause();
    }
}
