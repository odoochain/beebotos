// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

/**
 * @title BudgetController
 * @notice Manages DAO budget categories and spending limits
 */
contract BudgetController is OwnableUpgradeable {
    
    enum Category { Operations, Development, Marketing, Research, Grants, Reserve }
    
    struct Budget {
        uint256 allocated;
        uint256 spent;
        uint256 remaining;
        uint256 periodStart;
        uint256 periodEnd;
    }
    
    mapping(Category => Budget) public budgets;
    mapping(bytes32 => bool) public approvedExpenses;
    
    uint256 public constant BUDGET_PERIOD = 90 days;
    
    event BudgetAllocated(Category indexed category, uint256 amount);
    event ExpenseApproved(bytes32 indexed expenseId, Category category, uint256 amount);
    event ExpenseExecuted(bytes32 indexed expenseId, uint256 amount);
    
    function initialize() public initializer {
        __Ownable_init();
    }
    
    function allocateBudget(Category category, uint256 amount) external onlyOwner {
        budgets[category] = Budget({
            allocated: amount,
            spent: 0,
            remaining: amount,
            periodStart: block.timestamp,
            periodEnd: block.timestamp + BUDGET_PERIOD
        });
        
        emit BudgetAllocated(category, amount);
    }
    
    function approveExpense(
        bytes32 expenseId,
        Category category,
        uint256 amount
    ) external onlyOwner {
        Budget storage b = budgets[category];
        require(b.remaining >= amount, "Insufficient budget");
        require(block.timestamp <= b.periodEnd, "Budget expired");
        
        approvedExpenses[expenseId] = true;
        
        emit ExpenseApproved(expenseId, category, amount);
    }
    
    function executeExpense(
        bytes32 expenseId,
        Category category,
        uint256 amount
    ) external onlyOwner {
        require(approvedExpenses[expenseId], "Not approved");
        
        Budget storage b = budgets[category];
        b.spent += amount;
        b.remaining -= amount;
        
        approvedExpenses[expenseId] = false;
        
        emit ExpenseExecuted(expenseId, amount);
    }
    
    function getBudgetStatus(Category category) external view returns (
        uint256 allocated,
        uint256 spent,
        uint256 remaining,
        uint256 daysRemaining
    ) {
        Budget storage b = budgets[category];
        uint256 remainingTime = b.periodEnd > block.timestamp ? b.periodEnd - block.timestamp : 0;
        
        return (
            b.allocated,
            b.spent,
            b.remaining,
            remainingTime / 1 days
        );
    }
}
