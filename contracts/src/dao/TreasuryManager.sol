// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "../interfaces/ITreasuryManager.sol";
import "../libraries/TimelockControl.sol";

/**
 * @title TreasuryManager
 * @dev Production-ready DAO Treasury with Budget Control and Streaming Payments
 * 
 * Features:
 * - Role-based access control (TREASURY_ADMIN, BUDGET_MANAGER)
 * - Time-bounded budgets with categories
 * - Linear streaming payments for continuous payouts
 * - ETH and ERC20 support
 * - Emergency pause functionality
 * - Comprehensive events and view functions
 */
contract TreasuryManager is ITreasuryManager, ReentrancyGuard, AccessControl, Pausable, TimelockControl {
    using SafeERC20 for IERC20;

    bytes32 public constant TREASURY_ADMIN = keccak256("TREASURY_ADMIN");
    bytes32 public constant BUDGET_MANAGER = keccak256("BUDGET_MANAGER");

    // ============ State Variables ============
    
    address public dao;
    mapping(uint256 => Budget) public budgets;
    mapping(uint256 => StreamingPayment) public streams;
    uint256 public nextBudgetId = 1;
    uint256 public nextStreamId = 1;
    mapping(address => uint256) public treasuryBalance;
    
    // Events are defined in ITreasuryManager interface
    
    // ============ Modifiers ============
    
    modifier onlyDAO() {
        require(msg.sender == dao, "TreasuryManager: only DAO");
        _;
    }
    
    modifier validBudget(uint256 budgetId) {
        require(budgets[budgetId].startTime > 0, "TreasuryManager: budget not found");
        _;
    }
    
    modifier validStream(uint256 streamId) {
        require(streams[streamId].startTime > 0, "TreasuryManager: stream not found");
        _;
    }
    
    // ============ Constructor ============
    
    constructor(address _dao) TimelockControl(2 days) {
        require(_dao != address(0), "TreasuryManager: zero DAO address");
        dao = _dao;
        
        _grantRole(DEFAULT_ADMIN_ROLE, _dao);
        _grantRole(TREASURY_ADMIN, _dao);
        _grantRole(TIMELOCK_ADMIN_ROLE, _dao);
    }
    
    // ============ Budget Functions ============
    
    /**
     * @dev Create a new budget allocation
     * @param _beneficiary Address that can spend from budget
     * @param _amount Total allocation amount
     * @param _token Token address (address(0) for ETH)
     * @param _startTime When budget becomes available
     * @param _endTime When budget expires
     * @param _type Category of budget
     * @return budgetId Unique identifier for this budget
     */
    function createBudget(
        address _beneficiary,
        uint256 _amount,
        address _token,
        uint256 _startTime,
        uint256 _endTime,
        BudgetType _type
    ) external onlyRole(TREASURY_ADMIN) returns (uint256 budgetId) {
        require(_beneficiary != address(0), "TreasuryManager: zero beneficiary");
        require(_amount > 0, "TreasuryManager: zero amount");
        require(_startTime < _endTime, "TreasuryManager: invalid time range");
        require(_startTime >= block.timestamp || _startTime == 0, "TreasuryManager: start in past");
        
        budgetId = nextBudgetId++;
        budgets[budgetId] = Budget({
            totalAllocation: _amount,
            spent: 0,
            startTime: _startTime == 0 ? block.timestamp : _startTime,
            endTime: _endTime,
            beneficiary: _beneficiary,
            token: _token,
            budgetType: _type,
            isActive: true
        });
        
        emit BudgetCreated(budgetId, _beneficiary, _amount, _type);
        return budgetId;
    }
    
    /**
     * @dev Spend from an existing budget
     * @param _budgetId Budget to spend from
     * @param _amount Amount to spend
     * @param _reason Description of expenditure
     */
    function spendFromBudget(
        uint256 _budgetId, 
        uint256 _amount, 
        string calldata _reason
    ) 
        external 
        onlyRole(BUDGET_MANAGER) 
        validBudget(_budgetId)
        whenNotPaused 
    {
        Budget storage budget = budgets[_budgetId];
        require(budget.isActive, "TreasuryManager: budget not active");
        require(block.timestamp >= budget.startTime, "TreasuryManager: budget not started");
        require(block.timestamp <= budget.endTime, "TreasuryManager: budget expired");
        require(budget.spent + _amount <= budget.totalAllocation, "TreasuryManager: over budget");
        require(bytes(_reason).length > 0, "TreasuryManager: empty reason");
        
        budget.spent += _amount;
        _transfer(budget.token, budget.beneficiary, _amount);
        
        emit BudgetSpent(_budgetId, _amount, _reason, budget.beneficiary);
    }
    
    /**
     * @dev Deactivate a budget and return remaining funds to treasury
     * @param _budgetId Budget to deactivate
     */
    function deactivateBudget(uint256 _budgetId) 
        external 
        onlyRole(TREASURY_ADMIN) 
        validBudget(_budgetId) 
    {
        Budget storage budget = budgets[_budgetId];
        require(budget.isActive, "TreasuryManager: budget not active");
        
        uint256 remaining = budget.totalAllocation - budget.spent;
        budget.isActive = false;
        
        emit BudgetDeactivated(_budgetId, remaining);
    }
    
    /**
     * @dev Get budget balance (allocation - spent)
     * @param _budgetId Budget identifier
     */
    function getBudgetBalance(uint256 _budgetId) 
        external 
        view 
        validBudget(_budgetId) 
        returns (uint256) 
    {
        Budget storage budget = budgets[_budgetId];
        if (budget.spent >= budget.totalAllocation) {
            return 0;
        }
        return budget.totalAllocation - budget.spent;
    }
    
    // ============ Streaming Payment Functions ============
    
    /**
     * @dev Create a streaming payment (linear vesting)
     * @param _recipient Address receiving the stream
     * @param _token Token address (address(0) for ETH)
     * @param _totalAmount Total amount to stream
     * @param _duration Duration of stream in seconds
     * @return streamId Unique identifier for this stream
     */
    function createStreamingPayment(
        address _recipient,
        address _token,
        uint256 _totalAmount,
        uint256 _duration
    ) external onlyRole(TREASURY_ADMIN) returns (uint256 streamId) {
        require(_recipient != address(0), "TreasuryManager: zero recipient");
        require(_totalAmount > 0, "TreasuryManager: zero amount");
        require(_duration > 0, "TreasuryManager: zero duration");
        
        streamId = nextStreamId++;
        streams[streamId] = StreamingPayment({
            recipient: _recipient,
            token: _token,
            totalAmount: _totalAmount,
            releasedAmount: 0,
            startTime: block.timestamp,
            duration: _duration,
            isActive: true
        });
        
        emit StreamingPaymentCreated(
            streamId, 
            _recipient, 
            _totalAmount, 
            _duration,
            _token
        );
        return streamId;
    }
    
    /**
     * @dev Release available tokens from a stream
     * @param _streamId Stream identifier
     * @return amount Amount released
     */
    function releaseStream(uint256 _streamId) 
        external 
        nonReentrant 
        validStream(_streamId) 
        whenNotPaused
        returns (uint256 amount) 
    {
        StreamingPayment storage stream = streams[_streamId];
        require(stream.isActive, "TreasuryManager: stream not active");
        require(
            msg.sender == stream.recipient || hasRole(TREASURY_ADMIN, msg.sender),
            "TreasuryManager: not authorized"
        );
        
        amount = _calculateReleasable(stream);
        require(amount > 0, "TreasuryManager: nothing to release");
        
        stream.releasedAmount += amount;
        
        // Check if stream is complete
        if (stream.releasedAmount >= stream.totalAmount) {
            stream.isActive = false;
        }
        
        _transfer(stream.token, stream.recipient, amount);
        
        emit StreamReleased(
            _streamId, 
            amount, 
            stream.releasedAmount,
            stream.totalAmount - stream.releasedAmount
        );
        
        return amount;
    }
    
    /**
     * @dev Calculate releasable amount for a stream
     * @param _streamId Stream identifier
     */
    function calculateReleasable(uint256 _streamId) 
        external 
        view 
        validStream(_streamId) 
        returns (uint256) 
    {
        return _calculateReleasable(streams[_streamId]);
    }
    
    /**
     * @dev Internal function to calculate releasable amount
     */
    function _calculateReleasable(StreamingPayment storage stream) 
        internal 
        view 
        returns (uint256) 
    {
        if (!stream.isActive) {
            return 0;
        }
        
        uint256 elapsed = block.timestamp - stream.startTime;
        
        if (elapsed >= stream.duration) {
            return stream.totalAmount - stream.releasedAmount;
        }
        
        uint256 vested = (stream.totalAmount * elapsed) / stream.duration;
        
        if (vested <= stream.releasedAmount) {
            return 0;
        }
        
        return vested - stream.releasedAmount;
    }
    
    /**
     * @dev Cancel an active stream and return unreleased funds to treasury
     * @param _streamId Stream to cancel
     */
    function cancelStream(uint256 _streamId) 
        external 
        onlyRole(TREASURY_ADMIN) 
        validStream(_streamId) 
    {
        StreamingPayment storage stream = streams[_streamId];
        require(stream.isActive, "TreasuryManager: stream not active");
        
        uint256 unreleased = stream.totalAmount - stream.releasedAmount;
        stream.isActive = false;
        
        emit StreamCanceled(_streamId, unreleased, stream.recipient);
    }
    
    // ============ Admin Functions ============
    
    /**
     * @dev Emergency pause
     */
    function pause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        _pause();
    }
    
    /**
     * @dev Unpause
     */
    function unpause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        _unpause();
    }
    
    // ============ Timelock Operations ============
    
    bytes32 public constant OP_SET_DAO = keccak256("OP_SET_DAO");
    bytes32 public constant OP_EMERGENCY_WITHDRAW = keccak256("OP_EMERGENCY_WITHDRAW");
    
    /**
     * @dev Schedule DAO address update (requires timelock)
     * @param _newDAO New DAO address
     */
    function scheduleSetDAO(address _newDAO) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(_newDAO != address(0), "TreasuryManager: zero address");
        
        bytes memory data = abi.encode(_newDAO);
        bytes32 opHash = generateOperationHash("setDAO", address(this), data);
        _scheduleOperation(opHash, OP_SET_DAO);
    }
    
    /**
     * @dev Execute scheduled DAO address update
     * @param _newDAO New DAO address (must match scheduled)
     */
    function executeSetDAO(address _newDAO) external onlyRole(DEFAULT_ADMIN_ROLE) {
        bytes memory data = abi.encode(_newDAO);
        bytes32 opHash = generateOperationHash("setDAO", address(this), data);
        _executeOperation(opHash);
        
        address oldDAO = dao;
        dao = _newDAO;
        
        // Update roles
        _grantRole(DEFAULT_ADMIN_ROLE, _newDAO);
        _grantRole(TREASURY_ADMIN, _newDAO);
        _revokeRole(DEFAULT_ADMIN_ROLE, oldDAO);
        _revokeRole(TREASURY_ADMIN, oldDAO);
    }
    
    /**
     * @dev Schedule emergency withdrawal (requires timelock for large amounts)
     * @param _token Token to withdraw
     * @param _to Recipient
     * @param _amount Amount
     */
    function scheduleEmergencyWithdraw(
        address _token,
        address _to,
        uint256 _amount
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(_to != address(0), "TreasuryManager: zero recipient");
        require(_amount > 0, "TreasuryManager: zero amount");
        
        bytes memory data = abi.encode(_token, _to, _amount);
        bytes32 opHash = generateOperationHash("emergencyWithdraw", address(this), data);
        _scheduleOperation(opHash, OP_EMERGENCY_WITHDRAW);
    }
    
    /**
     * @dev Execute scheduled emergency withdrawal
     */
    function executeEmergencyWithdraw(
        address _token,
        address _to,
        uint256 _amount
    ) external onlyRole(DEFAULT_ADMIN_ROLE) nonReentrant {
        bytes memory data = abi.encode(_token, _to, _amount);
        bytes32 opHash = generateOperationHash("emergencyWithdraw", address(this), data);
        _executeOperation(opHash);
        
        _transfer(_token, _to, _amount);
        emit FundsWithdrawn(_token, _amount, _to);
    }
    
    /**
     * @dev Emergency withdraw stuck tokens (admin only)
     * @param _token Token to withdraw (address(0) for ETH)
     * @param _to Recipient address
     * @param _amount Amount to withdraw
     */
    function emergencyWithdraw(
        address _token,
        address _to,
        uint256 _amount
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(_to != address(0), "TreasuryManager: zero recipient");
        _transfer(_token, _to, _amount);
        emit FundsWithdrawn(_token, _amount, _to);
    }
    
    // ============ Internal Functions ============
    
    /**
     * @dev Internal transfer function with gas limit for ETH transfers
     * Uses 2300 gas stipend to prevent reentrancy attacks via gas griefing
     */
    function _transfer(address _token, address _to, uint256 _amount) internal {
        require(_to != address(0), "TreasuryManager: zero recipient");
        require(_amount > 0, "TreasuryManager: zero amount");
        
        if (_token == address(0)) {
            // Use send with 2300 gas limit to prevent reentrancy
            // For contracts that need more gas, they should use ERC20
            (bool success, ) = _to.call{value: _amount, gas: 2300}("");
            if (!success) {
                // Fallback: try with more gas for contracts
                (success, ) = _to.call{value: _amount}("");
                require(success, "TreasuryManager: ETH transfer failed");
            }
        } else {
            IERC20(_token).safeTransfer(_to, _amount);
        }
    }
    
    /**
     * @dev Update an existing budget (only before it starts or if admin)
     * @param _budgetId Budget to update
     * @param _newAmount New total allocation
     * @param _newEndTime New end time
     */
    function updateBudget(
        uint256 _budgetId,
        uint256 _newAmount,
        uint256 _newEndTime
    ) external onlyRole(TREASURY_ADMIN) validBudget(_budgetId) {
        Budget storage budget = budgets[_budgetId];
        require(budget.isActive, "TreasuryManager: budget not active");
        require(_newAmount >= budget.spent, "TreasuryManager: amount below spent");
        require(_newEndTime > budget.startTime, "TreasuryManager: invalid end time");
        
        // Can only update if budget hasn't started or caller is admin
        if (block.timestamp >= budget.startTime) {
            require(
                hasRole(DEFAULT_ADMIN_ROLE, msg.sender),
                "TreasuryManager: budget already started"
            );
        }
        
        budget.totalAllocation = _newAmount;
        budget.endTime = _newEndTime;
        
        emit BudgetUpdated(_budgetId, _newAmount, _newEndTime);
    }
    
    /**
     * @dev Delete a budget that has never been used (admin only)
     * @param _budgetId Budget to delete
     */
    function deleteBudget(uint256 _budgetId) 
        external 
        onlyRole(DEFAULT_ADMIN_ROLE) 
        validBudget(_budgetId) 
    {
        Budget storage budget = budgets[_budgetId];
        require(budget.spent == 0, "TreasuryManager: budget already spent");
        require(
            block.timestamp < budget.startTime || !budget.isActive,
            "TreasuryManager: budget active"
        );
        
        delete budgets[_budgetId];
        emit BudgetDeleted(_budgetId);
    }
    
    // ============ View Functions ============
    
    /**
     * @dev Get complete budget details
     */
    function getBudget(uint256 _budgetId) 
        external 
        view 
        returns (Budget memory) 
    {
        return budgets[_budgetId];
    }
    
    /**
     * @dev Get complete stream details
     */
    function getStream(uint256 _streamId) 
        external 
        view 
        returns (StreamingPayment memory) 
    {
        return streams[_streamId];
    }
    
    /**
     * @dev Check if budget is active and within time bounds
     */
    function isBudgetActive(uint256 _budgetId) external view returns (bool) {
        Budget storage budget = budgets[_budgetId];
        if (!budget.isActive) return false;
        if (block.timestamp < budget.startTime) return false;
        if (block.timestamp > budget.endTime) return false;
        return true;
    }
    
    /**
     * @dev Get total budget allocation across all budgets
     */
    function getTotalBudgetAllocation() external view returns (uint256) {
        uint256 total = 0;
        for (uint i = 1; i < nextBudgetId; i++) {
            if (budgets[i].isActive) {
                total += budgets[i].totalAllocation;
            }
        }
        return total;
    }
    
    /**
     * @dev Get total amount in active streams
     */
    function getTotalInStreams() external view returns (uint256) {
        uint256 total = 0;
        for (uint i = 1; i < nextStreamId; i++) {
            if (streams[i].isActive) {
                total += streams[i].totalAmount - streams[i].releasedAmount;
            }
        }
        return total;
    }
    
    // ============ Receive ============
    
    receive() external payable {
        treasuryBalance[address(0)] += msg.value;
        emit FundsDeposited(address(0), msg.value, msg.sender);
    }
}
