// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/governance/Governor.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorSettings.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorCountingSimple.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorVotes.sol";
import "@openzeppelin/contracts/governance/extensions/GovernorTimelockControl.sol";

/**
 * @title AgentGovernor
 * @dev Governor contract for AgentDAO with advanced features
 */
contract AgentGovernor is
    Governor,
    GovernorSettings,
    GovernorCountingSimple,
    GovernorVotes,
    GovernorTimelockControl
{
    // Quorum numerator (percentage of total supply required for quorum)
    uint256 public quorumNumerator;
    
    // Voting power from reputation
    mapping(address => uint256) public reputationPower;
    
    // Proposal threshold including reputation
    uint256 public constant REPUTATION_WEIGHT = 20; // 20% weight for reputation
    
    constructor(
        IVotes _token,
        TimelockController _timelock,
        uint256 _votingDelay,
        uint256 _votingPeriod,
        uint256 _proposalThreshold,
        uint256 _quorumNumerator
    )
        Governor("AgentGovernor")
        GovernorSettings(_votingDelay, _votingPeriod, _proposalThreshold)
        GovernorVotes(_token)
        GovernorTimelockControl(_timelock)
    {
        quorumNumerator = _quorumNumerator;
    }
    
    function quorum(uint256 blockNumber) public view override returns (uint256) {
        uint256 totalSupply = token.getPastTotalSupply(blockNumber);
        return (totalSupply * quorumNumerator) / 100;
    }
    
    function votingDelay()
        public
        view
        override(IGovernor, GovernorSettings)
        returns (uint256)
    {
        return super.votingDelay();
    }
    
    function votingPeriod()
        public
        view
        override(IGovernor, GovernorSettings)
        returns (uint256)
    {
        return super.votingPeriod();
    }
    
    function proposalThreshold()
        public
        view
        override(Governor, GovernorSettings)
        returns (uint256)
    {
        return super.proposalThreshold();
    }
    
    // Get voting power including reputation
    function getVotes(address account, uint256 blockNumber)
        public
        view
        override(IGovernor, Governor)
        returns (uint256)
    {
        uint256 tokenVotes = token.getPastVotes(account, blockNumber);
        uint256 repPower = reputationPower[account];
        
        // Calculate weighted voting power
        return tokenVotes + (repPower * tokenVotes / 100);
    }
    
    // Update reputation power (called by reputation contract)
    function updateReputationPower(address account, uint256 power) external {
        // Add access control
        reputationPower[account] = power;
    }
    
    // The following functions are overrides required by Solidity
    
    function state(uint256 proposalId)
        public
        view
        override(Governor, GovernorTimelockControl)
        returns (ProposalState)
    {
        return super.state(proposalId);
    }
    
    function propose(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        string memory description
    )
        public
        override(Governor, IGovernor)
        returns (uint256)
    {
        return super.propose(targets, values, calldatas, description);
    }
    
    function _execute(
        uint256 proposalId,
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    )
        internal
        override(Governor, GovernorTimelockControl)
    {
        super._execute(proposalId, targets, values, calldatas, descriptionHash);
    }
    
    function _cancel(
        address[] memory targets,
        uint256[] memory values,
        bytes[] memory calldatas,
        bytes32 descriptionHash
    )
        internal
        override(Governor, GovernorTimelockControl)
        returns (uint256)
    {
        return super._cancel(targets, values, calldatas, descriptionHash);
    }
    
    function _executor()
        internal
        view
        override(Governor, GovernorTimelockControl)
        returns (address)
    {
        return super._executor();
    }
    
    function supportsInterface(bytes4 interfaceId)
        public
        view
        override(Governor, GovernorTimelockControl)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }
}
