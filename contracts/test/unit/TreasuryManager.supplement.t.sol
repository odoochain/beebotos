// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/dao/TreasuryManager.sol";
import "../../src/dao/BeeToken.sol";
import "../../src/interfaces/ITreasuryManager.sol";

/**
 * @title TreasuryManagerSupplementTest
 * @dev Additional tests for TreasuryManager to reach 90%+ coverage
 */
contract TreasuryManagerSupplementTest is Test {
    BeeToken public token;
    TreasuryManager public treasury;
    address public owner = address(1);
    address public beneficiary = address(2);
    address public budgetManager = address(3);
    address public recipient = address(4);
    
    function setUp() public {
        vm.startPrank(owner);
        
        token = new BeeToken(owner, address(2), address(3), address(4), address(5));
        treasury = new TreasuryManager(owner);
        treasury.grantRole(treasury.BUDGET_MANAGER(), budgetManager);
        
        token.transfer(address(treasury), 100000 * 10**18);
        vm.deal(address(treasury), 100 ether);
        
        vm.stopPrank();
    }
    
    // ============ Budget Management Tests ============
    
    function testDeactivateBudget() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        vm.prank(owner);
        treasury.deactivateBudget(budgetId);
        
        (,,,,,,, bool isActive) = treasury.budgets(budgetId);
        assertFalse(isActive);
    }
    
    function testDeactivateBudgetNotFound() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: budget not found");
        treasury.deactivateBudget(999);
    }
    
    function testDeactivateAlreadyDeactivated() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        vm.prank(owner);
        treasury.deactivateBudget(budgetId);
        
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: budget not active");
        treasury.deactivateBudget(budgetId);
    }
    
    function testGetBudgetBalance() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        uint256 balance = treasury.getBudgetBalance(budgetId);
        assertEq(balance, 50000 * 10**18);
        
        // Spend some
        vm.prank(budgetManager);
        treasury.spendFromBudget(budgetId, 10000 * 10**18, "Test spend");
        
        balance = treasury.getBudgetBalance(budgetId);
        assertEq(balance, 40000 * 10**18);
    }
    
    function testGetBudgetBalanceZero() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        vm.prank(budgetManager);
        treasury.spendFromBudget(budgetId, 50000 * 10**18, "Spend all");
        
        uint256 balance = treasury.getBudgetBalance(budgetId);
        assertEq(balance, 0);
    }
    
    function testGetBudgetBalanceNotFound() public {
        vm.expectRevert("TreasuryManager: budget not found");
        treasury.getBudgetBalance(999);
    }
    
    // ============ Budget Validation Tests ============
    
    function testCreateBudgetZeroBeneficiary() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: zero beneficiary");
        treasury.createBudget(
            address(0),
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
    }
    
    function testCreateBudgetZeroAmount() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: zero amount");
        treasury.createBudget(
            beneficiary,
            0,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
    }
    
    function testCreateBudgetInvalidTimeRange() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: invalid time range");
        treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp + 90 days,
            block.timestamp,
            ITreasuryManager.BudgetType.Development
        );
    }
    
    function testCreateBudgetStartInPast() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: start in past");
        treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp - 1,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
    }
    
    function testSpendFromBudgetEmptyReason() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        vm.prank(budgetManager);
        vm.expectRevert("TreasuryManager: empty reason");
        treasury.spendFromBudget(budgetId, 10000 * 10**18, "");
    }
    
    // ============ Stream Management Tests ============
    
    function testCancelStream() public {
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            30 days
        );
        
        vm.prank(owner);
        treasury.cancelStream(streamId);
        
        (,,,,,, bool isActive) = treasury.streams(streamId);
        assertFalse(isActive);
    }
    
    function testCancelStreamNotFound() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: stream not found");
        treasury.cancelStream(999);
    }
    
    function testCancelStreamNotActive() public {
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            30 days
        );
        
        vm.warp(block.timestamp + 31 days);
        
        vm.prank(beneficiary);
        treasury.releaseStream(streamId);
        
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: stream not active");
        treasury.cancelStream(streamId);
    }
    
    function testCalculateReleasable() public {
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            10 days
        );
        
        // Initially 0
        assertEq(treasury.calculateReleasable(streamId), 0);
        
        // Halfway
        vm.warp(block.timestamp + 5 days);
        assertApproxEqRel(treasury.calculateReleasable(streamId), 500 * 10**18, 0.01e18);
        
        // Full
        vm.warp(block.timestamp + 6 days);
        assertEq(treasury.calculateReleasable(streamId), 1000 * 10**18);
    }
    
    function testCalculateReleasableNotFound() public {
        vm.expectRevert("TreasuryManager: stream not found");
        treasury.calculateReleasable(999);
    }
    
    // ============ Stream Validation Tests ============
    
    function testCreateStreamZeroRecipient() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: zero recipient");
        treasury.createStreamingPayment(
            address(0),
            address(token),
            1000 * 10**18,
            30 days
        );
    }
    
    function testCreateStreamZeroAmount() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: zero amount");
        treasury.createStreamingPayment(
            beneficiary,
            address(token),
            0,
            30 days
        );
    }
    
    function testCreateStreamZeroDuration() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: zero duration");
        treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            0
        );
    }
    
    // ============ Release Stream Validation Tests ============
    
    function testReleaseStreamNotFound() public {
        vm.prank(beneficiary);
        vm.expectRevert("TreasuryManager: stream not found");
        treasury.releaseStream(999);
    }
    
    function testReleaseStreamNotActive() public {
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            10 days
        );
        
        vm.warp(block.timestamp + 11 days);
        vm.prank(beneficiary);
        treasury.releaseStream(streamId);
        
        vm.prank(beneficiary);
        vm.expectRevert("TreasuryManager: stream not active");
        treasury.releaseStream(streamId);
    }
    
    // ============ Pause/Unpause Tests ============
    
    function testPause() public {
        vm.prank(owner);
        treasury.pause();
        assertTrue(treasury.paused());
    }
    
    function testUnpause() public {
        vm.prank(owner);
        treasury.pause();
        
        vm.prank(owner);
        treasury.unpause();
        assertFalse(treasury.paused());
    }
    
    function testSpendWhenPaused() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        vm.prank(owner);
        treasury.pause();
        
        vm.prank(budgetManager);
        vm.expectRevert();
        treasury.spendFromBudget(budgetId, 10000 * 10**18, "Test");
    }
    
    function testReleaseStreamWhenPaused() public {
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            10 days
        );
        
        vm.warp(block.timestamp + 5 days);
        
        vm.prank(owner);
        treasury.pause();
        
        vm.prank(beneficiary);
        vm.expectRevert();
        treasury.releaseStream(streamId);
    }
    
    // ============ DAO Management Tests ============
    
    // Note: TreasuryManager does not have setDAO function
    // DAO address is immutable after construction
    
    // ============ Emergency Withdraw Tests ============
    
    function testEmergencyWithdrawToken() public {
        uint256 withdrawAmount = 10000 * 10**18;
        uint256 ownerBalanceBefore = token.balanceOf(owner);
        
        vm.prank(owner);
        treasury.emergencyWithdraw(address(token), owner, withdrawAmount);
        
        assertEq(token.balanceOf(owner), ownerBalanceBefore + withdrawAmount);
    }
    
    function testEmergencyWithdrawETH() public {
        uint256 withdrawAmount = 10 ether;
        uint256 ownerBalanceBefore = owner.balance;
        
        vm.prank(owner);
        treasury.emergencyWithdraw(address(0), owner, withdrawAmount);
        
        assertEq(owner.balance, ownerBalanceBefore + withdrawAmount);
    }
    
    function testEmergencyWithdrawZeroRecipient() public {
        vm.prank(owner);
        vm.expectRevert("TreasuryManager: zero recipient");
        treasury.emergencyWithdraw(address(token), address(0), 10000 * 10**18);
    }
    
    // ============ View Function Tests ============
    
    function testGetBudget() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        TreasuryManager.Budget memory budget = treasury.getBudget(budgetId);
        assertEq(budget.beneficiary, beneficiary);
        assertEq(budget.totalAllocation, 50000 * 10**18);
    }
    
    function testGetStream() public {
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            30 days
        );
        
        TreasuryManager.StreamingPayment memory stream = treasury.getStream(streamId);
        assertEq(stream.recipient, beneficiary);
        assertEq(stream.totalAmount, 1000 * 10**18);
    }
    
    function testIsBudgetActive() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        assertTrue(treasury.isBudgetActive(budgetId));
        
        vm.warp(block.timestamp + 91 days);
        assertFalse(treasury.isBudgetActive(budgetId));
    }
    
    function testGetTotalBudgetAllocation() public {
        vm.prank(owner);
        treasury.createBudget(
            beneficiary,
            10000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        vm.prank(owner);
        treasury.createBudget(
            address(4),
            20000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Marketing
        );
        
        assertEq(treasury.getTotalBudgetAllocation(), 30000 * 10**18);
    }
    
    function testGetTotalInStreams() public {
        vm.prank(owner);
        treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            30 days
        );
        
        vm.prank(owner);
        treasury.createStreamingPayment(
            address(4),
            address(token),
            2000 * 10**18,
            60 days
        );
        
        assertEq(treasury.getTotalInStreams(), 3000 * 10**18);
    }
    
    // ============ Budget Type Tests ============
    
    function testCreateBudgetAllTypes() public {
        ITreasuryManager.BudgetType[5] memory types = [
            ITreasuryManager.BudgetType.Operational,
            ITreasuryManager.BudgetType.Development,
            ITreasuryManager.BudgetType.Marketing,
            ITreasuryManager.BudgetType.Research,
            ITreasuryManager.BudgetType.Emergency
        ];
        
        for (uint i = 0; i < types.length; i++) {
            vm.prank(owner);
            uint256 budgetId = treasury.createBudget(
                beneficiary,
                10000 * 10**18,
                address(token),
                block.timestamp,
                block.timestamp + 90 days,
                types[i]
            );
            
            (,,,,,, ITreasuryManager.BudgetType budgetType,) = treasury.budgets(budgetId);
            assertEq(uint(budgetType), uint(types[i]));
        }
    }
    
    // ============ Events Tests ============
    
    function testCreateBudgetEmitsEvent() public {
        vm.prank(owner);
        vm.expectEmit(true, true, false, true);
        emit ITreasuryManager.BudgetCreated(1, beneficiary, 50000 * 10**18, ITreasuryManager.BudgetType.Development);
        treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
    }
    
    function testSpendFromBudgetEmitsEvent() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        vm.prank(budgetManager);
        vm.expectEmit(true, false, false, true);
        emit ITreasuryManager.BudgetSpent(budgetId, 10000 * 10**18, "Test spend", beneficiary);
        treasury.spendFromBudget(budgetId, 10000 * 10**18, "Test spend");
    }
    
    function testDeactivateBudgetEmitsEvent() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        vm.prank(budgetManager);
        treasury.spendFromBudget(budgetId, 10000 * 10**18, "Test");
        
        vm.prank(owner);
        vm.expectEmit(true, false, false, true);
        emit ITreasuryManager.BudgetDeactivated(budgetId, 40000 * 10**18);
        treasury.deactivateBudget(budgetId);
    }
    
    function testCreateStreamEmitsEvent() public {
        vm.prank(owner);
        vm.expectEmit(true, true, false, true);
        emit ITreasuryManager.StreamingPaymentCreated(1, beneficiary, 1000 * 10**18, 30 days, address(token));
        treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            30 days
        );
    }
    
    function testReleaseStreamEmitsEvent() public {
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            10 days
        );
        
        vm.warp(block.timestamp + 5 days);
        
        vm.prank(beneficiary);
        vm.expectEmit(true, false, false, true);
        emit ITreasuryManager.StreamReleased(streamId, 500 * 10**18, 500 * 10**18, 500 * 10**18);
        treasury.releaseStream(streamId);
    }
    
    function testCancelStreamEmitsEvent() public {
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            30 days
        );
        
        vm.prank(owner);
        vm.expectEmit(true, false, false, true);
        emit ITreasuryManager.StreamCanceled(streamId, 1000 * 10**18, beneficiary);
        treasury.cancelStream(streamId);
    }
    
    function testEmergencyWithdrawEmitsEvent() public {
        vm.prank(owner);
        vm.expectEmit(true, false, false, true);
        emit ITreasuryManager.FundsWithdrawn(address(token), 10000 * 10**18, owner);
        treasury.emergencyWithdraw(address(token), owner, 10000 * 10**18);
    }
    
    // ============ Gas Tests ============
    
    function testGas_CreateBudget() public {
        vm.prank(owner);
        uint256 gasBefore = gasleft();
        treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        uint256 gasUsed = gasBefore - gasleft();
        emit log_named_uint("Gas for createBudget", gasUsed);
    }
    
    function testGas_SpendFromBudget() public {
        vm.prank(owner);
        uint256 budgetId = treasury.createBudget(
            beneficiary,
            50000 * 10**18,
            address(token),
            block.timestamp,
            block.timestamp + 90 days,
            ITreasuryManager.BudgetType.Development
        );
        
        vm.prank(budgetManager);
        uint256 gasBefore = gasleft();
        treasury.spendFromBudget(budgetId, 10000 * 10**18, "Test spend");
        uint256 gasUsed = gasBefore - gasleft();
        emit log_named_uint("Gas for spendFromBudget", gasUsed);
    }
    
    function testGas_CreateStream() public {
        vm.prank(owner);
        uint256 gasBefore = gasleft();
        treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            30 days
        );
        uint256 gasUsed = gasBefore - gasleft();
        emit log_named_uint("Gas for createStreamingPayment", gasUsed);
    }
    
    function testGas_ReleaseStream() public {
        vm.prank(owner);
        uint256 streamId = treasury.createStreamingPayment(
            beneficiary,
            address(token),
            1000 * 10**18,
            10 days
        );
        
        vm.warp(block.timestamp + 5 days);
        
        vm.prank(beneficiary);
        uint256 gasBefore = gasleft();
        treasury.releaseStream(streamId);
        uint256 gasUsed = gasBefore - gasleft();
        emit log_named_uint("Gas for releaseStream", gasUsed);
    }
}




