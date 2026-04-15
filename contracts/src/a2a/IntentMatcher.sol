// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "../interfaces/IA2ACommerce.sol";

/**
 * @title IntentMatcher
 * @dev Matches buyer intents with seller capabilities
 */
contract IntentMatcher {
    struct Intent {
        bytes32 id;
        address requester;
        string intentType;
        bytes requirements;
        uint256 maxPrice;
        address paymentToken;
        uint256 deadline;
        IntentStatus status;
    }
    
    struct Match {
        bytes32 intentId;
        address[] candidates;
        uint256[] scores;
        bool finalized;
    }
    
    enum IntentStatus {
        Active,
        Matched,
        Expired,
        Cancelled
    }
    
    mapping(bytes32 => Intent) public intents;
    mapping(bytes32 => Match) public matches;
    
    event IntentCreated(bytes32 indexed intentId, address indexed requester);
    event IntentMatched(bytes32 indexed intentId, address indexed provider);
    event MatchFinalized(bytes32 indexed intentId, address indexed winner);
    
    function createIntent(
        string calldata intentType,
        bytes calldata requirements,
        uint256 maxPrice,
        address paymentToken,
        uint256 deadline
    ) external returns (bytes32) {
        bytes32 intentId = keccak256(abi.encodePacked(
            msg.sender,
            intentType,
            block.timestamp
        ));
        
        intents[intentId] = Intent({
            id: intentId,
            requester: msg.sender,
            intentType: intentType,
            requirements: requirements,
            maxPrice: maxPrice,
            paymentToken: paymentToken,
            deadline: deadline,
            status: IntentStatus.Active
        });
        
        emit IntentCreated(intentId, msg.sender);
        
        return intentId;
    }
    
    function submitMatch(
        bytes32 intentId,
        address[] calldata candidates,
        uint256[] calldata scores
    ) external {
        require(intents[intentId].status == IntentStatus.Active, "Intent not active");
        require(candidates.length == scores.length, "Length mismatch");
        
        matches[intentId] = Match({
            intentId: intentId,
            candidates: candidates,
            scores: scores,
            finalized: false
        });
        
        emit IntentMatched(intentId, candidates[0]);
    }
    
    function finalizeMatch(bytes32 intentId, uint256 candidateIndex) external {
        Intent storage intent = intents[intentId];
        require(msg.sender == intent.requester, "Only requester");
        require(intent.status == IntentStatus.Active, "Intent not active");
        
        Match storage match_ = matches[intentId];
        require(!match_.finalized, "Already finalized");
        require(candidateIndex < match_.candidates.length, "Invalid index");
        
        match_.finalized = true;
        intent.status = IntentStatus.Matched;
        
        emit MatchFinalized(intentId, match_.candidates[candidateIndex]);
    }
    
    function cancelIntent(bytes32 intentId) external {
        Intent storage intent = intents[intentId];
        require(msg.sender == intent.requester, "Only requester");
        require(intent.status == IntentStatus.Active, "Intent not active");
        
        intent.status = IntentStatus.Cancelled;
    }
    
    function getTopMatches(bytes32 intentId, uint256 count) external view returns (address[] memory, uint256[] memory) {
        Match storage match_ = matches[intentId];
        uint256 resultCount = count > match_.candidates.length ? match_.candidates.length : count;
        
        address[] memory topCandidates = new address[](resultCount);
        uint256[] memory topScores = new uint256[](resultCount);
        
        // Simple bubble sort for small arrays
        for (uint256 i = 0; i < resultCount; i++) {
            uint256 maxIndex = i;
            for (uint256 j = i + 1; j < match_.candidates.length; j++) {
                if (match_.scores[j] > match_.scores[maxIndex]) {
                    maxIndex = j;
                }
            }
            topCandidates[i] = match_.candidates[maxIndex];
            topScores[i] = match_.scores[maxIndex];
        }
        
        return (topCandidates, topScores);
    }
}
