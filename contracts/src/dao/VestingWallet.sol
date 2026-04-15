// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title VestingWallet
 * @dev Production-ready token vesting contract with cliff, linear vesting, and revocability
 * 
 * Features:
 * - Configurable cliff period
 * - Linear vesting after cliff
 * - Revocable by owner (for terminated employees)
 * - Supports multiple beneficiaries with separate schedules
 * - Emergency release for governance
 */
contract VestingWallet is Ownable, ReentrancyGuard {
    using SafeERC20 for IERC20;
    
    struct VestingSchedule {
        address beneficiary;
        uint256 totalAmount;
        uint256 startTime;
        uint256 cliffDuration;
        uint256 vestingDuration;
        uint256 releasedAmount;
        bool revocable;
        bool revoked;
    }
    
    // Token being vested
    IERC20 public token;
    
    // Schedule ID counter
    uint256 public nextScheduleId;
    
    // All vesting schedules
    mapping(uint256 => VestingSchedule) public schedules;
    mapping(address => uint256[]) public beneficiarySchedules;
    
    // Total vested tokens per beneficiary
    mapping(address => uint256) public totalVested;
    mapping(address => uint256) public totalReleased;
    
    // Events
    event ScheduleCreated(
        uint256 indexed scheduleId,
        address indexed beneficiary,
        uint256 amount,
        uint256 startTime,
        uint256 cliffDuration,
        uint256 vestingDuration
    );
    event TokensReleased(
        uint256 indexed scheduleId,
        address indexed beneficiary,
        uint256 amount
    );
    event ScheduleRevoked(
        uint256 indexed scheduleId,
        address indexed beneficiary,
        uint256 unreleasedAmount
    );
    event TokensRecovered(address indexed token, uint256 amount);
    
    modifier onlyBeneficiary(uint256 scheduleId) {
        require(
            msg.sender == schedules[scheduleId].beneficiary,
            "VestingWallet: not beneficiary"
        );
        _;
    }
    
    modifier validSchedule(uint256 scheduleId) {
        require(
            scheduleId < nextScheduleId,
            "VestingWallet: invalid schedule"
        );
        _;
    }
    
    constructor(address _token) {
        require(_token != address(0), "VestingWallet: zero token");
        token = IERC20(_token);
    }
    
    /**
     * @dev Create a new vesting schedule
     * @param beneficiary Address receiving vested tokens
     * @param amount Total tokens to vest
     * @param startTime When vesting starts
     * @param cliffDuration Cliff period (no tokens released)
     * @param vestingDuration Total vesting duration (including cliff)
     * @param revocable Whether owner can revoke
     * @return scheduleId ID of the created schedule
     */
    function createVestingSchedule(
        address beneficiary,
        uint256 amount,
        uint256 startTime,
        uint256 cliffDuration,
        uint256 vestingDuration,
        bool revocable
    ) external onlyOwner returns (uint256 scheduleId) {
        require(beneficiary != address(0), "VestingWallet: zero beneficiary");
        require(amount > 0, "VestingWallet: zero amount");
        require(startTime >= block.timestamp, "VestingWallet: start in past");
        require(cliffDuration <= vestingDuration, "VestingWallet: cliff > duration");
        require(vestingDuration > 0, "VestingWallet: zero duration");
        
        // Ensure we have enough tokens
        require(
            token.balanceOf(address(this)) >= totalVestedTokens() + amount,
            "VestingWallet: insufficient balance"
        );
        
        scheduleId = nextScheduleId++;
        
        schedules[scheduleId] = VestingSchedule({
            beneficiary: beneficiary,
            totalAmount: amount,
            startTime: startTime,
            cliffDuration: cliffDuration,
            vestingDuration: vestingDuration,
            releasedAmount: 0,
            revocable: revocable,
            revoked: false
        });
        
        beneficiarySchedules[beneficiary].push(scheduleId);
        totalVested[beneficiary] += amount;
        
        emit ScheduleCreated(
            scheduleId,
            beneficiary,
            amount,
            startTime,
            cliffDuration,
            vestingDuration
        );
        
        return scheduleId;
    }
    
    /**
     * @dev Release vested tokens for a schedule
     * @param scheduleId ID of the vesting schedule
     */
    function release(uint256 scheduleId) 
        external 
        nonReentrant 
        validSchedule(scheduleId) 
        onlyBeneficiary(scheduleId) 
    {
        VestingSchedule storage schedule = schedules[scheduleId];
        require(!schedule.revoked, "VestingWallet: schedule revoked");
        
        uint256 releasable = _releasableAmount(schedule);
        require(releasable > 0, "VestingWallet: no tokens to release");
        
        schedule.releasedAmount += releasable;
        totalReleased[schedule.beneficiary] += releasable;
        
        token.safeTransfer(schedule.beneficiary, releasable);
        
        emit TokensReleased(scheduleId, schedule.beneficiary, releasable);
    }
    
    /**
     * @dev Release all available tokens across all schedules for sender
     */
    function releaseAll() external nonReentrant {
        uint256[] memory scheduleIds = beneficiarySchedules[msg.sender];
        uint256 totalReleasableAmount = 0;
        
        for (uint i = 0; i < scheduleIds.length; i++) {
            VestingSchedule storage schedule = schedules[scheduleIds[i]];
            if (!schedule.revoked && schedule.beneficiary == msg.sender) {
                uint256 releasable = _releasableAmount(schedule);
                if (releasable > 0) {
                    schedule.releasedAmount += releasable;
                    totalReleasableAmount += releasable;
                    emit TokensReleased(scheduleIds[i], msg.sender, releasable);
                }
            }
        }
        
        require(totalReleasableAmount > 0, "VestingWallet: no tokens to release");
        
        totalReleased[msg.sender] += totalReleasableAmount;
        token.safeTransfer(msg.sender, totalReleasableAmount);
    }
    
    /**
     * @dev Revoke a vesting schedule (only owner, only if revocable)
     * @param scheduleId ID of the schedule to revoke
     */
    function revoke(uint256 scheduleId) 
        external 
        onlyOwner 
        validSchedule(scheduleId) 
    {
        VestingSchedule storage schedule = schedules[scheduleId];
        require(schedule.revocable, "VestingWallet: not revocable");
        require(!schedule.revoked, "VestingWallet: already revoked");
        
        uint256 releasable = _releasableAmount(schedule);
        uint256 unreleased = schedule.totalAmount - schedule.releasedAmount - releasable;
        
        schedule.revoked = true;
        
        // Release any immediately vested tokens
        if (releasable > 0) {
            schedule.releasedAmount += releasable;
            totalReleased[schedule.beneficiary] += releasable;
            token.safeTransfer(schedule.beneficiary, releasable);
        }
        
        emit ScheduleRevoked(scheduleId, schedule.beneficiary, unreleased);
    }
    
    /**
     * @dev Calculate releasable amount for a schedule
     */
    function _releasableAmount(VestingSchedule storage schedule) 
        internal 
        view 
        returns (uint256) 
    {
        return _vestedAmount(schedule) - schedule.releasedAmount;
    }
    
    /**
     * @dev Calculate total vested amount for a schedule
     */
    function _vestedAmount(VestingSchedule storage schedule) 
        internal 
        view 
        returns (uint256) 
    {
        if (schedule.revoked) {
            return schedule.releasedAmount;
        }
        
        if (block.timestamp < schedule.startTime + schedule.cliffDuration) {
            return 0;
        }
        
        if (block.timestamp >= schedule.startTime + schedule.vestingDuration) {
            return schedule.totalAmount;
        }
        
        uint256 timeAfterCliff = block.timestamp - schedule.startTime - schedule.cliffDuration;
        uint256 vestingPeriod = schedule.vestingDuration - schedule.cliffDuration;
        
        return (schedule.totalAmount * timeAfterCliff) / vestingPeriod;
    }
    
    /**
     * @dev Deposit tokens into vesting contract
     * @param amount Amount to deposit
     */
    function deposit(uint256 amount) external {
        token.safeTransferFrom(msg.sender, address(this), amount);
    }
    
    /**
     * @dev Recover accidentally sent ERC20 tokens
     * @param tokenToRecover Token address to recover
     * @param amount Amount to recover
     */
    function recoverERC20(address tokenToRecover, uint256 amount) external onlyOwner {
        require(tokenToRecover != address(token), "VestingWallet: cannot recover vesting token");
        IERC20(tokenToRecover).safeTransfer(owner(), amount);
        emit TokensRecovered(tokenToRecover, amount);
    }
    
    /**
     * @dev Recover accidentally sent ETH
     */
    function recoverETH() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "VestingWallet: no ETH to recover");
        (bool success, ) = owner().call{value: balance}("");
        require(success, "VestingWallet: ETH recovery failed");
    }
    
    // ============ View Functions ============
    
    /**
     * @dev Get vesting schedule details
     */
    function getSchedule(uint256 scheduleId) 
        external 
        view 
        returns (VestingSchedule memory) 
    {
        return schedules[scheduleId];
    }
    
    /**
     * @dev Get all schedule IDs for a beneficiary
     */
    function getBeneficiarySchedules(address beneficiary) 
        external 
        view 
        returns (uint256[] memory) 
    {
        return beneficiarySchedules[beneficiary];
    }
    
    /**
     * @dev Calculate releasable amount for a schedule (view)
     */
    function releasableAmount(uint256 scheduleId) 
        external 
        view 
        validSchedule(scheduleId) 
        returns (uint256) 
    {
        return _releasableAmount(schedules[scheduleId]);
    }
    
    /**
     * @dev Calculate vested amount for a schedule (view)
     */
    function vestedAmount(uint256 scheduleId) 
        external 
        view 
        validSchedule(scheduleId) 
        returns (uint256) 
    {
        return _vestedAmount(schedules[scheduleId]);
    }
    
    /**
     * @dev Get total releasable for a beneficiary across all schedules
     */
    function totalReleasable(address beneficiary) external view returns (uint256) {
        uint256[] memory scheduleIds = beneficiarySchedules[beneficiary];
        uint256 total = 0;
        
        for (uint i = 0; i < scheduleIds.length; i++) {
            total += _releasableAmount(schedules[scheduleIds[i]]);
        }
        
        return total;
    }
    
    /**
     * @dev Calculate total tokens needed for all vesting schedules
     */
    function totalVestedTokens() public view returns (uint256) {
        uint256 total = 0;
        for (uint i = 0; i < nextScheduleId; i++) {
            if (!schedules[i].revoked) {
                total += schedules[i].totalAmount - schedules[i].releasedAmount;
            }
        }
        return total;
    }
    
    /**
     * @dev Check if cliff has passed for a schedule
     */
    function isCliffReached(uint256 scheduleId) 
        external 
        view 
        validSchedule(scheduleId) 
        returns (bool) 
    {
        VestingSchedule storage schedule = schedules[scheduleId];
        return block.timestamp >= schedule.startTime + schedule.cliffDuration;
    }
    
    /**
     * @dev Check if vesting is complete for a schedule
     */
    function isVestingComplete(uint256 scheduleId) 
        external 
        view 
        validSchedule(scheduleId) 
        returns (bool) 
    {
        VestingSchedule storage schedule = schedules[scheduleId];
        return block.timestamp >= schedule.startTime + schedule.vestingDuration;
    }
    
    receive() external payable {
        revert("VestingWallet: ETH not accepted");
    }
}
