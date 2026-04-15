// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title QuorumManager
 * @dev Manages dynamic quorum based on participation history
 */
contract QuorumManager is Ownable {
    
    struct QuorumConfig {
        uint256 baseQuorum;          // Base quorum percentage (e.g., 40 = 40%)
        uint256 minQuorum;           // Minimum quorum percentage
        uint256 maxQuorum;           // Maximum quorum percentage
        uint256 participationWeight; // Weight of historical participation
        bool dynamicEnabled;         // Whether dynamic quorum is enabled
    }
    
    QuorumConfig public config;
    
    // Historical participation rates
    uint256[] public participationHistory;
    uint256 public constant HISTORY_LENGTH = 10;
    
    // Current calculated quorum
    uint256 public currentQuorum;
    
    event QuorumUpdated(uint256 newQuorum, uint256 averageParticipation);
    event ConfigUpdated(QuorumConfig config);
    
    constructor(
        uint256 _baseQuorum,
        uint256 _minQuorum,
        uint256 _maxQuorum,
        uint256 _participationWeight
    ) {
        config = QuorumConfig({
            baseQuorum: _baseQuorum,
            minQuorum: _minQuorum,
            maxQuorum: _maxQuorum,
            participationWeight: _participationWeight,
            dynamicEnabled: true
        });
        currentQuorum = _baseQuorum;
    }
    
    /**
     * @dev Record participation rate for a vote
     */
    function recordParticipation(uint256 participationRate) external onlyOwner {
        // Add to history
        participationHistory.push(participationRate);
        
        // Keep only recent history
        if (participationHistory.length > HISTORY_LENGTH) {
            // Shift array (inefficient but simple)
            for (uint i = 0; i < participationHistory.length - 1; i++) {
                participationHistory[i] = participationHistory[i + 1];
            }
            participationHistory.pop();
        }
        
        // Recalculate quorum
        if (config.dynamicEnabled) {
            _recalculateQuorum();
        }
    }
    
    /**
     * @dev Calculate and update dynamic quorum
     */
    function _recalculateQuorum() internal {
        if (participationHistory.length == 0) {
            return;
        }
        
        // Calculate average participation
        uint256 sum = 0;
        for (uint i = 0; i < participationHistory.length; i++) {
            sum += participationHistory[i];
        }
        uint256 averageParticipation = sum / participationHistory.length;
        
        // Calculate dynamic quorum
        // Formula: base + (participation - base) * weight / 100
        int256 diff = int256(averageParticipation) - int256(config.baseQuorum);
        int256 adjustment = (diff * int256(config.participationWeight)) / 100;
        
        uint256 newQuorum;
        if (adjustment > 0) {
            newQuorum = config.baseQuorum + uint256(adjustment);
        } else {
            newQuorum = config.baseQuorum - uint256(-adjustment);
        }
        
        // Apply bounds
        if (newQuorum < config.minQuorum) {
            newQuorum = config.minQuorum;
        }
        if (newQuorum > config.maxQuorum) {
            newQuorum = config.maxQuorum;
        }
        
        currentQuorum = newQuorum;
        
        emit QuorumUpdated(newQuorum, averageParticipation);
    }
    
    /**
     * @dev Get the current quorum percentage
     */
    function getQuorum() external view returns (uint256) {
        return currentQuorum;
    }
    
    /**
     * @dev Calculate required quorum amount based on total supply
     */
    function getQuorumAmount(uint256 totalSupply) external view returns (uint256) {
        return (totalSupply * currentQuorum) / 100;
    }
    
    /**
     * @dev Update configuration
     */
    function updateConfig(QuorumConfig calldata _config) external onlyOwner {
        require(_config.minQuorum <= _config.maxQuorum, "Invalid bounds");
        require(_config.baseQuorum >= _config.minQuorum && _config.baseQuorum <= _config.maxQuorum, "Invalid base");
        
        config = _config;
        
        // Recalculate with new config
        if (config.dynamicEnabled) {
            _recalculateQuorum();
        } else {
            currentQuorum = config.baseQuorum;
        }
        
        emit ConfigUpdated(_config);
    }
    
    /**
     * @dev Get participation history
     */
    function getParticipationHistory() external view returns (uint256[] memory) {
        return participationHistory;
    }
    
    /**
     * @dev Get average participation
     */
    function getAverageParticipation() external view returns (uint256) {
        if (participationHistory.length == 0) {
            return 0;
        }
        
        uint256 sum = 0;
        for (uint i = 0; i < participationHistory.length; i++) {
            sum += participationHistory[i];
        }
        return sum / participationHistory.length;
    }
}
