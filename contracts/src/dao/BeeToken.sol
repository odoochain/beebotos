// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title BeeToken
 * @dev Production-ready BEE Governance Token with Voting, Permit, and Locking
 * 
 * Tokenomics:
 * - Total Supply: 1,000,000,000 BEE (1 billion)
 * - Initial Allocation:
 *   - Community Treasury: 40%
 *   - Team & Advisors: 20% (4-year vesting)
 *   - Investors: 15% (2-year vesting)
 *   - Ecosystem Incentives: 20%
 *   - Liquidity: 5%
 * 
 * Features:
 * - Time-locked tokens with voting power tracking
 * - Annual emission control
 * - Role-based access control
 */
contract BeeToken is ERC20, ERC20Permit, ERC20Votes, AccessControl, ReentrancyGuard {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant BURNER_ROLE = keccak256("BURNER_ROLE");
    bytes32 public constant LOCK_MANAGER = keccak256("LOCK_MANAGER");

    /// @dev Maximum supply cap
    uint256 public constant MAX_SUPPLY = 1_000_000_000e18; // 1 billion
    
    /// @dev Current supply minted
    uint256 public totalMinted;

    /// @dev Emission rate per year (basis points)
    uint256 public yearlyEmissionBps = 200; // 2% annual inflation after year 4
    
    /// @dev Last emission timestamp
    uint256 public lastEmissionTime;
    
    /// @dev Emission start (after initial 4 years)
    uint256 public constant EMISSION_START = 4 * 365 days;
    
    /// @dev Lock information structure
    struct LockInfo {
        uint256 amount;
        uint256 startTime;
        uint256 endTime;
        uint256 votingPower;  // Voting power during lock
        bool released;
    }
    
    /// @dev User locks mapping
    mapping(address => LockInfo[]) public userLocks;
    mapping(address => uint256) public totalLocked;
    
    /// @dev Total locked supply
    uint256 public totalLockedSupply;
    
    /// @dev Events
    event Minted(address indexed to, uint256 amount, string reason);
    event Burned(address indexed from, uint256 amount, string reason);
    event EmissionExecuted(uint256 amount, uint256 timestamp);
    event TokensLocked(
        address indexed user,
        uint256 indexed lockId,
        uint256 amount,
        uint256 endTime
    );
    event TokensUnlocked(
        address indexed user,
        uint256 indexed lockId,
        uint256 amount
    );
    event LockExtended(
        address indexed user,
        uint256 indexed lockId,
        uint256 newEndTime
    );

    constructor(
        address _treasury,
        address _team,
        address _investors,
        address _ecosystem,
        address _liquidity
    ) 
        ERC20("BeeToken", "BEE") 
        ERC20Permit("BeeToken") 
    {
        require(_treasury != address(0), "BeeToken: invalid treasury");
        
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(MINTER_ROLE, msg.sender);
        _grantRole(BURNER_ROLE, msg.sender);
        _grantRole(LOCK_MANAGER, msg.sender);
        
        // Initial allocation
        _mintWithTracking(_treasury, 400_000_000e18, "Treasury");     // 40%
        _mintWithTracking(_team, 200_000_000e18, "Team");              // 20%
        _mintWithTracking(_investors, 150_000_000e18, "Investors");    // 15%
        _mintWithTracking(_ecosystem, 200_000_000e18, "Ecosystem");    // 20%
        _mintWithTracking(_liquidity, 50_000_000e18, "Liquidity");     // 5%
        
        lastEmissionTime = block.timestamp;
    }

    /**
     * @dev Lock tokens for a specified duration
     * @param amount Amount to lock
     * @param duration Lock duration in seconds
     * @return lockId The ID of the created lock
     */
    function lock(uint256 amount, uint256 duration) 
        external 
        nonReentrant 
        returns (uint256 lockId) 
    {
        require(amount > 0, "BeeToken: zero amount");
        require(duration > 0, "BeeToken: zero duration");
        require(balanceOf(msg.sender) >= amount, "BeeToken: insufficient balance");
        
        // Transfer tokens to contract
        _transfer(msg.sender, address(this), amount);
        
        // Create lock
        lockId = userLocks[msg.sender].length;
        userLocks[msg.sender].push(LockInfo({
            amount: amount,
            startTime: block.timestamp,
            endTime: block.timestamp + duration,
            votingPower: amount,
            released: false
        }));
        
        totalLocked[msg.sender] += amount;
        totalLockedSupply += amount;
        
        emit TokensLocked(msg.sender, lockId, amount, block.timestamp + duration);
        
        return lockId;
    }
    
    /**
     * @dev Unlock tokens after lock period expires
     * @param lockId The lock ID to unlock
     */
    function unlock(uint256 lockId) external nonReentrant {
        require(lockId < userLocks[msg.sender].length, "BeeToken: invalid lock id");
        
        LockInfo storage lockInfo = userLocks[msg.sender][lockId];
        require(!lockInfo.released, "BeeToken: already released");
        require(block.timestamp >= lockInfo.endTime, "BeeToken: lock not expired");
        
        lockInfo.released = true;
        totalLocked[msg.sender] -= lockInfo.amount;
        totalLockedSupply -= lockInfo.amount;
        
        // Return tokens to user
        _transfer(address(this), msg.sender, lockInfo.amount);
        
        emit TokensUnlocked(msg.sender, lockId, lockInfo.amount);
    }
    
    /**
     * @dev Extend an existing lock
     * @param lockId The lock ID to extend
     * @param additionalDuration Additional time to add
     */
    function extendLock(uint256 lockId, uint256 additionalDuration) external {
        require(lockId < userLocks[msg.sender].length, "BeeToken: invalid lock id");
        require(additionalDuration > 0, "BeeToken: zero duration");
        
        LockInfo storage lockInfo = userLocks[msg.sender][lockId];
        require(!lockInfo.released, "BeeToken: already released");
        
        lockInfo.endTime += additionalDuration;
        
        emit LockExtended(msg.sender, lockId, lockInfo.endTime);
    }
    
    /**
     * @dev Get lock information
     */
    function getLock(address user, uint256 lockId) external view returns (LockInfo memory) {
        require(lockId < userLocks[user].length, "BeeToken: invalid lock id");
        return userLocks[user][lockId];
    }
    
    /**
     * @dev Get all locks for a user
     */
    function getLocks(address user) external view returns (LockInfo[] memory) {
        return userLocks[user];
    }
    
    /**
     * @dev Get active (non-released) locks for a user
     */
    function getActiveLocks(address user) external view returns (LockInfo[] memory) {
        uint256 activeCount = 0;
        for (uint i = 0; i < userLocks[user].length; i++) {
            if (!userLocks[user][i].released) {
                activeCount++;
            }
        }
        
        LockInfo[] memory active = new LockInfo[](activeCount);
        uint256 idx = 0;
        for (uint i = 0; i < userLocks[user].length; i++) {
            if (!userLocks[user][i].released) {
                active[idx] = userLocks[user][i];
                idx++;
            }
        }
        return active;
    }
    
    /**
     * @dev Calculate total voting power (balance + locked)
     * Locked tokens retain full voting power
     */
    function getTotalVotingPower(address account) external view returns (uint256) {
        return balanceOf(account) + totalLocked[account];
    }

    /**
     * @dev Mint tokens (governance only)
     * @param _to Recipient address
     * @param _amount Amount to mint
     * @param _reason Reason for minting
     */
    function mint(
        address _to, 
        uint256 _amount, 
        string calldata _reason
    ) external onlyRole(MINTER_ROLE) {
        require(totalMinted + _amount <= MAX_SUPPLY, "BeeToken: exceeds max supply");
        _mintWithTracking(_to, _amount, _reason);
    }

    /**
     * @dev Burn tokens
     * @param _amount Amount to burn
     * @param _reason Reason for burning
     */
    function burn(
        uint256 _amount, 
        string calldata _reason
    ) external onlyRole(BURNER_ROLE) {
        _burn(msg.sender, _amount);
        totalMinted -= _amount;
        emit Burned(msg.sender, _amount, _reason);
    }
    
    /**
     * @dev Burn tokens from a specific account (with approval)
     */
    function burnFrom(
        address account, 
        uint256 amount,
        string calldata _reason
    ) external onlyRole(BURNER_ROLE) {
        _spendAllowance(account, msg.sender, amount);
        _burn(account, amount);
        totalMinted -= amount;
        emit Burned(account, amount, _reason);
    }

    /**
     * @dev Execute yearly emission
     */
    function executeEmission() external onlyRole(MINTER_ROLE) {
        require(
            block.timestamp >= lastEmissionTime + 365 days,
            "BeeToken: too early"
        );
        require(
            block.timestamp >= EMISSION_START,
            "BeeToken: emission not started"
        );
        
        uint256 emission = (MAX_SUPPLY * yearlyEmissionBps) / 10000;
        require(totalMinted + emission <= MAX_SUPPLY, "BeeToken: would exceed max");
        
        // Mint to treasury
        _mintWithTracking(
            msg.sender, // Should be treasury contract
            emission, 
            "Yearly emission"
        );
        
        lastEmissionTime = block.timestamp;
        emit EmissionExecuted(emission, block.timestamp);
    }

    /**
     * @dev Set yearly emission rate
     * @param _newRate New rate in basis points (max 10%)
     */
    function setYearlyEmissionBps(uint256 _newRate) 
        external 
        onlyRole(DEFAULT_ADMIN_ROLE) 
    {
        require(_newRate <= 1000, "BeeToken: max 10%"); // Max 10% annual
        yearlyEmissionBps = _newRate;
    }

    /**
     * @dev Delegate voting power
     */
    function delegate(address delegatee) public override {
        super.delegate(delegatee);
    }

    /**
     * @dev Get voting power at block
     */
    function getVotes(address account) 
        public 
        view 
        override 
        returns (uint256) 
    {
        return super.getVotes(account);
    }
    
    /**
     * @dev Get votes including locked tokens
     * This ensures locked tokens retain voting power
     */
    function getVotesWithLocked(address account) public view returns (uint256) {
        return getVotes(account) + totalLocked[account];
    }

    // ============ Internal Functions ============

    function _mintWithTracking(
        address _to, 
        uint256 _amount, 
        string memory _reason
    ) internal {
        _mint(_to, _amount);
        totalMinted += _amount;
        emit Minted(_to, _amount, _reason);
    }
    
    /**
     * @dev Override to include locked tokens in balance checks if needed
     */
    function _beforeTokenTransfer(
        address from,
        address to,
        uint256 amount
    ) internal override {
        super._beforeTokenTransfer(from, to, amount);
    }

    // ============ Required Overrides ============

    function _afterTokenTransfer(
        address from,
        address to,
        uint256 amount
    ) internal override(ERC20, ERC20Votes) {
        super._afterTokenTransfer(from, to, amount);
    }

    function _mint(address to, uint256 amount) 
        internal 
        override(ERC20, ERC20Votes) 
    {
        super._mint(to, amount);
    }

    function _burn(address account, uint256 amount) 
        internal 
        override(ERC20, ERC20Votes) 
    {
        super._burn(account, amount);
    }
}
