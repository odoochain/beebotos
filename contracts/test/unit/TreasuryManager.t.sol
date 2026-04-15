// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/dao/TreasuryManager.sol";
import "../../src/dao/BeeToken.sol";
import "../../src/interfaces/ITreasuryManager.sol";

/**
 * @title TreasuryManagerTest
 * @dev Comprehensive tests for TreasuryManager with budget and streaming
 */
contract TreasuryManagerTest is Test {
    BeeToken public token;
    TreasuryManager public treasury;
    address public owner = address(1);
    address public beneficiary = address(2);
    address public budgetManager = address(3);
    
    function setUp() public {
        vm.startPrank(owner);
        
        // Deploy token
        token = new BeeToken(
            owner, address(2), address(3), address(4), address(5)
        );
        
        // Deploy treasury with DAO as owner
        treasury = new TreasuryManager(address(owner));
        
        // Grant budget manager role
        treasury.grantRole(treasury.BUDGET_MANAGER(), budgetManager);
        
        // Fund treasury with tokens
        token.transfer(address(treasury), 100000 * 10**18);
        
        // Fund treasury with ETH
        vm.deal(address(treasury), 100 ether);
        
        vm.stopPrank();
    }
    
    // ============ Budget Tests ============
    
    function testCreateBudget() public {
        uint256 startTime = block.timestamp;
        uint256 endTime = startTime + 90 days;
        uint256 amount = 50000 * 10**18;
        
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            amount,
            address(token),
            startTime,
            endTime,
            ITreasuryManager.BudgetType.Development
        );
        
        assertEq(budgetId, 1);
        
        // Verify budget details
        (
            uint256 totalAllocation,
            uint256 spent,
            uint256 budgetStart,
            uint256 budgetEnd,
            address budgetBeneficiary,
            address budgetToken,
            ITreasuryManager.BudgetType budgetType,
            bool isActive
        ) = treasury.budgets(budgetId);
        
        assertEq(totalAllocation, amount);
        assertEq(spent, 0);
        assertEq(budgetStart, startTime);
        assertEq(budgetEnd, endTime);
        assertEq(budgetBeneficiary, beneficiary);
        assertEq(budgetToken, address(token));
        assertEq(uint256(budgetType), uint256(ITreasuryManager.BudgetType.Development));
        assertTrue(isActive);
    }
    
    function testCreateMultipleBudgets() public {
        vm.startPrank(owner);
        
        uint256 budget1 = treasury.createBudget(
            beneficiary,
            10000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        uint256 budget2 = treasury.createBudget(
            address(4),
            20000 * 10**18,
            address(0), // ETH
            block.timestamp,
            block.timestamp + 60 days,
            ITreasuryManager.BudgetType.Marketing
        );
        
        assertEq(budget1, 1);
        assertEq(budget2, 2);
        
        vm.stopPrank();
    }
    
    function testSpendFromBudget() public {
        // Setup: Create budget
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        // Spend from budget
        uint256 spendAmount = 10000 * 10**18;
        vm.prank(budgetManager);
        treasury.spendFromBudget(budgetId, spendAmount, "Development milestone 1");
        
        // Verify spend recorded
        (, uint256 spent,,,,,,) = treasury.budgets(budgetId);
        assertEq(spent, spendAmount);
        
        // Verify beneficiary received tokens
        assertEq(token.balanceOf(beneficiary), spendAmount);
    }
    
    function testSpendFromBudgetRevertsWhenOverBudget() public {
        // Setup: Create budget
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        // Try to spend more than budget
        vm.prank(budgetManager);
        vm.expectRevert("Over budget");
        treasury.spendFromBudget(budgetId, 60000 * 10**18, "Over budget spend");
    }
    
    function testSpendFromBudgetRevertsWhenExpired() public {
        // Setup: Create budget
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        // Warp past end time
        vm.warp(block.timestamp + 91 days);
        
        // Try to spend after expiration
        vm.prank(budgetManager);
        vm.expectRevert("Expired");
        treasury.spendFromBudget(budgetId, 10000 * 10**18, "Late spend");
    }
    
    function testSpendFromBudgetRevertsWhenNotStarted() public {
        // Setup: Create budget that starts in the future
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp + 30 days,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        // Try to spend before start
        vm.prank(budgetManager);
        vm.expectRevert("Not started");
        treasury.spendFromBudget(budgetId, 10000 * 10**18, "Early spend");
    }
    
    function testSpendFromBudgetRevertsWhenInactive() public {
        // Setup and deactivate budget (would need deactivate function)
        // This test demonstrates the access control check
        
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        // Non-budget manager should not be able to spend
        vm.prank(address(0xdead));
        vm.expectRevert(
            abi.encodeWithSelector(
                bytes4(keccak256("AccessControlUnauthorizedAccount(address,bytes32)")),
                address(0xdead),
                treasury.BUDGET_MANAGER()
            )
        );
        treasury.spendFromBudget(budgetId, 10000 * 10**18, "Unauthorized");
    }
    
    // ============ Streaming Payment Tests ============
    
    function testCreateStream() public {
        uint256 streamAmount = 1000 * 10**18;
        uint256 duration = 30 days;
        
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            streamAmount,
            duration
        );
        
        assertEq(streamId, 1);
        
        // Verify stream details
        (
            address recipient,
            address streamToken,
            uint256 totalAmount,
            uint256 releasedAmount,
            uint256 startTime,
            uint256 streamDuration,
            bool isActive
        ) = treasury.streams(streamId);
        
        assertEq(recipient, beneficiary);
        assertEq(streamToken, address(token));
        assertEq(totalAmount, streamAmount);
        assertEq(releasedAmount, 0);
        assertEq(startTime, block.timestamp);
        assertEq(streamDuration, duration);
        assertTrue(isActive);
    }
    
    function testCreateStreamWithETH() public {
        uint256 streamAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(0), // ETH
            streamAmount,
            duration
        );
        
        (
            address recipient,
            address streamToken,
            uint256 totalAmount,
            ,,,bool isActive
        ) = treasury.streams(streamId);
        
        assertEq(recipient, beneficiary);
        assertEq(streamToken, address(0));
        assertEq(totalAmount, streamAmount);
        assertTrue(isActive);
    }
    
    function testReleaseStream() public {
        // Setup
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            10 days
        );
        
        // Warp halfway through
        vm.warp(block.timestamp + 5 days);
        
        // Release
        vm.prank(beneficiary);
        uint256 released = treasury.releaseStream(streamId);
        
        // Should release ~50%
        assertApproxEqRel(released, 500 * 10**18, 0.01e18); // ~50% with 1% tolerance
        assertGt(released, 0);
        
        // Verify recipient received tokens
        assertEq(token.balanceOf(beneficiary), released);
        
        // Verify stream updated
        (,,, uint256 releasedAmount,,,) = treasury.streams(streamId);
        assertEq(releasedAmount, released);
    }
    
    function testReleaseStreamFully() public {
        // Setup
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            10 days
        );
        
        // Warp past end
        vm.warp(block.timestamp + 11 days);
        
        // Release
        vm.prank(beneficiary);
        uint256 released = treasury.releaseStream(streamId);
        
        // Should release 100%
        assertEq(released, 1000 * 10**18);
        
        // Verify stream is now inactive
        (,,,,,,bool isActive) = treasury.streams(streamId);
        assertFalse(isActive);
    }
    
    function testReleaseStreamRevertsWhenNothingToRelease() public {
        // Setup
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            10 days
        );
        
        // Try to release immediately
        vm.prank(beneficiary);
        vm.expectRevert("Nothing to release");
        treasury.releaseStream(streamId);
    }
    
    function testMultipleReleases() public {
        // Setup
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            10 days
        );
        
        // First release at 30%
        vm.warp(block.timestamp + 3 days);
        vm.prank(beneficiary);
        uint256 release1 = treasury.releaseStream(streamId);
        
        // Second release at 60%
        vm.warp(block.timestamp + 3 days);
        vm.prank(beneficiary);
        uint256 release2 = treasury.releaseStream(streamId);
        
        // Third release at 100%
        vm.warp(block.timestamp + 5 days);
        vm.prank(beneficiary);
        uint256 release3 = treasury.releaseStream(streamId);
        
        // Total should be ~1000 tokens
        uint256 totalReleased = release1 + release2 + release3;
        assertApproxEqRel(totalReleased, 1000 * 10**18, 0.01e18);
    }
    
    // ============ Access Control Tests ============
    
    function testOnlyAdminCanCreateBudget() public {
        vm.prank(budgetManager);
        vm.expectRevert(
            abi.encodeWithSelector(
                bytes4(keccak256("AccessControlUnauthorizedAccount(address,bytes32)")),
                budgetManager,
                treasury.TREASURY_ADMIN()
            )
        );
        treasury.createBudget(
            beneficiary,
            10000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
    }
    
    function testOnlyAdminCanCreateStream() public {
        vm.prank(budgetManager);
        vm.expectRevert(
            abi.encodeWithSelector(
                bytes4(keccak256("AccessControlUnauthorizedAccount(address,bytes32)")),
                budgetManager,
                treasury.TREASURY_ADMIN()
            )
        );
        treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            30 days
        );
    }
    
    // ============ ETH Handling Tests ============
    
    function testReceiveETH() public {
        uint256 initialBalance = address(treasury).balance;
        
        vm.deal(address(this), 10 ether);
        (bool success, ) = address(treasury).call{value: 10 ether}("");
        assertTrue(success);
        
        assertEq(address(treasury).balance, initialBalance + 10 ether);
    }
}

