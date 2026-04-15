// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/governance/TimelockController.sol";

/**
 * @title DAOTimelock
 * @dev Timelock controller for DAO proposals with emergency features
 */
contract DAOTimelock is TimelockController {
    // Emergency pause
    bool public paused;
    address public guardian;
    
    // Operation expiration
    uint256 public constant GRACE_PERIOD = 14 days;
    
    event EmergencyPause(address indexed guardian);
    event EmergencyUnpause(address indexed guardian);
    
    modifier onlyGuardian() {
        require(msg.sender == guardian, "Timelock: caller is not the guardian");
        _;
    }
    
    modifier whenNotPaused() {
        require(!paused, "Timelock: paused");
        _;
    }
    
    constructor(
        uint256 minDelay,
        address[] memory proposers,
        address[] memory executors,
        address admin
    )
        TimelockController(minDelay, proposers, executors, admin)
    {
        guardian = admin;
    }
    
    /**
     * @dev Emergency pause - prevents any operation execution
     */
    function emergencyPause() external onlyGuardian {
        paused = true;
        emit EmergencyPause(msg.sender);
    }
    
    /**
     * @dev Emergency unpause
     */
    function emergencyUnpause() external onlyGuardian {
        require(paused, "Timelock: not paused");
        paused = false;
        emit EmergencyUnpause(msg.sender);
    }
    
    /**
     * @dev Override execute to add pause check
     */
    function execute(
        address target,
        uint256 value,
        bytes calldata data,
        bytes32 predecessor,
        bytes32 salt
    )
        public
        payable
        override
        whenNotPaused
    {
        super.execute(target, value, data, predecessor, salt);
    }
    
    /**
     * @dev Override executeBatch to add pause check
     */
    function executeBatch(
        address[] calldata targets,
        uint256[] calldata values,
        bytes[] calldata payloads,
        bytes32 predecessor,
        bytes32 salt
    )
        public
        payable
        override
        whenNotPaused
    {
        super.executeBatch(targets, values, payloads, predecessor, salt);
    }
    
    /**
     * @dev Check if operation is expired
     */
    function isOperationExpired(bytes32 id) public view returns (bool) {
        uint256 timestamp = getTimestamp(id);
        require(timestamp != 0, "Timelock: operation non-existent");
        require(timestamp != 1, "Timelock: operation already done");
        return block.timestamp > timestamp + GRACE_PERIOD;
    }
    
    /**
     * @dev Cancel expired operation
     */
    function cancelExpired(bytes32 id) external {
        require(isOperationExpired(id), "Timelock: operation not expired");
        cancel(id);
    }
    
    /**
     * @dev Transfer guardian role
     */
    function transferGuardian(address newGuardian) external onlyGuardian {
        require(newGuardian != address(0), "Timelock: zero address");
        guardian = newGuardian;
    }
}
