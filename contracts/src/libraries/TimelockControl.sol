// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/AccessControl.sol";

/**
 * @title TimelockControl
 * @notice Abstract contract for timelock-controlled operations
 * 
 * This contract provides a lightweight timelock mechanism for
 * critical operations. For full timelock functionality, use
 * the standalone Timelock contract with external execution.
 */
abstract contract TimelockControl is AccessControl {
    
    struct PendingOperation {
        bytes32 operationHash;
        uint256 scheduledTime;
        uint256 executionTime;
        bool executed;
        bool cancelled;
    }
    
    // Default delay: 2 days
    uint256 public constant DEFAULT_DELAY = 2 days;
    // Minimum delay: 1 day
    uint256 public constant MIN_DELAY = 1 days;
    // Maximum delay: 14 days
    uint256 public constant MAX_DELAY = 14 days;
    // Grace period for execution
    uint256 public constant GRACE_PERIOD = 7 days;
    
    bytes32 public constant TIMELOCK_ADMIN_ROLE = keccak256("TIMELOCK_ADMIN_ROLE");
    
    mapping(bytes32 => PendingOperation) public pendingOperations;
    bytes32[] public operationHistory;
    
    uint256 public timelockDelay;
    
    event OperationScheduled(
        bytes32 indexed operationHash,
        bytes32 indexed operationType,
        uint256 scheduledTime,
        uint256 executionTime
    );
    event OperationExecuted(bytes32 indexed operationHash, uint256 executionTime);
    event OperationCancelled(bytes32 indexed operationHash, string reason);
    event TimelockDelayChanged(uint256 oldDelay, uint256 newDelay);

    modifier onlyTimelockAdmin() {
        require(
            hasRole(TIMELOCK_ADMIN_ROLE, msg.sender) || hasRole(DEFAULT_ADMIN_ROLE, msg.sender),
            "TimelockControl: not timelock admin"
        );
        _;
    }

    constructor(uint256 _delay) {
        require(_delay >= MIN_DELAY && _delay <= MAX_DELAY, "TimelockControl: invalid delay");
        timelockDelay = _delay;
        _grantRole(TIMELOCK_ADMIN_ROLE, msg.sender);
    }

    /**
     * @dev Schedule an operation for future execution
     * @param operationHash Unique hash of the operation
     * @param operationType Type identifier for the operation
     */
    function _scheduleOperation(bytes32 operationHash, bytes32 operationType) internal {
        require(
            pendingOperations[operationHash].scheduledTime == 0,
            "TimelockControl: already scheduled"
        );
        
        uint256 executionTime = block.timestamp + timelockDelay;
        
        pendingOperations[operationHash] = PendingOperation({
            operationHash: operationHash,
            scheduledTime: block.timestamp,
            executionTime: executionTime,
            executed: false,
            cancelled: false
        });
        
        operationHistory.push(operationHash);
        
        emit OperationScheduled(operationHash, operationType, block.timestamp, executionTime);
    }

    /**
     * @dev Execute a scheduled operation
     * @param operationHash Hash of the operation to execute
     */
    function _executeOperation(bytes32 operationHash) internal {
        PendingOperation storage op = pendingOperations[operationHash];
        
        require(op.scheduledTime != 0, "TimelockControl: not scheduled");
        require(!op.executed, "TimelockControl: already executed");
        require(!op.cancelled, "TimelockControl: cancelled");
        require(block.timestamp >= op.executionTime, "TimelockControl: delay not met");
        require(
            block.timestamp <= op.executionTime + GRACE_PERIOD,
            "TimelockControl: grace period expired"
        );
        
        op.executed = true;
        
        emit OperationExecuted(operationHash, block.timestamp);
    }

    /**
     * @dev Cancel a scheduled operation
     * @param operationHash Hash of the operation to cancel
     * @param reason Reason for cancellation
     */
    function cancelOperation(bytes32 operationHash, string calldata reason) external onlyTimelockAdmin {
        PendingOperation storage op = pendingOperations[operationHash];
        
        require(op.scheduledTime != 0, "TimelockControl: not scheduled");
        require(!op.executed, "TimelockControl: already executed");
        require(!op.cancelled, "TimelockControl: already cancelled");
        
        op.cancelled = true;
        
        emit OperationCancelled(operationHash, reason);
    }

    /**
     * @dev Check if an operation is ready for execution
     */
    function isOperationReady(bytes32 operationHash) external view returns (bool) {
        PendingOperation storage op = pendingOperations[operationHash];
        
        if (op.scheduledTime == 0 || op.executed || op.cancelled) {
            return false;
        }
        
        if (block.timestamp < op.executionTime) {
            return false;
        }
        
        if (block.timestamp > op.executionTime + GRACE_PERIOD) {
            return false;
        }
        
        return true;
    }

    /**
     * @dev Get operation status
     */
    function getOperationStatus(bytes32 operationHash)
        external
        view
        returns (
            bool isScheduled,
            bool isReady,
            bool isExecuted,
            bool isCancelled,
            uint256 scheduledTime,
            uint256 executionTime
        )
    {
        PendingOperation storage op = pendingOperations[operationHash];
        
        isScheduled = op.scheduledTime != 0;
        isExecuted = op.executed;
        isCancelled = op.cancelled;
        scheduledTime = op.scheduledTime;
        executionTime = op.executionTime;
        
        isReady = isScheduled && 
                  !isExecuted && 
                  !isCancelled &&
                  block.timestamp >= executionTime &&
                  block.timestamp <= executionTime + GRACE_PERIOD;
    }

    /**
     * @dev Update timelock delay
     */
    function setTimelockDelay(uint256 newDelay) external onlyTimelockAdmin {
        require(newDelay >= MIN_DELAY && newDelay <= MAX_DELAY, "TimelockControl: invalid delay");
        
        uint256 oldDelay = timelockDelay;
        timelockDelay = newDelay;
        
        emit TimelockDelayChanged(oldDelay, newDelay);
    }

    /**
     * @dev Generate operation hash for critical operations
     */
    function generateOperationHash(
        string memory operationType,
        address target,
        bytes memory data
    ) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(operationType, target, data));
    }
}
