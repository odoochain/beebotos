// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

contract ProposalEngine is OwnableUpgradeable {
    
    enum ProposalType { Standard, Emergency, ParameterChange, Treasury, Upgrade, Text }
    enum ProposalState { Pending, Active, Canceled, Defeated, Succeeded, Queued, Expired, Executed }
    
    struct Proposal {
        uint256 id;
        address proposer;
        string title;
        bytes callData;
        address target;
        uint256 value;
        ProposalType proposalType;
        uint256 startBlock;
        uint256 endBlock;
        uint256 forVotes;
        uint256 againstVotes;
        bool canceled;
        bool executed;
    }
    
    mapping(uint256 => Proposal) public proposals;
    uint256 public proposalCount;
    
    event ProposalCreated(uint256 indexed id, address indexed proposer, string title);
    
    function propose(string calldata title, address target, bytes calldata callData) external returns (uint256) {
        proposalCount++;
        Proposal storage p = proposals[proposalCount];
        p.id = proposalCount;
        p.proposer = msg.sender;
        p.title = title;
        p.target = target;
        p.callData = callData;
        p.startBlock = block.number + 1;
        p.endBlock = block.number + 40320;
        
        emit ProposalCreated(proposalCount, msg.sender, title);
        return proposalCount;
    }
    
    function castVote(uint256 proposalId, bool support) external {
        Proposal storage p = proposals[proposalId];
        require(block.number >= p.startBlock && block.number <= p.endBlock, "Not active");
        
        if (support) {
            p.forVotes++;
        } else {
            p.againstVotes++;
        }
    }
    
    function execute(uint256 proposalId) external {
        Proposal storage p = proposals[proposalId];
        require(!p.executed && !p.canceled, "Invalid state");
        require(p.forVotes > p.againstVotes, "Not passed");
        
        p.executed = true;
        (bool success, ) = p.target.call{value: p.value}(p.callData);
        require(success, "Execution failed");
    }
}
