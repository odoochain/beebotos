// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

/**
 * @title VeBeeToken
 * @notice Vote-escrowed BEE token for governance
 */
contract VeBeeToken is ERC20Upgradeable, OwnableUpgradeable {
    
    struct Lock {
        uint256 amount;
        uint256 start;
        uint256 end;
        uint256 votingPower;
    }
    
    mapping(address => Lock) public locks;
    address public beeToken;
    
    event LockCreated(address indexed user, uint256 amount, uint256 duration);
    event LockExtended(address indexed user, uint256 newDuration);
    event LockWithdrawn(address indexed user, uint256 amount);
    
    function initialize(address _beeToken) public initializer {
        __ERC20_init("Vote-Escrowed BEE", "veBEE");
        __Ownable_init();
        beeToken = _beeToken;
    }
    
    function lock(uint256 amount, uint256 duration) external {
        require(duration >= 7 days, "Min 1 week");
        require(duration <= 4 * 365 days, "Max 4 years");
        require(amount > 0, "Amount > 0");
        
        // Transfer BEE tokens
        (bool success, ) = beeToken.call(
            abi.encodeWithSelector(0x23b872dd, msg.sender, address(this), amount)
        );
        require(success, "Transfer failed");
        
        uint256 votingPower = calculateVotingPower(amount, duration);
        
        locks[msg.sender] = Lock({
            amount: amount,
            start: block.timestamp,
            end: block.timestamp + duration,
            votingPower: votingPower
        });
        
        _mint(msg.sender, votingPower);
        
        emit LockCreated(msg.sender, amount, duration);
    }
    
    function calculateVotingPower(uint256 amount, uint256 duration) public pure returns (uint256) {
        // 1 year = 1x, 4 years = 4x
        return (amount * duration) / (365 days);
    }
    
    function getVotingPower(address user) external view returns (uint256) {
        Lock memory l = locks[user];
        if (block.timestamp > l.end) return 0;
        return l.votingPower;
    }
    
    function withdraw() external {
        Lock memory l = locks[msg.sender];
        require(block.timestamp > l.end, "Lock active");
        require(l.amount > 0, "Nothing to withdraw");
        
        uint256 amount = l.amount;
        delete locks[msg.sender];
        _burn(msg.sender, l.votingPower);
        
        (bool success, ) = beeToken.call(
            abi.encodeWithSelector(0xa9059cbb, msg.sender, amount)
        );
        require(success, "Transfer failed");
        
        emit LockWithdrawn(msg.sender, amount);
    }
}
