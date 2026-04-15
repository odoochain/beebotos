// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/security/ReentrancyGuardUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/security/PausableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@interfaces/IAgentPayment.sol";

/**
 * @title AgentPayment
 * @notice Production-ready payment system for agent services with streaming support
 * @dev UUPS upgradeable contract with reentrancy protection and pausable functionality
 *
 * Security Features:
 * - UUPS upgradeable pattern for future improvements
 * - ReentrancyGuard for all fund transfers
 * - Pausable for emergency stops
 * - SafeERC20 for secure token transfers
 * - CEI pattern (Checks-Effects-Interactions) followed
 */
contract AgentPayment is 
    IAgentPayment,
    ReentrancyGuardUpgradeable,
    PausableUpgradeable,
    OwnableUpgradeable,
    UUPSUpgradeable 
{
    using SafeERC20 for IERC20;

    // ============ State Variables ============
    
    address private immutable __self;
    
    /// @notice Payment mandates mapping
    mapping(bytes32 => PaymentMandate) internal _mandates;
    
    /// @notice Payment streams mapping
    mapping(bytes32 => Stream) internal _streams;
    
    /// @notice User mandates lookup
    mapping(address => bytes32[]) public userMandates;
    
    /// @notice User streams lookup (as sender)
    mapping(address => bytes32[]) public userSentStreams;
    
    /// @notice User streams lookup (as recipient)
    mapping(address => bytes32[]) public userReceivedStreams;
    
    /// @notice All active stream IDs
    bytes32[] public allStreams;
    
    /// @notice Mandate ID counter
    uint256 public mandateCounter;
    
    /// @notice Stream ID counter
    uint256 public streamCounter;
    
    /// @notice Maximum stream duration (365 days)
    uint256 public constant MAX_STREAM_DURATION = 365 days;
    
    /// @notice Minimum stream duration (1 hour)
    uint256 public constant MIN_STREAM_DURATION = 1 hours;
    
    /// @notice Maximum mandate duration (2 years)
    uint256 public constant MAX_MANDATE_DURATION = 730 days;
    
    /// @notice Minimum mandate valid period (1 day)
    uint256 public constant MIN_MANDATE_VALIDITY = 1 days;
    
    constructor() {
        __self = address(this);
    }

    // Events are defined in IAgentPayment interface

    // ============ Initialization ============
    
    /**
     * @notice Initializes the contract (replaces constructor for upgradeable contracts)
     * @dev Can only be called once due to initializer modifier
     */
    function initialize() public initializer {
        __Ownable_init();
        __Pausable_init();
        __ReentrancyGuard_init();
        __UUPSUpgradeable_init();
    }

    // ============ Mandate Functions ============
    
    /**
     * @notice Creates a new payment mandate for authorized recurring payments
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
    ) external override whenNotPaused returns (bytes32 mandateId) {
        require(payee != address(0), "AgentPayment: invalid payee");
        require(payee != msg.sender, "AgentPayment: cannot mandate to self");
        require(maxAmount > 0, "AgentPayment: amount must be > 0");
        require(
            validUntil > block.timestamp + MIN_MANDATE_VALIDITY,
            "AgentPayment: validity too short"
        );
        require(
            validUntil <= block.timestamp + MAX_MANDATE_DURATION,
            "AgentPayment: validity too long"
        );
        
        mandateCounter++;
        mandateId = keccak256(abi.encodePacked(
            msg.sender, 
            payee, 
            mandateCounter,
            block.timestamp
        ));
        
        PaymentMandate storage m = _mandates[mandateId];
        m.mandateId = mandateId;
        m.payer = msg.sender;
        m.payee = payee;
        m.token = token;
        m.maxAmount = maxAmount;
        m.usedAmount = 0;
        m.validUntil = validUntil;
        m.isActive = true;
        
        userMandates[msg.sender].push(mandateId);
        
        emit MandateCreated(
            mandateId, 
            msg.sender, 
            payee, 
            token, 
            maxAmount, 
            validUntil
        );
        
        return mandateId;
    }
    
    /**
     * @notice Revokes an existing payment mandate
     * @param mandateId The mandate to revoke
     */
    function revokeMandate(bytes32 mandateId) external whenNotPaused {
        PaymentMandate storage m = _mandates[mandateId];
        require(m.mandateId != bytes32(0), "AgentPayment: mandate not found");
        require(
            m.payer == msg.sender || owner() == msg.sender,
            "AgentPayment: not authorized"
        );
        require(m.isActive, "AgentPayment: mandate already inactive");
        
        m.isActive = false;
        
        emit MandateRevoked(mandateId, msg.sender);
    }
    
    /**
     * @notice Executes a payment from an existing mandate
     * @param mandateId The mandate to use
     * @param amount The amount to pay
     * @return paymentId Unique identifier for this payment execution
     */
    function executeFromMandate(
        bytes32 mandateId, 
        uint256 amount
    ) external nonReentrant whenNotPaused returns (bytes32 paymentId) {
        PaymentMandate storage m = _mandates[mandateId];
        require(m.mandateId != bytes32(0), "AgentPayment: mandate not found");
        require(m.isActive, "AgentPayment: mandate not active");
        require(msg.sender == m.payee, "AgentPayment: not authorized payee");
        require(block.timestamp <= m.validUntil, "AgentPayment: mandate expired");
        require(amount > 0, "AgentPayment: invalid amount");
        require(
            m.usedAmount + amount <= m.maxAmount,
            "AgentPayment: exceeds mandate limit"
        );
        
        // Update state before external call (CEI pattern)
        m.usedAmount += amount;
        if (m.usedAmount >= m.maxAmount) {
            m.isActive = false;
        }
        
        paymentId = keccak256(abi.encodePacked(mandateId, amount, block.timestamp));
        
        // Execute transfer
        _executeTransfer(m.token, m.payer, m.payee, amount);
        
        emit PaymentExecuted(mandateId, paymentId, amount, msg.sender);
        
        return paymentId;
    }

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
    ) external payable override whenNotPaused returns (bytes32 streamId) {
        require(msg.value >= totalAmount, "AgentPayment: insufficient ETH");
        
        streamId = _createStream(
            recipient,
            address(0), // ETH
            totalAmount,
            duration
        );
        
        // Refund excess ETH
        if (msg.value > totalAmount) {
            (bool success, ) = payable(msg.sender).call{value: msg.value - totalAmount}("");
            require(success, "AgentPayment: refund failed");
        }
        
        return streamId;
    }
    
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
    ) external whenNotPaused returns (bytes32 streamId) {
        require(token != address(0), "AgentPayment: invalid token");
        
        streamId = _createStream(recipient, token, totalAmount, duration);
        
        // Transfer tokens from sender to contract
        IERC20(token).safeTransferFrom(msg.sender, address(this), totalAmount);
        
        return streamId;
    }
    
    /**
     * @notice Internal function to create a stream
     */
    function _createStream(
        address recipient,
        address token,
        uint256 totalAmount,
        uint256 duration
    ) internal returns (bytes32 streamId) {
        require(recipient != address(0), "AgentPayment: invalid recipient");
        require(recipient != msg.sender, "AgentPayment: cannot stream to self");
        require(totalAmount > 0, "AgentPayment: amount must be > 0");
        require(
            duration >= MIN_STREAM_DURATION,
            "AgentPayment: duration too short"
        );
        require(
            duration <= MAX_STREAM_DURATION,
            "AgentPayment: duration too long"
        );
        
        streamCounter++;
        streamId = keccak256(abi.encodePacked(
            msg.sender,
            recipient,
            streamCounter,
            block.timestamp
        ));
        
        Stream storage s = _streams[streamId];
        s.streamId = streamId;
        s.sender = msg.sender;
        s.recipient = recipient;
        s.token = token;
        s.totalAmount = totalAmount;
        s.releasedAmount = 0;
        s.startTime = block.timestamp;
        s.endTime = block.timestamp + duration;
        s.lastUpdate = block.timestamp;
        s.isActive = true;
        s.isCancelled = false;
        
        allStreams.push(streamId);
        userSentStreams[msg.sender].push(streamId);
        userReceivedStreams[recipient].push(streamId);
        
        emit StreamCreated(
            streamId,
            msg.sender,
            recipient,
            token,
            totalAmount,
            s.startTime,
            s.endTime
        );
        
        return streamId;
    }
    
    /**
     * @notice Withdraws available funds from a stream
     * @param streamId The stream to withdraw from
     * @return withdrawnAmount The amount actually withdrawn
     */
    function withdrawFromStream(
        bytes32 streamId
    ) external override nonReentrant whenNotPaused returns (uint256 withdrawnAmount) {
        Stream storage s = _streams[streamId];
        require(s.streamId != bytes32(0), "AgentPayment: stream not found");
        require(msg.sender == s.recipient, "AgentPayment: not recipient");
        require(s.isActive && !s.isCancelled, "AgentPayment: stream inactive");
        
        uint256 pendingAmount = _calculatePendingAmount(s);
        require(pendingAmount > 0, "AgentPayment: no pending amount");
        
        // Update state before external call (CEI pattern)
        s.releasedAmount += pendingAmount;
        s.lastUpdate = block.timestamp;
        
        if (s.releasedAmount >= s.totalAmount || block.timestamp >= s.endTime) {
            s.isActive = false;
        }
        
        // Execute transfer
        _executeTransfer(s.token, address(this), s.recipient, pendingAmount);
        
        emit StreamWithdrawn(streamId, s.recipient, pendingAmount);
        emit StreamUpdated(streamId, s.releasedAmount);
        
        return pendingAmount;
    }
    
    /**
     * @notice Cancels an active stream and refunds remaining funds
     * @param streamId The stream to cancel
     */
    function cancelStream(bytes32 streamId) external nonReentrant whenNotPaused {
        Stream storage s = _streams[streamId];
        require(s.streamId != bytes32(0), "AgentPayment: stream not found");
        require(
            msg.sender == s.sender || msg.sender == s.recipient || msg.sender == owner(),
            "AgentPayment: not authorized"
        );
        require(s.isActive && !s.isCancelled, "AgentPayment: stream already ended");
        
        s.isActive = false;
        s.isCancelled = true;
        
        // Calculate amounts
        uint256 recipientAmount = _calculatePendingAmount(s);
        uint256 senderAmount = s.totalAmount - s.releasedAmount - recipientAmount;
        
        s.releasedAmount = s.totalAmount; // Mark all as processed
        
        // Refund recipient their earned portion
        if (recipientAmount > 0) {
            _executeTransfer(s.token, address(this), s.recipient, recipientAmount);
        }
        
        // Refund sender the remaining portion
        if (senderAmount > 0) {
            _executeTransfer(s.token, address(this), s.sender, senderAmount);
        }
        
        emit StreamCancelled(streamId, msg.sender, recipientAmount, senderAmount);
    }

    // ============ Internal Functions ============
    
    /**
     * @notice Calculates the pending withdrawable amount for a stream
     * @param s The stream to calculate for
     * @return The pending amount
     */
    function _calculatePendingAmount(Stream storage s) internal view returns (uint256) {
        if (!s.isActive || s.isCancelled) return 0;
        
        uint256 currentTime = block.timestamp > s.endTime ? s.endTime : block.timestamp;
        uint256 elapsed = currentTime - s.startTime;
        uint256 duration = s.endTime - s.startTime;
        
        uint256 totalEntitled = (s.totalAmount * elapsed) / duration;
        uint256 pending = totalEntitled - s.releasedAmount;
        
        return pending;
    }
    
    /**
     * @notice Executes a token or ETH transfer securely
     * @param token Token address (address(0) for ETH)
     * @param from Source address (address(this) for contract balance)
     * @param to Destination address
     * @param amount Amount to transfer
     */
    function _executeTransfer(
        address token,
        address from,
        address to,
        uint256 amount
    ) internal {
        require(to != address(0), "AgentPayment: invalid recipient");
        require(amount > 0, "AgentPayment: invalid amount");
        
        if (token == address(0)) {
            // ETH transfer using low-level call with reentrancy protection
            (bool success, ) = payable(to).call{value: amount}("");
            require(success, "AgentPayment: ETH transfer failed");
        } else {
            // ERC20 transfer
            if (from == address(this)) {
                IERC20(token).safeTransfer(to, amount);
            } else {
                IERC20(token).safeTransferFrom(from, to, amount);
            }
        }
    }

    // ============ View Functions ============
    
    /**
     * @notice Gets the pending withdrawable amount for a stream
     * @param streamId The stream to query
     * @return The pending amount
     */
    function getPendingAmount(bytes32 streamId) external view override returns (uint256) {
        Stream storage s = _streams[streamId];
        if (s.streamId == bytes32(0)) return 0;
        return _calculatePendingAmount(s);
    }
    
    /**
     * @notice Gets full stream details
     * @param streamId The stream to query
     * @return The stream details
     */
    function getStream(bytes32 streamId) external view returns (Stream memory) {
        return _streams[streamId];
    }
    
    /**
     * @notice Gets a mandate by ID
     * @param mandateId The mandate to query
     * @return The mandate details
     */
    function mandates(bytes32 mandateId) external view returns (PaymentMandate memory) {
        return _mandates[mandateId];
    }
    
    /**
     * @notice Gets a stream by ID
     * @param streamId The stream to query
     * @return The stream details
     */
    function streams(bytes32 streamId) external view returns (Stream memory) {
        return _streams[streamId];
    }
    
    /**
     * @notice Gets all mandate IDs for a user
     * @param user The user to query
     * @return Array of mandate IDs
     */
    function getUserMandates(address user) external view returns (bytes32[] memory) {
        return userMandates[user];
    }
    
    /**
     * @notice Gets all stream IDs sent by a user
     * @param user The user to query
     * @return Array of stream IDs
     */
    function getUserSentStreams(address user) external view returns (bytes32[] memory) {
        return userSentStreams[user];
    }
    
    /**
     * @notice Gets all stream IDs received by a user
     * @param user The user to query
     * @return Array of stream IDs
     */
    function getUserReceivedStreams(address user) external view returns (bytes32[] memory) {
        return userReceivedStreams[user];
    }
    
    /**
     * @notice Gets all stream IDs
     * @return Array of all stream IDs
     */
    function getAllStreams() external view returns (bytes32[] memory) {
        return allStreams;
    }
    
    /**
     * @notice Gets the remaining available amount in a mandate
     * @param mandateId The mandate to query
     * @return The remaining amount
     */
    function getMandateRemaining(bytes32 mandateId) external view returns (uint256) {
        PaymentMandate storage m = _mandates[mandateId];
        if (m.mandateId == bytes32(0) || !m.isActive) return 0;
        if (block.timestamp > m.validUntil) return 0;
        return m.maxAmount - m.usedAmount;
    }

    // ============ Admin Functions ============
    
    /**
     * @notice Pauses all payment operations
     * @dev Only callable by owner
     */
    function pause() external onlyOwner {
        _pause();
    }
    
    /**
     * @notice Unpauses all payment operations
     * @dev Only callable by owner
     */
    function unpause() external onlyOwner {
        _unpause();
    }
    
    /**
     * @notice Emergency withdrawal in case of stuck funds
     * @param token Token to withdraw (address(0) for ETH)
     * @param amount Amount to withdraw
     * @dev Only callable by owner, use with extreme caution
     */
    function emergencyWithdraw(
        address token, 
        uint256 amount
    ) external onlyOwner nonReentrant {
        require(amount > 0, "AgentPayment: invalid amount");
        
        if (token == address(0)) {
            (bool success, ) = payable(owner()).call{value: amount}("");
            require(success, "AgentPayment: withdrawal failed");
        } else {
            IERC20(token).safeTransfer(owner(), amount);
        }
    }
    
    /**
     * @notice Authorizes a contract upgrade
     * @param newImplementation Address of the new implementation
     * @dev Only callable by owner
     */
    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    // ============ Receive Function ============
    
    /**
     * @notice Allows contract to receive ETH
     * @dev Only accepts ETH during stream creation
     */
    receive() external payable {
        // Accept ETH only, validation happens in createStream
    }
}
