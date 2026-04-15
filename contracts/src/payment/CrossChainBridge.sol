// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

/**
 * @title CrossChainBridge
 * @dev Production-ready cross-chain bridge with multi-signature verification
 * 
 * Security Features:
 * - Multi-signature validation for cross-chain operations
 * - Pausing mechanism for emergency response
 * - Request nonce tracking to prevent replay attacks
 * - Validator set management with threshold
 */
contract CrossChainBridge is Ownable, Pausable, ReentrancyGuard {
    using SafeERC20 for IERC20;
    using ECDSA for bytes32;
    
    struct BridgeRequest {
        bytes32 requestId;
        address sender;
        address recipient;
        uint256 amount;
        address token;
        uint256 targetChain;
        bytes32 targetToken;
        BridgeState state;
        uint256 timestamp;
        uint256 nonce;  // Added for replay protection
    }
    
    enum BridgeState {
        Pending,
        Locked,
        Confirmed,
        Completed,
        Failed,
        Refunded
    }
    
    // State mappings
    mapping(bytes32 => BridgeRequest) public requests;
    mapping(uint256 => bool) public supportedChains;
    mapping(address => bool) public supportedTokens;
    mapping(bytes32 => bool) public completedRequests;
    mapping(address => bool) public validators;
    mapping(bytes32 => mapping(address => bool)) public hasSigned;  // Track who signed each request
    mapping(address => uint256) public validatorRewards;
    
    // Validator management
    address[] public validatorList;
    uint256 public requiredSignatures;
    uint256 public validatorCount;
    
    // Fee configuration
    uint256 public feeBasisPoints = 30; // 0.3%
    uint256 public constant MAX_FEE = 100; // 1% (in basis points)
    uint256 public constant BASIS_POINTS = 10000;
    
    // Security parameters
    uint256 public constant MIN_LOCK_DURATION = 1 hours;
    uint256 public constant MAX_LOCK_DURATION = 7 days;
    uint256 public requestCounter;  // Monotonic nonce counter
    
    // Events
    event BridgeInitiated(
        bytes32 indexed requestId,
        address indexed sender,
        uint256 targetChain,
        uint256 amount,
        uint256 nonce
    );
    event BridgeCompleted(
        bytes32 indexed requestId,
        address indexed recipient,
        uint256 amount,
        uint256 validatorCount
    );
    event BridgeFailed(bytes32 indexed requestId, string reason);
    event BridgeRefunded(bytes32 indexed requestId, address indexed recipient, uint256 amount);
    event ValidatorAdded(address indexed validator);
    event ValidatorRemoved(address indexed validator);
    event RequiredSignaturesChanged(uint256 oldValue, uint256 newValue);
    event ValidatorRewardClaimed(address indexed validator, uint256 amount);
    event ProofSubmitted(bytes32 indexed requestId, address indexed validator, bytes32 signatureHash);
    
    // Modifiers
    modifier onlySupportedChain(uint256 chainId) {
        require(supportedChains[chainId], "CrossChainBridge: unsupported chain");
        _;
    }
    
    modifier onlySupportedToken(address token) {
        require(supportedTokens[token] || token == address(0), "CrossChainBridge: unsupported token");
        _;
    }
    
    modifier onlyValidator() {
        require(validators[msg.sender], "CrossChainBridge: not a validator");
        _;
    }
    
    modifier nonReentrantRequest(bytes32 requestId) {
        require(requests[requestId].sender == address(0), "CrossChainBridge: request already exists");
        _;
    }
    
    /**
     * @dev Constructor initializes the bridge with validator set
     * @param _validators Initial validator addresses
     * @param _requiredSignatures Number of signatures required for validation
     */
    constructor(address[] memory _validators, uint256 _requiredSignatures) {
        require(_validators.length > 0, "CrossChainBridge: empty validator set");
        require(
            _requiredSignatures > 0 && _requiredSignatures <= _validators.length,
            "CrossChainBridge: invalid required signatures"
        );
        
        requiredSignatures = _requiredSignatures;
        
        for (uint i = 0; i < _validators.length; i++) {
            require(_validators[i] != address(0), "CrossChainBridge: zero validator");
            require(!validators[_validators[i]], "CrossChainBridge: duplicate validator");
            validators[_validators[i]] = true;
            validatorList.push(_validators[i]);
        }
        validatorCount = _validators.length;
    }
    
    /**
     * @dev Initiate a cross-chain transfer
     * @param token Token address (address(0) for ETH)
     * @param amount Amount to transfer
     * @param targetChain Destination chain ID
     * @param targetToken Token address on target chain
     * @param recipient Recipient address on target chain
     * @return requestId Unique identifier for this bridge request
     */
    function bridgeOut(
        address token,
        uint256 amount,
        uint256 targetChain,
        bytes32 targetToken,
        address recipient
    )
        external
        payable
        nonReentrant
        whenNotPaused
        onlySupportedChain(targetChain)
        onlySupportedToken(token)
        returns (bytes32)
    {
        require(amount > 0, "CrossChainBridge: invalid amount");
        require(recipient != address(0), "CrossChainBridge: invalid recipient");
        require(targetToken != bytes32(0), "CrossChainBridge: invalid target token");
        
        // Generate unique request ID with nonce for replay protection
        uint256 nonce = ++requestCounter;
        bytes32 requestId = keccak256(abi.encodePacked(
            msg.sender,
            token,
            amount,
            targetChain,
            targetToken,
            recipient,
            nonce,
            block.chainid,
            block.timestamp
        ));
        
        require(requests[requestId].sender == address(0), "CrossChainBridge: request exists");
        
        // Calculate and collect fee
        uint256 fee = (amount * feeBasisPoints) / BASIS_POINTS;
        uint256 netAmount = amount - fee;
        
        // Lock assets
        if (token == address(0)) {
            require(msg.value == amount, "CrossChainBridge: invalid ETH amount");
        } else {
            require(msg.value == 0, "CrossChainBridge: ETH not accepted for ERC20");
            IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
        }
        
        requests[requestId] = BridgeRequest({
            requestId: requestId,
            sender: msg.sender,
            recipient: recipient,
            amount: netAmount,
            token: token,
            targetChain: targetChain,
            targetToken: targetToken,
            state: BridgeState.Locked,
            timestamp: block.timestamp,
            nonce: nonce
        });
        
        emit BridgeInitiated(requestId, msg.sender, targetChain, netAmount, nonce);
        
        return requestId;
    }
    
    /**
     * @dev Complete a bridge transfer on target chain with multi-sig verification
     * @param requestId The bridge request ID
     * @param recipient Recipient address
     * @param amount Amount to release
     * @param token Token address
     * @param signatures Array of validator signatures
     */
    function bridgeIn(
        bytes32 requestId,
        address recipient,
        uint256 amount,
        address token,
        bytes[] calldata signatures
    )
        external
        nonReentrant
        whenNotPaused
    {
        require(!completedRequests[requestId], "CrossChainBridge: already completed");
        require(signatures.length >= requiredSignatures, "CrossChainBridge: insufficient signatures");
        
        // Verify multi-signature proof
        require(
            verifyCrossChainProof(requestId, recipient, amount, token, signatures),
            "CrossChainBridge: invalid proof"
        );
        
        completedRequests[requestId] = true;
        
        // Release assets
        if (token == address(0)) {
            require(address(this).balance >= amount, "CrossChainBridge: insufficient ETH");
            (bool success, ) = recipient.call{value: amount}("");
            require(success, "CrossChainBridge: ETH transfer failed");
        } else {
            require(
                IERC20(token).balanceOf(address(this)) >= amount,
                "CrossChainBridge: insufficient token balance"
            );
            IERC20(token).safeTransfer(recipient, amount);
        }
        
        emit BridgeCompleted(requestId, recipient, amount, signatures.length);
    }
    
    /**
     * @dev Verify cross-chain proof with multi-signature validation
     * @param requestId The bridge request ID
     * @param recipient Expected recipient
     * @param amount Expected amount
     * @param token Expected token
     * @param signatures Array of validator signatures
     * @return bool True if proof is valid
     */
    function verifyCrossChainProof(
        bytes32 requestId,
        address recipient,
        uint256 amount,
        address token,
        bytes[] calldata signatures
    ) public view returns (bool) {
        if (signatures.length < requiredSignatures) {
            return false;
        }
        
        // Create message hash
        bytes32 messageHash = keccak256(abi.encodePacked(
            requestId,
            recipient,
            amount,
            token,
            block.chainid  // Include chain ID to prevent cross-chain replay
        ));
        
        bytes32 ethSignedMessageHash = messageHash.toEthSignedMessageHash();
        
        // Verify unique signers and count valid signatures
        address[] memory uniqueSigners = new address[](signatures.length);
        uint256 validCount = 0;
        
        for (uint i = 0; i < signatures.length; i++) {
            address signer = ethSignedMessageHash.recover(signatures[i]);
            
            // Check if valid validator
            if (!validators[signer]) {
                continue;
            }
            
            // Check for duplicate signatures
            bool isDuplicate = false;
            for (uint j = 0; j < validCount; j++) {
                if (uniqueSigners[j] == signer) {
                    isDuplicate = true;
                    break;
                }
            }
            
            if (!isDuplicate) {
                uniqueSigners[validCount] = signer;
                validCount++;
            }
        }
        
        return validCount >= requiredSignatures;
    }
    
    /**
     * @dev Submit proof for a bridge request (for validators)
     * @param requestId The bridge request ID
     * @param signatures Validator signatures
     */
    function submitProof(
        bytes32 requestId,
        bytes[] calldata signatures
    ) external onlyValidator {
        BridgeRequest storage request = requests[requestId];
        require(request.sender != address(0), "CrossChainBridge: request not found");
        require(request.state == BridgeState.Locked, "CrossChainBridge: invalid state");
        
        require(
            verifyCrossChainProof(requestId, request.recipient, request.amount, request.token, signatures),
            "CrossChainBridge: invalid proof"
        );
        
        request.state = BridgeState.Confirmed;
        
        emit ProofSubmitted(requestId, msg.sender, keccak256(abi.encode(signatures)));
    }
    
    /**
     * @dev Refund a failed or expired bridge request
     * @param requestId The bridge request ID
     */
    function refund(bytes32 requestId) external nonReentrant {
        BridgeRequest storage request = requests[requestId];
        require(request.sender != address(0), "CrossChainBridge: request not found");
        require(
            request.sender == msg.sender || msg.sender == owner(),
            "CrossChainBridge: not authorized"
        );
        require(request.state == BridgeState.Locked, "CrossChainBridge: invalid state");
        require(
            block.timestamp > request.timestamp + MAX_LOCK_DURATION,
            "CrossChainBridge: refund window not open"
        );
        
        request.state = BridgeState.Refunded;
        
        // Return assets
        if (request.token == address(0)) {
            (bool success, ) = request.sender.call{value: request.amount}("");
            require(success, "CrossChainBridge: refund failed");
        } else {
            IERC20(request.token).safeTransfer(request.sender, request.amount);
        }
        
        emit BridgeRefunded(requestId, request.sender, request.amount);
    }
    
    // ============ Admin Functions ============
    
    /**
     * @dev Add a new validator
     * @param validator Validator address
     */
    function addValidator(address validator) external onlyOwner {
        require(validator != address(0), "CrossChainBridge: zero address");
        require(!validators[validator], "CrossChainBridge: already validator");
        
        validators[validator] = true;
        validatorList.push(validator);
        validatorCount++;
        
        emit ValidatorAdded(validator);
    }
    
    /**
     * @dev Remove a validator
     * @param validator Validator address
     */
    function removeValidator(address validator) external onlyOwner {
        require(validators[validator], "CrossChainBridge: not a validator");
        require(
            validatorCount > requiredSignatures,
            "CrossChainBridge: cannot remove below threshold"
        );
        
        validators[validator] = false;
        validatorCount--;
        
        // Remove from list (swap and pop)
        for (uint i = 0; i < validatorList.length; i++) {
            if (validatorList[i] == validator) {
                validatorList[i] = validatorList[validatorList.length - 1];
                validatorList.pop();
                break;
            }
        }
        
        emit ValidatorRemoved(validator);
    }
    
    /**
     * @dev Update required signature threshold
     * @param newRequired New number of required signatures
     */
    function setRequiredSignatures(uint256 newRequired) external onlyOwner {
        require(
            newRequired > 0 && newRequired <= validatorCount,
            "CrossChainBridge: invalid threshold"
        );
        
        uint256 oldRequired = requiredSignatures;
        requiredSignatures = newRequired;
        
        emit RequiredSignaturesChanged(oldRequired, newRequired);
    }
    
    /**
     * @dev Add supported chain
     * @param chainId Chain ID to support
     */
    function addSupportedChain(uint256 chainId) external onlyOwner {
        supportedChains[chainId] = true;
    }
    
    /**
     * @dev Remove supported chain
     * @param chainId Chain ID to remove
     */
    function removeSupportedChain(uint256 chainId) external onlyOwner {
        supportedChains[chainId] = false;
    }
    
    /**
     * @dev Add supported token
     * @param token Token address
     */
    function addSupportedToken(address token) external onlyOwner {
        require(token != address(0), "CrossChainBridge: zero address");
        supportedTokens[token] = true;
    }
    
    /**
     * @dev Remove supported token
     * @param token Token address
     */
    function removeSupportedToken(address token) external onlyOwner {
        supportedTokens[token] = false;
    }
    
    /**
     * @dev Update fee rate (max 1%)
     * @param newFee New fee in basis points
     */
    function setFee(uint256 newFee) external onlyOwner {
        require(newFee <= MAX_FEE, "CrossChainBridge: fee too high");
        feeBasisPoints = newFee;
    }
    
    /**
     * @dev Withdraw accumulated fees
     * @param token Token to withdraw (address(0) for ETH)
     */
    function withdrawFees(address token) external onlyOwner {
        uint256 balance;
        if (token == address(0)) {
            balance = address(this).balance;
            require(balance > 0, "CrossChainBridge: no ETH to withdraw");
            (bool success, ) = owner().call{value: balance}("");
            require(success, "CrossChainBridge: withdrawal failed");
        } else {
            balance = IERC20(token).balanceOf(address(this));
            require(balance > 0, "CrossChainBridge: no tokens to withdraw");
            IERC20(token).safeTransfer(owner(), balance);
        }
    }
    
    /**
     * @dev Emergency pause
     */
    function pause() external onlyOwner {
        _pause();
    }
    
    /**
     * @dev Unpause
     */
    function unpause() external onlyOwner {
        _unpause();
    }
    
    // ============ View Functions ============
    
    /**
     * @dev Get all validators
     */
    function getValidators() external view returns (address[] memory) {
        return validatorList;
    }
    
    /**
     * @dev Check if request is expired
     */
    function isRequestExpired(bytes32 requestId) external view returns (bool) {
        BridgeRequest storage request = requests[requestId];
        if (request.sender == address(0)) return false;
        return block.timestamp > request.timestamp + MAX_LOCK_DURATION;
    }
    
    receive() external payable {
        require(msg.value > 0, "CrossChainBridge: zero value");
    }
}
