// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title VotingStrategy
 * @dev Pluggable voting strategies for different proposal types
 */
contract VotingStrategy is Ownable {
    
    enum StrategyType {
        SimpleMajority,
        SuperMajority,
        Quadratic,
        Weighted
    }
    
    struct Strategy {
        StrategyType strategyType;
        uint256 threshold;      // For majority strategies
        uint256 quorum;         // Minimum participation
        bool enabled;
    }
    
    // Strategy for each proposal type
    mapping(bytes32 => Strategy) public strategies;
    
    // Default strategy
    Strategy public defaultStrategy;
    
    event StrategySet(bytes32 indexed proposalType, Strategy strategy);
    event DefaultStrategySet(Strategy strategy);
    
    constructor() {
        defaultStrategy = Strategy({
            strategyType: StrategyType.SimpleMajority,
            threshold: 50,  // 50% for simple majority
            quorum: 40,      // 40% quorum
            enabled: true
        });
    }
    
    /**
     * @dev Set strategy for a proposal type
     */
    function setStrategy(
        bytes32 proposalType,
        StrategyType strategyType,
        uint256 threshold,
        uint256 quorum
    ) external onlyOwner {
        strategies[proposalType] = Strategy({
            strategyType: strategyType,
            threshold: threshold,
            quorum: quorum,
            enabled: true
        });
        
        emit StrategySet(proposalType, strategies[proposalType]);
    }
    
    /**
     * @dev Set default strategy
     */
    function setDefaultStrategy(
        StrategyType strategyType,
        uint256 threshold,
        uint256 quorum
    ) external onlyOwner {
        defaultStrategy = Strategy({
            strategyType: strategyType,
            threshold: threshold,
            quorum: quorum,
            enabled: true
        });
        
        emit DefaultStrategySet(defaultStrategy);
    }
    
    /**
     * @dev Check if proposal passes
     */
    function isPassed(
        bytes32 proposalType,
        uint256 forVotes,
        uint256 againstVotes,
        uint256 abstainVotes,
        uint256 totalSupply
    ) external view returns (bool) {
        Strategy memory strategy = strategies[proposalType].enabled 
            ? strategies[proposalType] 
            : defaultStrategy;
        
        // Check quorum
        uint256 totalVotes = forVotes + againstVotes + abstainVotes;
        uint256 participation = (totalVotes * 100) / totalSupply;
        if (participation < strategy.quorum) {
            return false;
        }
        
        // Apply voting strategy
        if (strategy.strategyType == StrategyType.SimpleMajority) {
            return forVotes > againstVotes;
        } 
        else if (strategy.strategyType == StrategyType.SuperMajority) {
            uint256 total = forVotes + againstVotes;
            if (total == 0) return false;
            return (forVotes * 100) / total >= strategy.threshold;
        }
        else if (strategy.strategyType == StrategyType.Quadratic) {
            // For quadratic voting, compare square roots
            uint256 forSqrt = sqrt(forVotes);
            uint256 againstSqrt = sqrt(againstVotes);
            return forSqrt > againstSqrt;
        }
        else if (strategy.strategyType == StrategyType.Weighted) {
            // Weighted logic would be implemented here
            return forVotes > againstVotes;
        }
        
        return false;
    }
    
    /**
     * @dev Babylonian method for integer square root
     */
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
    
    /**
     * @dev Get strategy for a proposal type
     */
    function getStrategy(bytes32 proposalType) external view returns (Strategy memory) {
        if (strategies[proposalType].enabled) {
            return strategies[proposalType];
        }
        return defaultStrategy;
    }
    
    /**
     * @dev Disable strategy for a proposal type
     */
    function disableStrategy(bytes32 proposalType) external onlyOwner {
        strategies[proposalType].enabled = false;
    }
}
