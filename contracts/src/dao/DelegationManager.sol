// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

/**
 * @title DelegationManager
 * @notice Manages voting power delegation for agents and humans
 */
contract DelegationManager is OwnableUpgradeable {
    
    struct Delegation {
        address delegator;
        address delegatee;
        uint256 amount;
        uint256 delegatedAt;
        bool isActive;
    }
    
    mapping(address => address) public delegations;
    mapping(address => uint256) public delegatedPower;
    mapping(address => address[]) public delegateeDelegators;
    
    uint256 public constant MIN_DELEGATION_TIME = 7 days;
    
    event DelegationCreated(address indexed delegator, address indexed delegatee, uint256 amount);
    event DelegationRevoked(address indexed delegator, address indexed delegatee);
    
    function initialize() public initializer {
        __Ownable_init();
    }
    
    function delegate(address delegatee) external {
        require(delegatee != address(0), "Invalid delegatee");
        require(delegatee != msg.sender, "Cannot delegate to self");
        require(delegations[msg.sender] == address(0), "Already delegated");
        
        delegations[msg.sender] = delegatee;
        delegateeDelegators[delegatee].push(msg.sender);
        
        emit DelegationCreated(msg.sender, delegatee, 0);
    }
    
    function revokeDelegation() external {
        address delegatee = delegations[msg.sender];
        require(delegatee != address(0), "Not delegated");
        
        delete delegations[msg.sender];
        
        // Remove from delegatee's list
        address[] storage list = delegateeDelegators[delegatee];
        for (uint i = 0; i < list.length; i++) {
            if (list[i] == msg.sender) {
                list[i] = list[list.length - 1];
                list.pop();
                break;
            }
        }
        
        emit DelegationRevoked(msg.sender, delegatee);
    }
    
    function getDelegatee(address delegator) external view returns (address) {
        return delegations[delegator];
    }
    
    function getDelegators(address delegatee) external view returns (address[] memory) {
        return delegateeDelegators[delegatee];
    }
    
    function getDelegatedPower(address delegatee) external view returns (uint256) {
        return delegateeDelegators[delegatee].length;
    }
}
