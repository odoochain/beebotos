// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title IVotingSystem
 * @dev Interface for voting system with reputation and delegation
 */
interface IVotingSystem {
    /// @notice Vote types
    enum VoteType {
        Against,
        For,
        Abstain
    }

    /// @notice Vote information
    struct Vote {
        uint256 weight;
        uint8 support;
        string reason;
        uint256 timestamp;
    }

    /// @notice Events
    event VoteCast(
        address indexed voter,
        uint256 indexed proposalId,
        uint8 support,
        uint256 weight,
        string reason
    );

    event VotingPowerChanged(address indexed account, uint256 newPower);

    /// @notice Cast a vote
    function castVote(
        uint256 proposalId,
        uint8 support,
        string calldata reason
    ) external returns (uint256 weight);

    /// @notice Get voting power for an account
    function getVotingPower(address account) external view returns (uint256);

    /// @notice Get voting power at a specific block
    function getPastVotingPower(address account, uint256 blockNumber) external view returns (uint256);

    /// @notice Get votes for a proposal
    function getProposalVotes(uint256 proposalId) external view returns (
        uint256 againstVotes,
        uint256 forVotes,
        uint256 abstainVotes
    );

    /// @notice Check if an account has voted
    function hasVoted(uint256 proposalId, address account) external view returns (bool);

    /// @notice Get vote details
    function getVote(uint256 proposalId, address account) external view returns (Vote memory);

    /// @notice Calculate quorum
    function calculateQuorum(uint256 totalSupply) external view returns (uint256);

    /// @notice Check if proposal passed
    function hasPassed(uint256 proposalId) external view returns (bool);
}
