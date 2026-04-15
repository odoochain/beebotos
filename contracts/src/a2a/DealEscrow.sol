// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/**
 * @title DealEscrow
 * @notice Production-ready escrow service for A2A deals with access control
 * 
 * Security Features:
 * - Only A2ACommerce contract can create and manage escrows
 * - Owner can refund in emergency situations
 * - Upgradeable via UUPS proxy pattern
 * - Reentrancy protection for all fund transfers
 */
contract DealEscrow is 
    ReentrancyGuardUpgradeable, 
    OwnableUpgradeable,
    UUPSUpgradeable 
{
    using SafeERC20 for IERC20;
    
    struct Escrow {
        bytes32 escrowId;
        bytes32 dealId;           // Reference to parent deal
        address buyer;
        address seller;
        address token;
        uint256 amount;
        uint256 fee;              // Platform fee
        uint256 createdAt;
        uint256 releasedAt;       // When funds were released
        bool isReleased;
        bool isRefunded;
    }
    
    // Access control
    address public a2aCommerce;
    address public feeRecipient;
    
    // State
    mapping(bytes32 => Escrow) public escrows;
    mapping(bytes32 => bool) public processedDeals;  // Prevent duplicate escrows
    
    // Platform fee (in basis points, e.g., 250 = 2.5%)
    uint256 public platformFeeBps;
    uint256 public constant MAX_PLATFORM_FEE = 500; // Max 5%
    uint256 public constant BASIS_POINTS = 10000;
    
    // Emergency refund window
    uint256 public constant REFUND_WINDOW = 30 days;
    
    // Events
    event EscrowCreated(
        bytes32 indexed escrowId, 
        bytes32 indexed dealId,
        address indexed buyer, 
        address seller,
        address token,
        uint256 amount,
        uint256 fee
    );
    event EscrowReleased(
        bytes32 indexed escrowId, 
        address indexed seller, 
        uint256 sellerAmount,
        uint256 feeAmount
    );
    event EscrowRefunded(
        bytes32 indexed escrowId, 
        address indexed buyer, 
        uint256 amount
    );
    event A2ACommerceSet(address indexed oldAddress, address indexed newAddress);
    event FeeRecipientSet(address indexed oldRecipient, address indexed newRecipient);
    event PlatformFeeUpdated(uint256 oldFee, uint256 newFee);
    
    // Modifiers
    modifier onlyA2ACommerce() {
        require(
            msg.sender == a2aCommerce,
            "DealEscrow: caller is not A2ACommerce"
        );
        _;
    }
    
    modifier onlySellerOrAdmin(bytes32 escrowId) {
        require(
            msg.sender == escrows[escrowId].seller || 
            msg.sender == owner(),
            "DealEscrow: not authorized"
        );
        _;
    }
    
    /**
     * @dev Initializer for proxy deployment
     * @param _a2aCommerce Address of A2ACommerce contract
     * @param _feeRecipient Address to receive platform fees
     * @param _platformFeeBps Platform fee in basis points
     */
    function initialize(
        address _a2aCommerce,
        address _feeRecipient,
        uint256 _platformFeeBps
    ) public initializer {
        require(_a2aCommerce != address(0), "DealEscrow: zero A2ACommerce");
        require(_feeRecipient != address(0), "DealEscrow: zero fee recipient");
        require(_platformFeeBps <= MAX_PLATFORM_FEE, "DealEscrow: fee too high");
        
        __ReentrancyGuard_init();
        __Ownable_init();
        __UUPSUpgradeable_init();
        
        a2aCommerce = _a2aCommerce;
        feeRecipient = _feeRecipient;
        platformFeeBps = _platformFeeBps;
    }
    
    /**
     * @dev Create a new escrow (only callable by A2ACommerce)
     * @param dealId The parent deal ID
     * @param buyer Buyer address
     * @param seller Seller address
     * @param token Token address (address(0) for ETH)
     * @param amount Total escrow amount
     * @return escrowId Unique escrow identifier
     */
    function createEscrow(
        bytes32 dealId,
        address buyer,
        address seller,
        address token,
        uint256 amount
    ) 
        external 
        payable 
        onlyA2ACommerce 
        returns (bytes32 escrowId) 
    {
        require(!processedDeals[dealId], "DealEscrow: deal already processed");
        require(buyer != address(0), "DealEscrow: zero buyer");
        require(seller != address(0), "DealEscrow: zero seller");
        require(buyer != seller, "DealEscrow: buyer is seller");
        require(amount > 0, "DealEscrow: zero amount");
        
        // Calculate fee
        uint256 fee = (amount * platformFeeBps) / BASIS_POINTS;
        uint256 escrowAmount = amount - fee;
        
        // Verify funds received
        if (token == address(0)) {
            require(msg.value == amount, "DealEscrow: invalid ETH amount");
        } else {
            require(msg.value == 0, "DealEscrow: ETH not accepted for ERC20");
            // Transfer tokens from buyer (via A2ACommerce allowance)
            IERC20(token).safeTransferFrom(buyer, address(this), amount);
        }
        
        // Generate unique escrow ID
        escrowId = keccak256(abi.encodePacked(
            dealId,
            buyer,
            seller,
            token,
            amount,
            block.timestamp
        ));
        
        require(escrows[escrowId].escrowId == bytes32(0), "DealEscrow: escrow exists");
        
        escrows[escrowId] = Escrow({
            escrowId: escrowId,
            dealId: dealId,
            buyer: buyer,
            seller: seller,
            token: token,
            amount: escrowAmount,
            fee: fee,
            createdAt: block.timestamp,
            releasedAt: 0,
            isReleased: false,
            isRefunded: false
        });
        
        processedDeals[dealId] = true;
        
        emit EscrowCreated(
            escrowId,
            dealId,
            buyer,
            seller,
            token,
            escrowAmount,
            fee
        );
        
        return escrowId;
    }
    
    /**
     * @dev Release escrow funds to seller (only callable by A2ACommerce)
     * @param escrowId Escrow identifier
     * @param sellerAmount Amount to send to seller (after fee deduction)
     */
    function releaseEscrow(
        bytes32 escrowId,
        uint256 sellerAmount
    ) 
        external 
        nonReentrant 
        onlyA2ACommerce 
    {
        Escrow storage escrow = escrows[escrowId];
        require(escrow.escrowId != bytes32(0), "DealEscrow: escrow not found");
        require(!escrow.isReleased, "DealEscrow: already released");
        require(!escrow.isRefunded, "DealEscrow: already refunded");
        require(
            sellerAmount + escrow.fee == escrow.amount || 
            sellerAmount <= escrow.amount,
            "DealEscrow: invalid amounts"
        );
        
        escrow.isReleased = true;
        escrow.releasedAt = block.timestamp;
        
        uint256 actualSellerAmount = sellerAmount > 0 ? sellerAmount : escrow.amount - escrow.fee;
        
        // Transfer to seller
        if (escrow.token == address(0)) {
            (bool success, ) = escrow.seller.call{value: actualSellerAmount}("");
            require(success, "DealEscrow: ETH transfer to seller failed");
            
            // Transfer fee to fee recipient
            if (escrow.fee > 0) {
                (bool feeSuccess, ) = feeRecipient.call{value: escrow.fee}("");
                require(feeSuccess, "DealEscrow: ETH fee transfer failed");
            }
        } else {
            IERC20(escrow.token).safeTransfer(escrow.seller, actualSellerAmount);
            
            if (escrow.fee > 0) {
                IERC20(escrow.token).safeTransfer(feeRecipient, escrow.fee);
            }
        }
        
        emit EscrowReleased(escrowId, escrow.seller, actualSellerAmount, escrow.fee);
    }
    
    /**
     * @dev Refund escrow to buyer
     * @param escrowId Escrow identifier
     * 
     * Can be called by:
     * - A2ACommerce (normal refund flow)
     * - Seller (agrees to cancel)
     * - Owner (emergency after refund window)
     */
    function refundEscrow(bytes32 escrowId) external nonReentrant {
        Escrow storage escrow = escrows[escrowId];
        require(escrow.escrowId != bytes32(0), "DealEscrow: escrow not found");
        require(!escrow.isReleased, "DealEscrow: already released");
        require(!escrow.isRefunded, "DealEscrow: already refunded");
        
        bool isAuthorized = false;
        
        if (msg.sender == a2aCommerce) {
            // A2ACommerce can refund anytime
            isAuthorized = true;
        } else if (msg.sender == escrow.seller) {
            // Seller can agree to refund
            isAuthorized = true;
        } else if (msg.sender == owner()) {
            // Owner can refund after emergency window
            require(
                block.timestamp > escrow.createdAt + REFUND_WINDOW,
                "DealEscrow: emergency window not open"
            );
            isAuthorized = true;
        }
        
        require(isAuthorized, "DealEscrow: not authorized");
        
        escrow.isRefunded = true;
        
        // Return full amount to buyer (including fee, as platform didn't deliver)
        uint256 refundAmount = escrow.amount + escrow.fee;
        
        if (escrow.token == address(0)) {
            (bool success, ) = escrow.buyer.call{value: refundAmount}("");
            require(success, "DealEscrow: ETH refund failed");
        } else {
            IERC20(escrow.token).safeTransfer(escrow.buyer, refundAmount);
        }
        
        emit EscrowRefunded(escrowId, escrow.buyer, refundAmount);
    }
    
    /**
     * @dev Emergency refund by admin (ignores time window)
     * @param escrowId Escrow identifier
     */
    function emergencyRefund(bytes32 escrowId) external onlyOwner nonReentrant {
        Escrow storage escrow = escrows[escrowId];
        require(escrow.escrowId != bytes32(0), "DealEscrow: escrow not found");
        require(!escrow.isReleased, "DealEscrow: already released");
        require(!escrow.isRefunded, "DealEscrow: already refunded");
        
        escrow.isRefunded = true;
        
        uint256 refundAmount = escrow.amount + escrow.fee;
        
        if (escrow.token == address(0)) {
            (bool success, ) = escrow.buyer.call{value: refundAmount}("");
            require(success, "DealEscrow: emergency refund failed");
        } else {
            IERC20(escrow.token).safeTransfer(escrow.buyer, refundAmount);
        }
        
        emit EscrowRefunded(escrowId, escrow.buyer, refundAmount);
    }
    
    // ============ Timelock Configuration ============
    
    uint256 public constant MIN_TIMELOCK_DELAY = 1 days;
    uint256 public timelockDelay;
    
    struct TimelockOperation {
        uint256 scheduledTime;
        uint256 executionTime;
        bool executed;
        bool cancelled;
    }
    
    mapping(bytes32 => TimelockOperation) public timelockOperations;
    
    bytes32 public constant OP_SET_A2A_COMMERCE = keccak256("OP_SET_A2A_COMMERCE");
    bytes32 public constant OP_SET_FEE_RECIPIENT = keccak256("OP_SET_FEE_RECIPIENT");
    bytes32 public constant OP_SET_PLATFORM_FEE = keccak256("OP_SET_PLATFORM_FEE");
    
    event TimelockOperationScheduled(bytes32 indexed opHash, uint256 executionTime);
    event TimelockOperationExecuted(bytes32 indexed opHash);
    event TimelockOperationCancelled(bytes32 indexed opHash);
    
    /**
     * @dev Initialize timelock
     */
    function initializeTimelock(uint256 _delay) external onlyOwner {
        require(timelockDelay == 0, "DealEscrow: timelock already initialized");
        require(_delay >= MIN_TIMELOCK_DELAY, "DealEscrow: delay too short");
        timelockDelay = _delay;
    }
    
    // ============ Admin Functions with Timelock ============
    
    /**
     * @dev Schedule A2ACommerce address update
     */
    function scheduleSetA2ACommerce(address newA2ACommerce) external onlyOwner {
        require(timelockDelay > 0, "DealEscrow: timelock not initialized");
        require(newA2ACommerce != address(0), "DealEscrow: zero address");
        
        bytes32 opHash = keccak256(abi.encode(OP_SET_A2A_COMMERCE, newA2ACommerce, block.timestamp));
        require(timelockOperations[opHash].scheduledTime == 0, "DealEscrow: already scheduled");
        
        uint256 execTime = block.timestamp + timelockDelay;
        timelockOperations[opHash] = TimelockOperation({
            scheduledTime: block.timestamp,
            executionTime: execTime,
            executed: false,
            cancelled: false
        });
        
        emit TimelockOperationScheduled(opHash, execTime);
    }
    
    /**
     * @dev Execute scheduled A2ACommerce update
     */
    function executeSetA2ACommerce(address newA2ACommerce, uint256 scheduledTime) external onlyOwner {
        bytes32 opHash = keccak256(abi.encode(OP_SET_A2A_COMMERCE, newA2ACommerce, scheduledTime));
        
        TimelockOperation storage op = timelockOperations[opHash];
        require(op.scheduledTime != 0, "DealEscrow: not scheduled");
        require(!op.executed, "DealEscrow: already executed");
        require(!op.cancelled, "DealEscrow: cancelled");
        require(block.timestamp >= op.executionTime, "DealEscrow: delay not met");
        
        op.executed = true;
        emit A2ACommerceSet(a2aCommerce, newA2ACommerce);
        a2aCommerce = newA2ACommerce;
        emit TimelockOperationExecuted(opHash);
    }
    
    /**
     * @dev Schedule fee recipient update
     */
    function scheduleSetFeeRecipient(address newFeeRecipient) external onlyOwner {
        require(timelockDelay > 0, "DealEscrow: timelock not initialized");
        require(newFeeRecipient != address(0), "DealEscrow: zero address");
        
        bytes32 opHash = keccak256(abi.encode(OP_SET_FEE_RECIPIENT, newFeeRecipient, block.timestamp));
        require(timelockOperations[opHash].scheduledTime == 0, "DealEscrow: already scheduled");
        
        uint256 execTime = block.timestamp + timelockDelay;
        timelockOperations[opHash] = TimelockOperation({
            scheduledTime: block.timestamp,
            executionTime: execTime,
            executed: false,
            cancelled: false
        });
        
        emit TimelockOperationScheduled(opHash, execTime);
    }
    
    /**
     * @dev Execute scheduled fee recipient update
     */
    function executeSetFeeRecipient(address newFeeRecipient, uint256 scheduledTime) external onlyOwner {
        bytes32 opHash = keccak256(abi.encode(OP_SET_FEE_RECIPIENT, newFeeRecipient, scheduledTime));
        
        TimelockOperation storage op = timelockOperations[opHash];
        require(op.scheduledTime != 0, "DealEscrow: not scheduled");
        require(!op.executed, "DealEscrow: already executed");
        require(!op.cancelled, "DealEscrow: cancelled");
        require(block.timestamp >= op.executionTime, "DealEscrow: delay not met");
        
        op.executed = true;
        emit FeeRecipientSet(feeRecipient, newFeeRecipient);
        feeRecipient = newFeeRecipient;
        emit TimelockOperationExecuted(opHash);
    }
    
    /**
     * @dev Schedule platform fee update
     */
    function scheduleSetPlatformFee(uint256 newFeeBps) external onlyOwner {
        require(timelockDelay > 0, "DealEscrow: timelock not initialized");
        require(newFeeBps <= MAX_PLATFORM_FEE, "DealEscrow: fee too high");
        
        bytes32 opHash = keccak256(abi.encode(OP_SET_PLATFORM_FEE, newFeeBps, block.timestamp));
        require(timelockOperations[opHash].scheduledTime == 0, "DealEscrow: already scheduled");
        
        uint256 execTime = block.timestamp + timelockDelay;
        timelockOperations[opHash] = TimelockOperation({
            scheduledTime: block.timestamp,
            executionTime: execTime,
            executed: false,
            cancelled: false
        });
        
        emit TimelockOperationScheduled(opHash, execTime);
    }
    
    /**
     * @dev Execute scheduled platform fee update
     */
    function executeSetPlatformFee(uint256 newFeeBps, uint256 scheduledTime) external onlyOwner {
        bytes32 opHash = keccak256(abi.encode(OP_SET_PLATFORM_FEE, newFeeBps, scheduledTime));
        
        TimelockOperation storage op = timelockOperations[opHash];
        require(op.scheduledTime != 0, "DealEscrow: not scheduled");
        require(!op.executed, "DealEscrow: already executed");
        require(!op.cancelled, "DealEscrow: cancelled");
        require(block.timestamp >= op.executionTime, "DealEscrow: delay not met");
        
        op.executed = true;
        emit PlatformFeeUpdated(platformFeeBps, newFeeBps);
        platformFeeBps = newFeeBps;
        emit TimelockOperationExecuted(opHash);
    }
    
    /**
     * @dev Cancel scheduled operation
     */
    function cancelTimelockOperation(bytes32 opHash) external onlyOwner {
        TimelockOperation storage op = timelockOperations[opHash];
        require(op.scheduledTime != 0, "DealEscrow: not scheduled");
        require(!op.executed, "DealEscrow: already executed");
        require(!op.cancelled, "DealEscrow: already cancelled");
        
        op.cancelled = true;
        emit TimelockOperationCancelled(opHash);
    }
    
    /**
     * @dev Check if operation is ready
     */
    function isTimelockOperationReady(bytes32 opHash) external view returns (bool) {
        TimelockOperation storage op = timelockOperations[opHash];
        return op.scheduledTime != 0 && !op.executed && !op.cancelled && block.timestamp >= op.executionTime;
    }
    
    // ============ Legacy Direct Functions ============
    
    /**
     * @dev Update A2ACommerce address (direct - use timelock for production)
     */
    function setA2ACommerce(address newA2ACommerce) external onlyOwner {
        require(newA2ACommerce != address(0), "DealEscrow: zero address");
        emit A2ACommerceSet(a2aCommerce, newA2ACommerce);
        a2aCommerce = newA2ACommerce;
    }
    
    /**
     * @dev Update fee recipient (direct)
     */
    function setFeeRecipient(address newFeeRecipient) external onlyOwner {
        require(newFeeRecipient != address(0), "DealEscrow: zero address");
        emit FeeRecipientSet(feeRecipient, newFeeRecipient);
        feeRecipient = newFeeRecipient;
    }
    
    /**
     * @dev Update platform fee (direct - small changes only)
     */
    function setPlatformFee(uint256 newFeeBps) external onlyOwner {
        require(newFeeBps <= MAX_PLATFORM_FEE, "DealEscrow: fee too high");
        // If timelock active, prevent large increases without it
        if (timelockDelay > 0 && newFeeBps > platformFeeBps + 50) {
            revert("DealEscrow: use scheduleSetPlatformFee for large increases");
        }
        emit PlatformFeeUpdated(platformFeeBps, newFeeBps);
        platformFeeBps = newFeeBps;
    }
    
    // ============ View Functions ============
    
    /**
     * @dev Get escrow details
     */
    function getEscrow(bytes32 escrowId) external view returns (Escrow memory) {
        return escrows[escrowId];
    }
    
    /**
     * @dev Check if deal has been processed
     */
    function isDealProcessed(bytes32 dealId) external view returns (bool) {
        return processedDeals[dealId];
    }
    
    /**
     * @dev Check if refund window is open for emergency refund
     */
    function isRefundWindowOpen(bytes32 escrowId) external view returns (bool) {
        Escrow storage escrow = escrows[escrowId];
        if (escrow.escrowId == bytes32(0)) return false;
        return block.timestamp > escrow.createdAt + REFUND_WINDOW;
    }
    
    // ============ Upgrade Authorization ============
    
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
    
    // ============ Receive ============
    
    receive() external payable {
        revert("DealEscrow: direct deposits not allowed");
    }
}
