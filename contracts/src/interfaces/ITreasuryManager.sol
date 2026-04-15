// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title ITreasuryManager
 * @dev Interface for TreasuryManager implementation
 * 
 * This interface matches the actual TreasuryManager contract:
 * - Budget management with categories and time constraints
 * - Streaming payments for continuous vesting
 * - Role-based access control (TREASURY_ADMIN, BUDGET_MANAGER)
 */
interface ITreasuryManager {
    // ============ Enums ============
    
    enum BudgetType { Operational, Development, Marketing, Research, Emergency }
    
    // ============ Structs ============
    
    struct Budget {
        uint256 totalAllocation;
        uint256 spent;
        uint256 startTime;
        uint256 endTime;
        address beneficiary;
        address token;
        BudgetType budgetType;
        bool isActive;
    }
    
    struct StreamingPayment {
        address recipient;
        address token;
        uint256 totalAmount;
        uint256 releasedAmount;
        uint256 startTime;
        uint256 duration;
        bool isActive;
    }
    
    // ============ Events ============
    
    event BudgetCreated(
        uint256 indexed budgetId, 
        address indexed beneficiary, 
        uint256 amount,
        BudgetType budgetType
    );
    event BudgetSpent(
        uint256 indexed budgetId, 
        uint256 amount, 
        string reason,
        address indexed recipient
    );
    event StreamingPaymentCreated(
        uint256 indexed streamId, 
        address indexed recipient, 
        uint256 amount,
        uint256 duration,
        address token
    );
    event StreamReleased(
        uint256 indexed streamId, 
        uint256 amount,
        uint256 totalReleased,
        uint256 remaining
    );
    event BudgetDeactivated(uint256 indexed budgetId, uint256 remainingAmount);
    event StreamCanceled(uint256 indexed streamId, uint256 unreleasedAmount, address recipient);
    event FundsWithdrawn(address indexed token, uint256 amount, address indexed recipient);
    event FundsDeposited(address indexed token, uint256 amount, address indexed sender);
    
    // ============ Budget Functions ============
    
    /**
     * @dev Create a new budget allocation
     * @param beneficiary Address that can spend from budget
     * @param amount Total allocation amount
     * @param token Token address (address(0) for ETH)
     * @param startTime When budget becomes available
     * @param endTime When budget expires
     * @param budgetType Category of budget
     * @return budgetId Unique identifier for this budget
     */
    function createBudget(
        address beneficiary,
        uint256 amount,
        address token,
        uint256 startTime,
        uint256 endTime,
        BudgetType budgetType
    ) external returns (uint256 budgetId);
    
    /**
     * @dev Spend from an existing budget
     * @param budgetId Budget to spend from
     * @param amount Amount to spend
     * @param reason Description of expenditure
     */
    function spendFromBudget(
        uint256 budgetId, 
        uint256 amount, 
        string calldata reason
    ) external;
    
    /**
     * @dev Get budget details
     * @param budgetId Budget identifier
     */
    function budgets(uint256 budgetId) external view returns (
        uint256 totalAllocation,
        uint256 spent,
        uint256 startTime,
        uint256 endTime,
        address beneficiary,
        address token,
        BudgetType budgetType,
        bool isActive
    );
    
    /**
     * @dev Get current budget balance (allocation - spent)
     */
    function getBudgetBalance(uint256 budgetId) external view returns (uint256);
    
    /**
     * @dev Deactivate a budget (admin only)
     */
    function deactivateBudget(uint256 budgetId) external;
    
    // ============ Streaming Payment Functions ============
    
    /**
     * @dev Create a streaming payment (linear vesting)
     * @param recipient Address receiving the stream
     * @param token Token address (address(0) for ETH)
     * @param totalAmount Total amount to stream
     * @param duration Duration of stream in seconds
     * @return streamId Unique identifier for this stream
     */
    function createStreamingPayment(
        address recipient,
        address token,
        uint256 totalAmount,
        uint256 duration
    ) external returns (uint256 streamId);
    
    /**
     * @dev Release available tokens from a stream
     * @param streamId Stream identifier
     * @return amount Amount released
     */
    function releaseStream(uint256 streamId) external returns (uint256 amount);
    
    /**
     * @dev Calculate releasable amount for a stream
     */
    function calculateReleasable(uint256 streamId) external view returns (uint256);
    
    /**
     * @dev Get stream details
     */
    function streams(uint256 streamId) external view returns (
        address recipient,
        address token,
        uint256 totalAmount,
        uint256 releasedAmount,
        uint256 startTime,
        uint256 duration,
        bool isActive
    );
    
    /**
     * @dev Cancel an active stream (admin only)
     */
    function cancelStream(uint256 streamId) external;
    
    // ============ Role Management ============
    
    // Role functions (grantRole, revokeRole, hasRole, DEFAULT_ADMIN_ROLE) 
    // are inherited from OpenZeppelin's AccessControl
    
    /**
     * @dev Get treasury admin role identifier
     */
    function TREASURY_ADMIN() external pure returns (bytes32);
    
    /**
     * @dev Get budget manager role identifier
     */
    function BUDGET_MANAGER() external pure returns (bytes32);
    
    // ============ State Variables ============
    
    function dao() external view returns (address);
    function nextBudgetId() external view returns (uint256);
    function nextStreamId() external view returns (uint256);
    function treasuryBalance(address token) external view returns (uint256);
    
    // ============ Receive ============
    
    receive() external payable;
    
    // ============ Budget Update/Delete ============
    
    /**
     * @dev Update an existing budget
     * @param budgetId Budget to update
     * @param newAmount New total allocation
     * @param newEndTime New end time
     */
    function updateBudget(
        uint256 budgetId,
        uint256 newAmount,
        uint256 newEndTime
    ) external;
    
    /**
     * @dev Delete a budget that has never been used
     * @param budgetId Budget to delete
     */
    function deleteBudget(uint256 budgetId) external;
    
    // ============ Additional Events ============
    
    event BudgetUpdated(uint256 indexed budgetId, uint256 newAmount, uint256 newEndTime);
    event BudgetDeleted(uint256 indexed budgetId);
}
