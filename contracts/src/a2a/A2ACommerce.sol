// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "../interfaces/IA2ACommerce.sol";
import "./DealEscrow.sol";

/**
 * @title A2ACommerce
 * @notice Production-ready A2A commerce with pause and improved ID generation
 * 
 * Timelock Integration:
 * - Critical operations (setPlatformFee, setFeeRecipient) support timelock
 * - Use schedule + execute pattern for sensitive changes
 */
contract A2ACommerce is IA2ACommerce, ReentrancyGuardUpgradeable, OwnableUpgradeable, PausableUpgradeable, UUPSUpgradeable {
    using SafeERC20 for IERC20;
    
    DealEscrow public escrow;
    
    mapping(bytes32 => ServiceListing) public services;
    mapping(bytes32 => Deal) public deals;
    mapping(address => bytes32[]) public providerServices;
    mapping(address => bytes32[]) public buyerDeals;
    mapping(bytes32 => bool) public usedServiceIds;  // Prevent ID collisions
    mapping(bytes32 => bool) public usedDealIds;
    
    address private immutable __self;
    
    uint256 public platformFeeBasisPoints;
    address public feeRecipient;
    uint256 public dealCounter;  // Sequential counter for deal ID entropy
    uint256 public serviceCounter;  // Sequential counter for service ID entropy
    
    constructor() {
        __self = address(this);
    }
    
    // Events are defined in IA2ACommerce interface
    
    modifier onlyServiceProvider(bytes32 serviceId) {
        require(services[serviceId].provider == msg.sender, "A2ACommerce: not provider");
        _;
    }
    

    
    function initialize(address escrowAddress) public initializer {
        require(escrowAddress != address(0), "A2ACommerce: zero escrow");
        
        __Ownable_init();
        __ReentrancyGuard_init();
        __Pausable_init();
        escrow = DealEscrow(payable(escrowAddress));
        platformFeeBasisPoints = 250; // 2.5%
        feeRecipient = msg.sender;
        dealCounter = 0;
        serviceCounter = 0;
    }
    
    function listService(
        string calldata metadataURI,
        uint256 price,
        address paymentToken
    ) external override whenNotPaused returns (bytes32 serviceId) {
        require(price > 0, "A2ACommerce: price must be > 0");
        require(bytes(metadataURI).length > 0, "A2ACommerce: metadata required");
        
        // Generate service ID with entropy
        serviceCounter++;
        serviceId = keccak256(abi.encodePacked(
            msg.sender,
            metadataURI,
            serviceCounter,
            block.timestamp,
            block.number,
            block.prevrandao  // Post-merge randomness
        ));
        
        require(!usedServiceIds[serviceId], "A2ACommerce: ID collision");
        usedServiceIds[serviceId] = true;
        
        ServiceListing storage service = services[serviceId];
        service.serviceId = serviceId;
        service.provider = msg.sender;
        service.metadataURI = metadataURI;
        service.price = price;
        service.paymentToken = paymentToken;
        service.isActive = true;
        
        providerServices[msg.sender].push(serviceId);
        
        emit ServiceListed(serviceId, msg.sender, price, paymentToken);
    }
    
    function updateService(
        bytes32 serviceId,
        uint256 newPrice,
        bool isActive
    ) external onlyServiceProvider(serviceId) whenNotPaused {
        services[serviceId].price = newPrice;
        services[serviceId].isActive = isActive;
        emit ServiceUpdated(serviceId, newPrice, isActive);
    }
    
    function removeService(bytes32 serviceId) external onlyServiceProvider(serviceId) {
        services[serviceId].isActive = false;
        emit ServiceUpdated(serviceId, 0, false);
    }
    
    function createDeal(
        bytes32 serviceId,
        uint256 expiresAt
    ) external override whenNotPaused returns (bytes32 dealId) {
        ServiceListing memory service = services[serviceId];
        require(service.isActive, "A2ACommerce: service not active");
        require(expiresAt > block.timestamp, "A2ACommerce: invalid expiration");
        require(service.provider != msg.sender, "A2ACommerce: cannot buy own service");
        
        // Generate deal ID with entropy
        dealCounter++;
        dealId = keccak256(abi.encodePacked(
            msg.sender,
            serviceId,
            dealCounter,
            block.timestamp,
            block.number,
            block.prevrandao
        ));
        
        require(!usedDealIds[dealId], "A2ACommerce: deal ID collision");
        usedDealIds[dealId] = true;
        
        Deal storage deal = deals[dealId];
        deal.dealId = dealId;
        deal.buyer = msg.sender;
        deal.seller = service.provider;
        deal.serviceId = serviceId;
        deal.price = service.price;
        deal.paymentToken = service.paymentToken;
        deal.createdAt = block.timestamp;
        deal.expiresAt = expiresAt;
        deal.status = DealStatus.Pending;
        
        buyerDeals[msg.sender].push(dealId);
        
        emit DealCreated(dealId, serviceId, msg.sender);
    }
    
    function fundDeal(bytes32 dealId) external payable override nonReentrant whenNotPaused {
        Deal storage deal = deals[dealId];
        require(deal.status == DealStatus.Pending, "A2ACommerce: invalid status");
        require(block.timestamp < deal.expiresAt, "A2ACommerce: deal expired");
        
        if (deal.paymentToken == address(0)) {
            require(msg.value >= deal.price, "A2ACommerce: insufficient ETH");
            // Refund excess
            if (msg.value > deal.price) {
                (bool success, ) = msg.sender.call{value: msg.value - deal.price}("");
                require(success, "A2ACommerce: refund failed");
            }
        } else {
            IERC20(deal.paymentToken).safeTransferFrom(msg.sender, address(this), deal.price);
        }
        
        // Create escrow
        bytes32 escrowId = escrow.createEscrow{value: deal.paymentToken == address(0) ? deal.price : 0}(
            dealId,
            deal.buyer,
            deal.seller,
            deal.paymentToken,
            deal.price
        );
        
        deal.escrowId = escrowId;
        deal.status = DealStatus.Funded;
        
        emit DealFunded(dealId, escrowId);
    }
    
    function completeDeal(bytes32 dealId) external override nonReentrant whenNotPaused {
        Deal storage deal = deals[dealId];
        require(
            deal.status == DealStatus.InProgress || deal.status == DealStatus.Delivered, 
            "A2ACommerce: invalid status"
        );
        require(deal.buyer == msg.sender, "A2ACommerce: not buyer");
        
        // Calculate fee
        uint256 fee = calculatePlatformFee(deal.price);
        uint256 sellerAmount = deal.price - fee;
        
        escrow.releaseEscrow(deal.escrowId, sellerAmount);
        
        deal.status = DealStatus.Completed;
        services[deal.serviceId].totalSales++;
        
        // Transfer fee to fee recipient
        if (deal.paymentToken == address(0)) {
            (bool success, ) = feeRecipient.call{value: fee}("");
            require(success, "A2ACommerce: fee transfer failed");
        } else {
            IERC20(deal.paymentToken).safeTransfer(feeRecipient, fee);
        }
        
        emit DealCompleted(dealId, deal.buyer, deal.seller);
    }
    
    function cancelDeal(bytes32 dealId) external nonReentrant {
        Deal storage deal = deals[dealId];
        require(deal.status == DealStatus.Pending || deal.status == DealStatus.Funded, 
            "A2ACommerce: cannot cancel");
        require(
            deal.buyer == msg.sender || deal.seller == msg.sender || msg.sender == owner(),
            "A2ACommerce: not authorized"
        );
        
        if (deal.status == DealStatus.Funded) {
            // Refund from escrow
            escrow.refundEscrow(deal.escrowId);
        }
        
        deal.status = DealStatus.Cancelled;
        emit DealCancelled(dealId, msg.sender);
    }
    
    function calculatePlatformFee(uint256 amount) public view returns (uint256) {
        return (amount * platformFeeBasisPoints) / 10000;
    }
    
    function getDeal(bytes32 dealId) external view returns (Deal memory) {
        return deals[dealId];
    }
    
    function getService(bytes32 serviceId) external view returns (ServiceListing memory) {
        return services[serviceId];
    }
    
    function getBuyerDeals(address buyer) external view returns (bytes32[] memory) {
        return buyerDeals[buyer];
    }
    
    function getProviderServices(address provider) external view returns (bytes32[] memory) {
        return providerServices[provider];
    }
    
    // ============ Timelock Configuration ============
    
    // Minimum delay for critical operations: 1 day
    uint256 public constant MIN_TIMELOCK_DELAY = 1 days;
    uint256 public timelockDelay;
    
    struct TimelockOperation {
        bytes32 operationHash;
        uint256 scheduledTime;
        uint256 executionTime;
        bool executed;
        bool cancelled;
    }
    
    mapping(bytes32 => TimelockOperation) public timelockOperations;
    
    event TimelockOperationScheduled(
        bytes32 indexed operationHash,
        string operationType,
        uint256 executionTime
    );
    event TimelockOperationExecuted(bytes32 indexed operationHash);
    event TimelockOperationCancelled(bytes32 indexed operationHash, string reason);
    event TimelockDelayChanged(uint256 oldDelay, uint256 newDelay);
    
    bytes32 public constant OP_SET_PLATFORM_FEE = keccak256("OP_SET_PLATFORM_FEE");
    bytes32 public constant OP_SET_FEE_RECIPIENT = keccak256("OP_SET_FEE_RECIPIENT");
    bytes32 public constant OP_SET_ESCROW = keccak256("OP_SET_ESCROW");
    
    // ============ Timelock Functions ============
    
    /**
     * @dev Initialize timelock settings
     */
    function initializeTimelock(uint256 _delay) external onlyOwner {
        require(timelockDelay == 0, "A2ACommerce: timelock already initialized");
        require(_delay >= MIN_TIMELOCK_DELAY, "A2ACommerce: delay too short");
        timelockDelay = _delay;
    }
    
    /**
     * @dev Schedule platform fee update
     */
    function scheduleSetPlatformFee(uint256 newFeeBps) external onlyOwner {
        require(timelockDelay > 0, "A2ACommerce: timelock not initialized");
        require(newFeeBps <= 1000, "A2ACommerce: max 10%");
        
        bytes32 opHash = keccak256(abi.encode(OP_SET_PLATFORM_FEE, newFeeBps, block.timestamp));
        require(timelockOperations[opHash].scheduledTime == 0, "A2ACommerce: already scheduled");
        
        uint256 executionTime = block.timestamp + timelockDelay;
        timelockOperations[opHash] = TimelockOperation({
            operationHash: opHash,
            scheduledTime: block.timestamp,
            executionTime: executionTime,
            executed: false,
            cancelled: false
        });
        
        emit TimelockOperationScheduled(opHash, "setPlatformFee", executionTime);
    }
    
    /**
     * @dev Execute scheduled platform fee update
     */
    function executeSetPlatformFee(uint256 newFeeBps, uint256 scheduledTime) external onlyOwner {
        bytes32 opHash = keccak256(abi.encode(OP_SET_PLATFORM_FEE, newFeeBps, scheduledTime));
        
        TimelockOperation storage op = timelockOperations[opHash];
        require(op.scheduledTime != 0, "A2ACommerce: not scheduled");
        require(!op.executed, "A2ACommerce: already executed");
        require(!op.cancelled, "A2ACommerce: cancelled");
        require(block.timestamp >= op.executionTime, "A2ACommerce: delay not met");
        
        op.executed = true;
        platformFeeBasisPoints = newFeeBps;
        
        emit TimelockOperationExecuted(opHash);
    }
    
    /**
     * @dev Schedule fee recipient update
     */
    function scheduleSetFeeRecipient(address newRecipient) external onlyOwner {
        require(timelockDelay > 0, "A2ACommerce: timelock not initialized");
        require(newRecipient != address(0), "A2ACommerce: zero address");
        
        bytes32 opHash = keccak256(abi.encode(OP_SET_FEE_RECIPIENT, newRecipient, block.timestamp));
        require(timelockOperations[opHash].scheduledTime == 0, "A2ACommerce: already scheduled");
        
        uint256 executionTime = block.timestamp + timelockDelay;
        timelockOperations[opHash] = TimelockOperation({
            operationHash: opHash,
            scheduledTime: block.timestamp,
            executionTime: executionTime,
            executed: false,
            cancelled: false
        });
        
        emit TimelockOperationScheduled(opHash, "setFeeRecipient", executionTime);
    }
    
    /**
     * @dev Execute scheduled fee recipient update
     */
    function executeSetFeeRecipient(address newRecipient, uint256 scheduledTime) external onlyOwner {
        bytes32 opHash = keccak256(abi.encode(OP_SET_FEE_RECIPIENT, newRecipient, scheduledTime));
        
        TimelockOperation storage op = timelockOperations[opHash];
        require(op.scheduledTime != 0, "A2ACommerce: not scheduled");
        require(!op.executed, "A2ACommerce: already executed");
        require(!op.cancelled, "A2ACommerce: cancelled");
        require(block.timestamp >= op.executionTime, "A2ACommerce: delay not met");
        
        op.executed = true;
        feeRecipient = newRecipient;
        
        emit TimelockOperationExecuted(opHash);
    }
    
    /**
     * @dev Schedule escrow update
     */
    function scheduleSetEscrow(address newEscrow) external onlyOwner {
        require(timelockDelay > 0, "A2ACommerce: timelock not initialized");
        require(newEscrow != address(0), "A2ACommerce: zero escrow");
        
        bytes32 opHash = keccak256(abi.encode(OP_SET_ESCROW, newEscrow, block.timestamp));
        require(timelockOperations[opHash].scheduledTime == 0, "A2ACommerce: already scheduled");
        
        uint256 executionTime = block.timestamp + timelockDelay;
        timelockOperations[opHash] = TimelockOperation({
            operationHash: opHash,
            scheduledTime: block.timestamp,
            executionTime: executionTime,
            executed: false,
            cancelled: false
        });
        
        emit TimelockOperationScheduled(opHash, "setEscrow", executionTime);
    }
    
    /**
     * @dev Execute scheduled escrow update
     */
    function executeSetEscrow(address newEscrow, uint256 scheduledTime) external onlyOwner {
        bytes32 opHash = keccak256(abi.encode(OP_SET_ESCROW, newEscrow, scheduledTime));
        
        TimelockOperation storage op = timelockOperations[opHash];
        require(op.scheduledTime != 0, "A2ACommerce: not scheduled");
        require(!op.executed, "A2ACommerce: already executed");
        require(!op.cancelled, "A2ACommerce: cancelled");
        require(block.timestamp >= op.executionTime, "A2ACommerce: delay not met");
        
        op.executed = true;
        escrow = DealEscrow(payable(newEscrow));
        
        emit TimelockOperationExecuted(opHash);
    }
    
    /**
     * @dev Cancel a scheduled operation
     */
    function cancelTimelockOperation(bytes32 opHash, string calldata reason) external onlyOwner {
        TimelockOperation storage op = timelockOperations[opHash];
        require(op.scheduledTime != 0, "A2ACommerce: not scheduled");
        require(!op.executed, "A2ACommerce: already executed");
        require(!op.cancelled, "A2ACommerce: already cancelled");
        
        op.cancelled = true;
        emit TimelockOperationCancelled(opHash, reason);
    }
    
    /**
     * @dev Check if operation is ready
     */
    function isTimelockOperationReady(bytes32 opHash) external view returns (bool) {
        TimelockOperation storage op = timelockOperations[opHash];
        return op.scheduledTime != 0 && 
               !op.executed && 
               !op.cancelled && 
               block.timestamp >= op.executionTime;
    }
    
    /**
     * @dev Update timelock delay (immediate for initialization, timelocked after)
     */
    function setTimelockDelay(uint256 newDelay) external onlyOwner {
        require(newDelay >= MIN_TIMELOCK_DELAY, "A2ACommerce: delay too short");
        
        uint256 oldDelay = timelockDelay;
        timelockDelay = newDelay;
        
        emit TimelockDelayChanged(oldDelay, newDelay);
    }
    
    // Legacy direct functions (for backward compatibility, can be disabled)
    
    function setPlatformFee(uint256 newFeeBps) external onlyOwner {
        require(newFeeBps <= 1000, "A2ACommerce: max 10%");
        // If timelock is set, require it for fee changes > 1%
        if (timelockDelay > 0 && newFeeBps > platformFeeBasisPoints + 100) {
            revert("A2ACommerce: use scheduleSetPlatformFee for large increases");
        }
        platformFeeBasisPoints = newFeeBps;
    }
    
    function setFeeRecipient(address newRecipient) external onlyOwner {
        require(newRecipient != address(0), "A2ACommerce: zero address");
        feeRecipient = newRecipient;
    }
    
    function setEscrow(address newEscrow) external onlyOwner {
        require(newEscrow != address(0), "A2ACommerce: zero escrow");
        escrow = DealEscrow(payable(newEscrow));
    }
    
    function pause() external onlyOwner {
        _pause();
    }
    
    function unpause() external onlyOwner {
        _unpause();
    }
    
    // Emergency withdrawal
    function emergencyWithdraw(
        address token,
        address to,
        uint256 amount
    ) external onlyOwner {
        require(to != address(0), "A2ACommerce: zero address");
        if (token == address(0)) {
            (bool success, ) = to.call{value: amount}("");
            require(success, "A2ACommerce: withdrawal failed");
        } else {
            IERC20(token).safeTransfer(to, amount);
        }
    }
    
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
    
    // Storage gap
    uint256[50] private __gap;
}
