// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title Timelock
 * @notice Production-ready timelock for critical operations
 * 
 * Features:
 * - Delayed execution for sensitive operations
 * - Role-based access control
 * - Operation cancellation capability
 * - Minimum and maximum delay bounds
 */
contract Timelock is AccessControl, ReentrancyGuard {
    bytes32 public constant PROPOSER_ROLE = keccak256("PROPOSER_ROLE");
    bytes32 public constant EXECUTOR_ROLE = keccak256("EXECUTOR_ROLE");
    bytes32 public constant CANCELLER_ROLE = keccak256("CANCELLER_ROLE");

    // Minimum delay: 2 days
    uint256 public constant MINIMUM_DELAY = 2 days;
    // Maximum delay: 30 days
    uint256 public constant MAXIMUM_DELAY = 30 days;
    // Grace period for execution after ETA
    uint256 public constant GRACE_PERIOD = 14 days;

    uint256 public delay;
    
    mapping(bytes32 => bool) public queuedTransactions;
    mapping(bytes32 => uint256) public transactionETA;
    mapping(bytes32 => bool) public executedTransactions;
    
    uint256 public queuedTransactionCount;
    uint256 public executedTransactionCount;

    event TransactionQueued(
        bytes32 indexed txHash,
        address indexed target,
        uint256 value,
        string signature,
        bytes data,
        uint256 eta
    );
    event TransactionExecuted(
        bytes32 indexed txHash,
        address indexed target,
        uint256 value,
        string signature,
        bytes data,
        uint256 eta
    );
    event TransactionCancelled(bytes32 indexed txHash);
    event DelayChanged(uint256 oldDelay, uint256 newDelay);

    modifier onlyTimelock() {
        require(msg.sender == address(this), "Timelock: caller is not timelock");
        _;
    }

    constructor(
        address admin,
        uint256 _delay
    ) {
        require(admin != address(0), "Timelock: zero admin address");
        require(_delay >= MINIMUM_DELAY, "Timelock: delay below minimum");
        require(_delay <= MAXIMUM_DELAY, "Timelock: delay above maximum");

        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(PROPOSER_ROLE, admin);
        _grantRole(EXECUTOR_ROLE, admin);
        _grantRole(CANCELLER_ROLE, admin);

        delay = _delay;
    }

    /**
     * @dev Queue a transaction for future execution
     * @param target Target contract address
     * @param value ETH value to send
     * @param signature Function signature (e.g., "transfer(address,uint256)")
     * @param data Encoded function parameters
     * @param eta Estimated time of execution (must be >= delay from now)
     * @return txHash Unique hash of the queued transaction
     */
    function queueTransaction(
        address target,
        uint256 value,
        string memory signature,
        bytes memory data,
        uint256 eta
    ) external onlyRole(PROPOSER_ROLE) returns (bytes32 txHash) {
        require(target != address(0), "Timelock: zero target");
        require(
            eta >= block.timestamp + delay,
            "Timelock: eta must satisfy delay"
        );
        require(
            eta <= block.timestamp + delay + GRACE_PERIOD,
            "Timelock: eta too far in future"
        );

        txHash = keccak256(abi.encode(target, value, signature, data, eta));

        require(!queuedTransactions[txHash], "Timelock: already queued");

        queuedTransactions[txHash] = true;
        transactionETA[txHash] = eta;
        queuedTransactionCount++;

        emit TransactionQueued(txHash, target, value, signature, data, eta);

        return txHash;
    }

    /**
     * @dev Cancel a queued transaction
     * @param txHash Transaction hash to cancel
     */
    function cancelTransaction(bytes32 txHash) external onlyRole(CANCELLER_ROLE) {
        require(queuedTransactions[txHash], "Timelock: not queued");
        require(!executedTransactions[txHash], "Timelock: already executed");

        queuedTransactions[txHash] = false;
        delete transactionETA[txHash];
        queuedTransactionCount--;

        emit TransactionCancelled(txHash);
    }

    /**
     * @dev Execute a queued transaction after delay has passed
     * @param target Target contract address
     * @param value ETH value to send
     * @param signature Function signature
     * @param data Encoded function parameters
     * @param eta Estimated time of execution
     */
    function executeTransaction(
        address target,
        uint256 value,
        string memory signature,
        bytes memory data,
        uint256 eta
    ) external payable onlyRole(EXECUTOR_ROLE) nonReentrant {
        bytes32 txHash = keccak256(abi.encode(target, value, signature, data, eta));

        require(queuedTransactions[txHash], "Timelock: not queued");
        require(!executedTransactions[txHash], "Timelock: already executed");
        require(block.timestamp >= eta, "Timelock: delay not satisfied");
        require(block.timestamp <= eta + GRACE_PERIOD, "Timelock: transaction expired");

        queuedTransactions[txHash] = false;
        executedTransactions[txHash] = true;
        executedTransactionCount++;

        bytes memory callData;
        if (bytes(signature).length == 0) {
            callData = data;
        } else {
            callData = abi.encodePacked(bytes4(keccak256(bytes(signature))), data);
        }

        (bool success, ) = target.call{value: value}(callData);
        require(success, "Timelock: execution failed");

        emit TransactionExecuted(txHash, target, value, signature, data, eta);
    }

    /**
     * @dev Update the delay (requires timelock itself to execute)
     * @param newDelay New delay value
     */
    function setDelay(uint256 newDelay) external onlyTimelock {
        require(newDelay >= MINIMUM_DELAY, "Timelock: delay below minimum");
        require(newDelay <= MAXIMUM_DELAY, "Timelock: delay above maximum");

        uint256 oldDelay = delay;
        delay = newDelay;

        emit DelayChanged(oldDelay, newDelay);
    }

    /**
     * @dev Check if a transaction can be executed
     * @param target Target contract address
     * @param value ETH value to send
     * @param signature Function signature
     * @param data Encoded function parameters
     * @param eta Estimated time of execution
     * @return executable Whether the transaction can be executed now
     * @return reason Reason if cannot execute
     */
    function canExecute(
        address target,
        uint256 value,
        string memory signature,
        bytes memory data,
        uint256 eta
    ) external view returns (bool executable, string memory reason) {
        bytes32 txHash = keccak256(abi.encode(target, value, signature, data, eta));

        if (!queuedTransactions[txHash]) {
            return (false, "Not queued");
        }
        if (executedTransactions[txHash]) {
            return (false, "Already executed");
        }
        if (block.timestamp < eta) {
            return (false, "Delay not satisfied");
        }
        if (block.timestamp > eta + GRACE_PERIOD) {
            return (false, "Transaction expired");
        }

        return (true, "");
    }

    /**
     * @dev Get transaction details
     */
    function getTransactionStatus(bytes32 txHash)
        external
        view
        returns (
            bool isQueued,
            bool isExecuted,
            uint256 eta
        )
    {
        return (
            queuedTransactions[txHash],
            executedTransactions[txHash],
            transactionETA[txHash]
        );
    }

    receive() external payable {}
}
