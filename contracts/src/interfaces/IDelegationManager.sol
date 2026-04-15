// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title IDelegationManager
 * @dev Interface for vote delegation management
 */
interface IDelegationManager {
    /// @notice Delegation information
    struct Delegation {
        address delegator;
        address delegatee;
        uint256 amount;
        uint256 delegatedAt;
        uint256 expiresAt;
        bool active;
    }

    /// @notice Agent representative information
    struct AgentRepresentative {
        address agent;
        address owner;
        bytes32 capabilities;
        uint256 reputation;
        bool active;
    }

    /// @notice Events
    event DelegateChanged(address indexed delegator, address indexed fromDelegate, address indexed toDelegate);
    event AgentRepRegistered(address indexed agent, address indexed owner, bytes32 capabilities);
    event DelegationExpired(address indexed delegator, address indexed delegatee);

    /// @notice Delegate voting power
    function delegate(address delegatee) external;

    /// @notice Register an agent as a representative
    function registerAgentRep(address agent, bytes32 capabilities) external;

    /// @notice Get current delegate
    function delegates(address account) external view returns (address);

    /// @notice Get voting power
    function getVotingPower(address account) external view returns (uint256);

    /// @notice Get detailed voting power breakdown
    function getVotingPowerDetails(address account) external view returns (
        uint256 basePower,
        uint256 delegatedPower,
        uint256 reputationBonus
    );

    /// @notice Get agent representative info
    function getAgentRep(address agent) external view returns (AgentRepresentative memory);

    /// @notice Get all delegations for a delegatee
    function getDelegationsFor(address delegatee) external view returns (Delegation[] memory);

    /// @notice Revoke delegation
    function revokeDelegation() external;

    /// @notice Check if delegation is expired
    function isDelegationExpired(address delegator) external view returns (bool);
}
