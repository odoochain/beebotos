// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title IProposalEngine
 * @dev Interface for proposal creation and management
 */
interface IProposalEngine {
    /// @notice Proposal types
    enum ProposalType {
        ParameterChange,
        TreasuryAllocation,
        ContractUpgrade,
        AgentRegistration,
        EmergencyAction
    }

    /// @notice Proposal parameters
    struct ProposalParams {
        ProposalType proposalType;
        bytes data;
        uint256 value;
        address target;
    }

    /// @notice Events
    event ProposalCreated(
        uint256 indexed proposalId,
        address indexed proposer,
        ProposalType proposalType,
        string description
    );

    event ProposalValidated(uint256 indexed proposalId, bool valid);

    /// @notice Create a parameter change proposal
    function createParameterChange(
        address target,
        bytes calldata data,
        string calldata description
    ) external returns (uint256 proposalId);

    /// @notice Create a treasury allocation proposal
    function createTreasuryAllocation(
        address[] calldata recipients,
        uint256[] calldata amounts,
        string calldata description
    ) external returns (uint256 proposalId);

    /// @notice Create a contract upgrade proposal
    function createContractUpgrade(
        address proxy,
        address implementation,
        string calldata description
    ) external returns (uint256 proposalId);

    /// @notice Create an agent registration proposal
    function createAgentRegistration(
        address agent,
        bytes32 capabilities,
        string calldata description
    ) external returns (uint256 proposalId);

    /// @notice Validate a proposal
    function validateProposal(uint256 proposalId) external view returns (bool);

    /// @notice Get proposal parameters
    function getProposalParams(uint256 proposalId) external view returns (ProposalParams memory);

    /// @notice Check if proposal type requires special handling
    function requiresSpecialHandling(ProposalType proposalType) external pure returns (bool);
}
