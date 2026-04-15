// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/dao/BeeToken.sol";

/**
 * @title BeeTokenTest
 * @dev Comprehensive tests for BeeToken with locking functionality
 */
contract BeeTokenTest is Test {
    BeeToken public token;
    address public owner = address(1);
    address public user = address(2);
    address public team = address(3);
    address public investors = address(4);
    address public ecosystem = address(5);
    address public liquidity = address(6);
    
    function setUp() public {
        vm.startPrank(owner);
        token = new BeeToken(
            owner,      // treasury
            team,
            investors,
            ecosystem,
            liquidity
        );
        vm.stopPrank();
    }
    
    // ============ Basic Token Tests ============
    
    function testInitialAllocation() public view {
        assertEq(token.balanceOf(owner), 400_000_000e18);      // Treasury
        assertEq(token.balanceOf(team), 200_000_000e18);       // Team
        assertEq(token.balanceOf(investors), 150_000_000e18);  // Investors
        assertEq(token.balanceOf(ecosystem), 200_000_000e18);  // Ecosystem
        assertEq(token.balanceOf(liquidity), 50_000_000e18);   // Liquidity
        assertEq(token.totalMinted(), 1_000_000_000e18);
    }
    
    function testMint() public {
        vm.prank(owner);
        token.mint(user, 1000 * 10**18, "Test mint");
        
        assertEq(token.balanceOf(user), 1000 * 10**18);
    }
    
    function testBurn() public {
        // First send some tokens to user
        vm.prank(owner);
        token.transfer(user, 1000 * 10**18);
        
        vm.prank(owner);
        token.grantRole(token.BURNER_ROLE(), user);
        
        vm.prank(user);
        token.burn(500 * 10**18, "Test burn");
        
        assertEq(token.balanceOf(user), 500 * 10**18);
    }
    
    // ============ Voting Power Tests ============
    
    function testDelegate() public {
        vm.prank(owner);
        token.transfer(user, 1000 * 10**18);
        
        vm.prank(user);
        token.delegate(user);
        
        assertEq(token.delegates(user), user);
    }
    
    function testVotingPower() public {
        vm.prank(owner);
        token.transfer(user, 1000 * 10**18);
        
        vm.prank(user);
        token.delegate(user);
        
        uint256 votes = token.getVotes(user);
        assertEq(votes, 1000 * 10**18);
    }
    
    // ============ Token Lock Tests ============
    
    function testLock() public {
        // Transfer tokens to user first
        vm.prank(owner);
        token.transfer(user, 1000 * 10**18);
        
        vm.startPrank(user);
        uint256 lockAmount = 500 * 10**18;
        uint256 duration = 365 days;
        
        uint256 lockId = token.lock(lockAmount, duration);
        
        // Check lock was created
        assertEq(lockId, 0);
        assertEq(token.balanceOf(user), 500 * 10**18); // Remaining balance
        assertEq(token.totalLocked(user), lockAmount);
        
        // Check lock details
        BeeToken.LockInfo memory lockInfo = token.getLock(user, lockId);
        assertEq(lockInfo.amount, lockAmount);
        assertEq(lockInfo.endTime, block.timestamp + duration);
        assertFalse(lockInfo.released);
        vm.stopPrank();
    }
    
    function testLockRevertsOnInsufficientBalance() public {
        vm.prank(user);
        vm.expectRevert("BeeToken: insufficient balance");
        token.lock(1000 * 10**18, 365 days);
    }
    
    function testUnlock() public {
        // Setup: transfer and lock
        vm.prank(owner);
        token.transfer(user, 1000 * 10**18);
        
        vm.prank(user);
        uint256 lockId = token.lock(500 * 10**18, 365 days);
        
        // Fast forward past lock duration
        vm.warp(block.timestamp + 366 days);
        
        // Unlock
        vm.prank(user);
        token.unlock(lockId);
        
        // Verify tokens returned
        assertEq(token.balanceOf(user), 1000 * 10**18);
        assertEq(token.totalLocked(user), 0);
        
        // Verify lock is marked as released
        BeeToken.LockInfo memory lockInfo = token.getLock(user, lockId);
        assertTrue(lockInfo.released);
    }
    
    function testUnlockRevertsIfNotExpired() public {
        // Setup: transfer and lock
        vm.prank(owner);
        token.transfer(user, 1000 * 10**18);
        
        vm.prank(user);
        uint256 lockId = token.lock(500 * 10**18, 365 days);
        
        // Try to unlock before expiration
        vm.prank(user);
        vm.expectRevert("BeeToken: lock not expired");
        token.unlock(lockId);
    }
    
    function testExtendLock() public {
        // Setup: transfer and lock
        vm.prank(owner);
        token.transfer(user, 1000 * 10**18);
        
        vm.prank(user);
        uint256 lockId = token.lock(500 * 10**18, 365 days);
        
        uint256 originalEndTime = block.timestamp + 365 days;
        
        // Extend lock
        vm.prank(user);
        token.extendLock(lockId, 180 days);
        
        // Verify extension
        BeeToken.LockInfo memory lockInfo = token.getLock(user, lockId);
        assertEq(lockInfo.endTime, originalEndTime + 180 days);
    }
    
    function testMultipleLocks() public {
        // Setup
        vm.prank(owner);
        token.transfer(user, 3000 * 10**18);
        
        vm.startPrank(user);
        
        // Create multiple locks
        uint256 lock1 = token.lock(500 * 10**18, 365 days);
        uint256 lock2 = token.lock(1000 * 10**18, 730 days);
        uint256 lock3 = token.lock(500 * 10**18, 180 days);
        
        assertEq(lock1, 0);
        assertEq(lock2, 1);
        assertEq(lock3, 2);
        
        // Check total locked
        assertEq(token.totalLocked(user), 2000 * 10**18);
        assertEq(token.balanceOf(user), 1000 * 10**18);
        
        // Get all locks
        BeeToken.LockInfo[] memory locks = token.getLocks(user);
        assertEq(locks.length, 3);
        
        vm.stopPrank();
    }
    
    function testGetActiveLocks() public {
        // Setup
        vm.prank(owner);
        token.transfer(user, 2000 * 10**18);
        
        vm.startPrank(user);
        token.lock(500 * 10**18, 365 days);
        token.lock(500 * 10**18, 180 days);
        vm.stopPrank();
        
        // Get active locks
        BeeToken.LockInfo[] memory activeLocks = token.getActiveLocks(user);
        assertEq(activeLocks.length, 2);
        
        // Fast forward and unlock first lock
        vm.warp(block.timestamp + 200 days);
        vm.prank(user);
        token.unlock(1); // Unlock the 180-day lock
        
        // Check active locks again
        activeLocks = token.getActiveLocks(user);
        assertEq(activeLocks.length, 1);
    }
    
    function testLockedTokensRetainVotingPower() public {
        // Setup
        vm.prank(owner);
        token.transfer(user, 1000 * 10**18);
        
        vm.startPrank(user);
        token.delegate(user);
        token.lock(500 * 10**18, 365 days);
        vm.stopPrank();
        
        // Check voting power includes locked tokens
        uint256 votingPower = token.getVotesWithLocked(user);
        assertEq(votingPower, 1000 * 10**18);
        
        // Regular votes should only include unlocked balance
        uint256 regularVotes = token.getVotes(user);
        assertEq(regularVotes, 500 * 10**18);
    }
    
    function testLockEventEmission() public {
        vm.prank(owner);
        token.transfer(user, 1000 * 10**18);
        
        vm.prank(user);
        
        vm.expectEmit(true, true, false, true);
        emit BeeToken.TokensLocked(user, 0, 500 * 10**18, block.timestamp + 365 days);
        
        token.lock(500 * 10**18, 365 days);
    }
    
    // ============ Access Control Tests ============
    
    function testOnlyMinterCanMint() public {
        vm.prank(user);
        vm.expectRevert(
            abi.encodeWithSelector(
                bytes4(keccak256("AccessControlUnauthorizedAccount(address,bytes32)")),
                user,
                token.MINTER_ROLE()
            )
        );
        token.mint(user, 1000 * 10**18, "Unauthorized");
    }
    
    function testOnlyBurnerCanBurn() public {
        vm.prank(user);
        vm.expectRevert(
            abi.encodeWithSelector(
                bytes4(keccak256("AccessControlUnauthorizedAccount(address,bytes32)")),
                user,
                token.BURNER_ROLE()
            )
        );
        token.burn(1000 * 10**18, "Unauthorized");
    }
}
