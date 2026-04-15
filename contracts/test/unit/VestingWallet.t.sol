// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/dao/VestingWallet.sol";
import "../../src/dao/BeeToken.sol";

/**
 * @title VestingWalletTest
 * @dev Comprehensive tests for VestingWallet
 */
contract VestingWalletTest is Test {
    BeeToken public token;
    VestingWallet public vesting;
    
    address public owner = address(1);
    address public beneficiary1 = address(2);
    address public beneficiary2 = address(3);
    
    uint256 constant INITIAL_SUPPLY = 1_000_000e18;
    
    function setUp() public {
        vm.startPrank(owner);
        
        // Deploy token
        token = new BeeToken(
            owner, address(4), address(5), address(6), address(7)
        );
        
        // Deploy vesting contract
        vesting = new VestingWallet(address(token));
        
        // Fund vesting contract
        token.transfer(address(vesting), INITIAL_SUPPLY);
        
        vm.stopPrank();
    }
    
    // ============ Schedule Creation Tests ============
    
    function testCreateVestingSchedule() public {
        uint256 amount = 100_000e18;
        uint256 startTime = block.timestamp + 30 days;
        uint256 cliffDuration = 180 days;
        uint256 vestingDuration = 730 days; // 2 years
        
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            amount,
            startTime,
            cliffDuration,
            vestingDuration,
            true // revocable
        );
        
        assertEq(scheduleId, 0);
        
        // Verify schedule details
        VestingWallet.VestingSchedule memory schedule = vesting.getSchedule(scheduleId);
        assertEq(schedule.beneficiary, beneficiary1);
        assertEq(schedule.totalAmount, amount);
        assertEq(schedule.startTime, startTime);
        assertEq(schedule.cliffDuration, cliffDuration);
        assertEq(schedule.vestingDuration, vestingDuration);
        assertEq(schedule.releasedAmount, 0);
        assertTrue(schedule.revocable);
        assertFalse(schedule.revoked);
        
        // Verify beneficiary tracking
        uint256[] memory schedules = vesting.getBeneficiarySchedules(beneficiary1);
        assertEq(schedules.length, 1);
        assertEq(schedules[0], scheduleId);
    }
    
    function testCreateMultipleSchedules() public {
        vm.startPrank(owner);
        
        uint256 schedule1 = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        uint256 schedule2 = vesting.createVestingSchedule(
            beneficiary1,
            50_000e18,
            block.timestamp,
            90 days,
            365 days,
            false
        );
        
        uint256 schedule3 = vesting.createVestingSchedule(
            beneficiary2,
            200_000e18,
            block.timestamp,
            365 days,
            1460 days,
            true
        );
        
        assertEq(schedule1, 0);
        assertEq(schedule2, 1);
        assertEq(schedule3, 2);
        
        assertEq(vesting.totalVested(beneficiary1), 150_000e18);
        assertEq(vesting.totalVested(beneficiary2), 200_000e18);
        
        vm.stopPrank();
    }
    
    function testCreateScheduleRevertsOnZeroBeneficiary() public {
        vm.prank(owner);
        vm.expectRevert("VestingWallet: zero beneficiary");
        vesting.createVestingSchedule(
            address(0),
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
    }
    
    function testCreateScheduleRevertsOnZeroAmount() public {
        vm.prank(owner);
        vm.expectRevert("VestingWallet: zero amount");
        vesting.createVestingSchedule(
            beneficiary1,
            0,
            block.timestamp,
            180 days,
            730 days,
            true
        );
    }
    
    function testCreateScheduleRevertsOnPastStart() public {
        vm.prank(owner);
        vm.expectRevert("VestingWallet: start in past");
        vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp - 1,
            180 days,
            730 days,
            true
        );
    }
    
    function testCreateScheduleRevertsOnCliffExceedsDuration() public {
        vm.prank(owner);
        vm.expectRevert("VestingWallet: cliff > duration");
        vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            730 days,
            180 days, // Less than cliff
            true
        );
    }
    
    // ============ Vesting Calculation Tests ============
    
    function testVestingBeforeCliff() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        // Before cliff
        vm.warp(block.timestamp + 90 days);
        assertEq(vesting.vestedAmount(scheduleId), 0);
        assertEq(vesting.releasableAmount(scheduleId), 0);
        assertFalse(vesting.isCliffReached(scheduleId));
    }
    
    function testVestingAtCliff() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        // At cliff - still 0 (linear vesting starts after cliff)
        vm.warp(block.timestamp + 180 days);
        assertEq(vesting.vestedAmount(scheduleId), 0);
        assertEq(vesting.releasableAmount(scheduleId), 0);
        assertTrue(vesting.isCliffReached(scheduleId));
    }
    
    function testVestingAfterCliff() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        // Halfway through vesting period (180 + 275 days = 455 days)
        vm.warp(block.timestamp + 455 days);
        
        uint256 vested = vesting.vestedAmount(scheduleId);
        // Should be ~50% vested (275 / 550 days after cliff)
        assertApproxEqRel(vested, 50_000e18, 0.02e18); // 2% tolerance
        assertGt(vested, 0);
        assertLt(vested, 100_000e18);
    }
    
    function testFullVesting() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        // After full vesting
        vm.warp(block.timestamp + 730 days);
        
        assertEq(vesting.vestedAmount(scheduleId), 100_000e18);
        assertEq(vesting.releasableAmount(scheduleId), 100_000e18);
        assertTrue(vesting.isVestingComplete(scheduleId));
    }
    
    // ============ Release Tests ============
    
    function testRelease() public {
        // Setup schedule
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        // Fast forward to full vesting
        vm.warp(block.timestamp + 730 days);
        
        uint256 balanceBefore = token.balanceOf(beneficiary1);
        
        vm.prank(beneficiary1);
        vesting.release(scheduleId);
        
        uint256 balanceAfter = token.balanceOf(beneficiary1);
        assertEq(balanceAfter - balanceBefore, 100_000e18);
        
        // Verify schedule updated
        VestingWallet.VestingSchedule memory schedule = vesting.getSchedule(scheduleId);
        assertEq(schedule.releasedAmount, 100_000e18);
    }
    
    function testReleasePartial() public {
        // Setup schedule
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        // Release at 50% vested
        vm.warp(block.timestamp + 455 days);
        
        vm.prank(beneficiary1);
        vesting.release(scheduleId);
        
        uint256 released1 = vesting.totalReleased(beneficiary1);
        assertApproxEqRel(released1, 50_000e18, 0.02e18);
        
        // Fast forward to full vesting and release again
        vm.warp(block.timestamp + 275 days);
        
        vm.prank(beneficiary1);
        vesting.release(scheduleId);
        
        uint256 totalReleased = vesting.totalReleased(beneficiary1);
        assertEq(totalReleased, 100_000e18);
    }
    
    function testReleaseRevertsIfNoTokens() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        // Before any tokens vested
        vm.prank(beneficiary1);
        vm.expectRevert("VestingWallet: no tokens to release");
        vesting.release(scheduleId);
    }
    
    function testReleaseRevertsIfNotBeneficiary() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        vm.warp(block.timestamp + 730 days);
        
        vm.prank(beneficiary2);
        vm.expectRevert("VestingWallet: not beneficiary");
        vesting.release(scheduleId);
    }
    
    function testReleaseAll() public {
        // Setup multiple schedules
        vm.startPrank(owner);
        vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        vesting.createVestingSchedule(
            beneficiary1,
            50_000e18,
            block.timestamp,
            90 days,
            365 days,
            true
        );
        vm.stopPrank();
        
        // Fast forward
        vm.warp(block.timestamp + 730 days);
        
        uint256 balanceBefore = token.balanceOf(beneficiary1);
        
        vm.prank(beneficiary1);
        vesting.releaseAll();
        
        uint256 balanceAfter = token.balanceOf(beneficiary1);
        assertEq(balanceAfter - balanceBefore, 150_000e18);
    }
    
    function testTotalReleasable() public {
        vm.startPrank(owner);
        vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        vm.stopPrank();
        
        vm.warp(block.timestamp + 455 days);
        
        uint256 totalReleasable = vesting.totalReleasable(beneficiary1);
        // Should be ~50% of 200k = 100k
        assertApproxEqRel(totalReleasable, 100_000e18, 0.02e18);
    }
    
    // ============ Revocation Tests ============
    
    function testRevoke() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true // revocable
        );
        
        // Fast forward to partial vesting
        vm.warp(block.timestamp + 455 days);
        
        uint256 vestedBefore = vesting.vestedAmount(scheduleId);
        
        vm.prank(owner);
        vesting.revoke(scheduleId);
        
        // Verify schedule revoked
        VestingWallet.VestingSchedule memory schedule = vesting.getSchedule(scheduleId);
        assertTrue(schedule.revoked);
        
        // Beneficiary should have received vested amount
        assertEq(token.balanceOf(beneficiary1), vestedBefore);
    }
    
    function testRevokeRevertsIfNotRevocable() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            false // not revocable
        );
        
        vm.prank(owner);
        vm.expectRevert("VestingWallet: not revocable");
        vesting.revoke(scheduleId);
    }
    
    function testRevokeRevertsIfAlreadyRevoked() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        vm.startPrank(owner);
        vesting.revoke(scheduleId);
        
        vm.expectRevert("VestingWallet: already revoked");
        vesting.revoke(scheduleId);
        vm.stopPrank();
    }
    
    function testRevokeRevertsIfNotOwner() public {
        vm.prank(owner);
        uint256 scheduleId = vesting.createVestingSchedule(
            beneficiary1,
            100_000e18,
            block.timestamp,
            180 days,
            730 days,
            true
        );
        
        vm.prank(beneficiary1);
        vm.expectRevert();
        vesting.revoke(scheduleId);
    }
    
    // ============ Recovery Tests ============
    
    function testRecoverERC20() public {
        // Deploy another token
        vm.startPrank(owner);
        BeeToken otherToken = new BeeToken(
            owner, address(2), address(3), address(4), address(5)
        );
        otherToken.transfer(address(vesting), 1000e18);
        
        uint256 balanceBefore = otherToken.balanceOf(owner);
        
        vesting.recoverERC20(address(otherToken), 1000e18);
        
        uint256 balanceAfter = otherToken.balanceOf(owner);
        assertEq(balanceAfter - balanceBefore, 1000e18);
        vm.stopPrank();
    }
    
    function testRecoverERC20RevertsForVestingToken() public {
        vm.prank(owner);
        vm.expectRevert("VestingWallet: cannot recover vesting token");
        vesting.recoverERC20(address(token), 1000e18);
    }
    
    function testRecoverETH() public {
        // Send ETH to contract
        vm.deal(address(vesting), 10 ether);
        
        uint256 balanceBefore = owner.balance;
        
        vm.prank(owner);
        vesting.recoverETH();
        
        uint256 balanceAfter = owner.balance;
        assertEq(balanceAfter - balanceBefore, 10 ether);
    }
    
    // ============ Deposit Tests ============
    
    function testDeposit() public {
        uint256 amount = 500_000e18;
        
        vm.startPrank(owner);
        token.approve(address(vesting), amount);
        vesting.deposit(amount);
        vm.stopPrank();
        
        assertEq(token.balanceOf(address(vesting)), INITIAL_SUPPLY + amount);
    }
}
