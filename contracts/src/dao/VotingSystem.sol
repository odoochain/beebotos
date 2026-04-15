// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

contract VotingSystem is OwnableUpgradeable {
    
    mapping(address => uint256) public votingPower;
    mapping(uint256 => mapping(address => bool)) public hasVoted;
    mapping(uint256 => uint256) public proposalVotes;
    
    event VoteCast(address indexed voter, uint256 indexed proposalId, bool support, uint256 weight);
    
    function initialize() public initializer {
        __Ownable_init();
    }
    
    function setVotingPower(address voter, uint256 power) external onlyOwner {
        votingPower[voter] = power;
    }
    
    function getVotes(address voter, uint256 blockNumber) external view returns (uint256) {
        return votingPower[voter];
    }
    
    function castVote(uint256 proposalId, bool support) external {
        require(!hasVoted[proposalId][msg.sender], "Already voted");
        
        uint256 weight = votingPower[msg.sender];
        require(weight > 0, "No voting power");
        
        hasVoted[proposalId][msg.sender] = true;
        
        if (support) {
            proposalVotes[proposalId] += weight;
        }
        
        emit VoteCast(msg.sender, proposalId, support, weight);
    }
}
