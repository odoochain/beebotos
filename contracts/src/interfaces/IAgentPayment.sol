// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title IAgentPayment
 * @notice Interface for the AgentPayment contract
 * @dev Defines the external functions and events for payment mandates and streams
 */
interface IAgentPayment {
    
    // ============ Structs ============
    
    struct PaymentMandate {
        bytes32 mandateId;
        address payer;
        address payee;
        address token;
        uint256 maxAmount;
        uint256 usedAmount;
        uint256 validUntil;
        bool isActive;
    }
    
    struct Stream {
        bytes32 streamId;
        address sender;
        address recipient;
        address token;
        uint256 totalAmount;
        uint256 releasedAmount;
        uint256 startTime;
        uint256 endTime;
        uint256 lastUpdate;
        bool isActive;
        bool isCancelled;
    }
    
    // ============ Events ============
    
    event MandateCreated(
        bytes32 indexed mandateId, 
        address indexed payer, 
        address indexed payee,
        address token,
        uint256 maxAmount,
        uint256 validUntil
    );
    
    event MandateRevoked(bytes32 indexed mandateId, address indexed revoker);
    
    event PaymentExecuted(
        bytes32 indexed mandateId, 
        bytes32 indexed paymentId, 
        uint256 amount,
        address indexed executor
    );
    
    event StreamCreated(
        bytes32 indexed streamId, 
        address indexed sender, 
        address indexed recipient,
        address token,
        uint256 totalAmount,
        uint256 startTime,
        uint256 endTime
    );
    
    event StreamWithdrawn(
        bytes32 indexed streamId, 
        address indexed recipient, 
        uint256 amount
    );
    
    event StreamCancelled(
        bytes32 indexed streamId, 
        address indexed canceller,
        uint256 recipientRefund,
        uint256 senderRefund
    );
    
    event StreamUpdated(bytes32 indexed streamId, uint256 releasedAmount);
    
    // ============ Mandate Functions ============
    
    /**
     * @notice Creates a new payment mandate
     * @param payee Address authorized to receive payments
     * @param token Token address (address(0) for native ETH)
     * @param maxAmount Maximum total amount that can be paid
     * @param validUntil Timestamp when the mandate expires
     * @return mandateId Unique identifier for the created mandate
     */
    function createMandate(
        address payee,
        address token,
        uint256 maxAmount,
        uint256 validUntil
    ) external returns (bytes32 mandateId);
    
    /**
     * @notice Revokes an existing payment mandate
     * @param mandateId The mandate to revoke
     */
    function revokeMandate(bytes32 mandateId) external;
    
    /**
     * @notice Executes a payment from an existing mandate
     * @param mandateId The mandate to use
     * @param amount The amount to pay
     * @return paymentId Unique identifier for this payment execution
     */
    function executeFromMandate(
        bytes32 mandateId, 
        uint256 amount
    ) external returns (bytes32 paymentId);
    
    // ============ Stream Functions ============
    
    /**
     * @notice Creates a new ETH payment stream
     * @param recipient Address receiving the stream
     * @param totalAmount Total amount to stream
     * @param duration Duration of the stream in seconds
     * @return streamId Unique identifier for the created stream
     */
    function createStream(
        address recipient,
        uint256 totalAmount,
        uint256 duration
    ) external payable returns (bytes32 streamId);
    
    /**
     * @notice Creates a new ERC20 token payment stream
     * @param token Token address
     * @param recipient Address receiving the stream
     * @param totalAmount Total amount to stream
     * @param duration Duration of the stream in seconds
     * @return streamId Unique identifier for the created stream
     */
    function createERC20Stream(
        address token,
        address recipient,
        uint256 totalAmount,
        uint256 duration
    ) external returns (bytes32 streamId);
    
    /**
     * @notice Withdraws available funds from a stream
     * @param streamId The stream to withdraw from
     * @return withdrawnAmount The amount actually withdrawn
     */
    function withdrawFromStream(bytes32 streamId) external returns (uint256 withdrawnAmount);
    
    /**
     * @notice Cancels an active stream
     * @param streamId The stream to cancel
     */
    function cancelStream(bytes32 streamId) external;
    
    // ============ View Functions ============
    
    /**
     * @notice Gets the pending withdrawable amount for a stream
     * @param streamId The stream to query
     * @return The pending amount
     */
    function getPendingAmount(bytes32 streamId) external view returns (uint256);
    
    /**
     * @notice Gets full stream details
     * @param streamId The stream to query
     * @return The stream details
     */
    function getStream(bytes32 streamId) external view returns (Stream memory);
    
    /**
     * @notice Gets a mandate by ID
     * @param mandateId The mandate to query
     * @return The mandate details
     */
    function mandates(bytes32 mandateId) external view returns (PaymentMandate memory);
    
    /**
     * @notice Gets a stream by ID
     * @param streamId The stream to query
     * @return The stream details
     */
    function streams(bytes32 streamId) external view returns (Stream memory);
    
    /**
     * @notice Gets all mandate IDs for a user
     * @param user The user to query
     * @return Array of mandate IDs
     */
    function getUserMandates(address user) external view returns (bytes32[] memory);
    
    /**
     * @notice Gets the remaining available amount in a mandate
     * @param mandateId The mandate to query
     * @return The remaining amount
     */
    function getMandateRemaining(bytes32 mandateId) external view returns (uint256);
    
    // ============ State Variables Getters ============
    
    function mandateCounter() external view returns (uint256);
    function streamCounter() external view returns (uint256);
    function allStreams(uint256 index) external view returns (bytes32);
    function userMandates(address user, uint256 index) external view returns (bytes32);
    
    // ============ Constants ============
    
    function MAX_STREAM_DURATION() external pure returns (uint256);
    function MIN_STREAM_DURATION() external pure returns (uint256);
    function MAX_MANDATE_DURATION() external pure returns (uint256);
    function MIN_MANDATE_VALIDITY() external pure returns (uint256);
}
