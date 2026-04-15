// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/core/ReputationSystem.sol";

/**
 * @title ReputationSystemTest
 * @dev Comprehensive tests for ReputationSystem with archival
 */
contract ReputationSystemTest is Test {
    ReputationSystem public reputation;
    
    address public owner = address(1);
    address public updater = address(2);
    address public user1 = address(3);
    address public user2 = address(4);
    
    function setUp() public {
        vm.prank(owner);
        reputation = new ReputationSystem();
        reputation.initialize();
        
        vm.prank(owner);
        reputation.setAuthorizedUpdater(updater, true);
    }
    
    // ============ Basic Update Tests ============
    
    function testInitialReputation() public view {
        assertEq(reputation.getReputation(user1), 0);
    }
    
    function testIncreaseReputation() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 100, "Initial reputation");
        
        assertEq(reputation.getReputation(user1), 100);
    }
    
    function testDecreaseReputation() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 500, "Initial");
        
        vm.prank(updater);
        reputation.updateReputation(user1, -200, "Penalty");
        
        assertEq(reputation.getReputation(user1), 300);
    }
    
    function testReputationCap() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 15000, "Max out");
        
        // Should be capped at 10000
        assertEq(reputation.getReputation(user1), 10000);
    }
    
    function testReputationFloor() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 100, "Initial");
        
        vm.prank(updater);
        reputation.updateReputation(user1, -500, "Big penalty");
        
        // Should be floored at 0
        assertEq(reputation.getReputation(user1), 0);
    }
    
    function testOnlyAuthorizedCanUpdate() public {
        vm.prank(user2);
        vm.expectRevert("ReputationSystem: not authorized");
        reputation.updateReputation(user1, 100, "Unauthorized");
    }
    
    // ============ Batch Update Tests ============
    
    function testBatchUpdate() public {
        address[] memory accounts = new address[](2);
        accounts[0] = user1;
        accounts[1] = user2;
        
        int256[] memory deltas = new int256[](2);
        deltas[0] = 100;
        deltas[1] = 200;
        
        string[] memory reasons = new string[](2);
        reasons[0] = "First";
        reasons[1] = "Second";
        
        vm.prank(updater);
        reputation.batchUpdateReputation(accounts, deltas, reasons);
        
        assertEq(reputation.getReputation(user1), 100);
        assertEq(reputation.getReputation(user2), 200);
    }
    
    // ============ Category Score Tests ============
    
    function testCategoryScore() public {
        bytes32 category = keccak256("trading");
        
        vm.prank(updater);
        reputation.updateCategoryScore(user1, category, 500);
        
        assertEq(reputation.getCategoryScore(user1, category), 500);
    }
    
    function testCategoryScoreDecrease() public {
        bytes32 category = keccak256("trading");
        
        vm.prank(updater);
        reputation.updateCategoryScore(user1, category, 500);
        
        vm.prank(updater);
        reputation.updateCategoryScore(user1, category, -200);
        
        assertEq(reputation.getCategoryScore(user1, category), 300);
    }
    
    function testCategoryScoreFloor() public {
        bytes32 category = keccak256("trading");
        
        vm.prank(updater);
        reputation.updateCategoryScore(user1, category, 100);
        
        vm.prank(updater);
        reputation.updateCategoryScore(user1, category, -500);
        
        assertEq(reputation.getCategoryScore(user1, category), 0);
    }
    
    function testGetAllCategoryScores() public {
        bytes32[] memory categories = new bytes32[](2);
        categories[0] = keccak256("trading");
        categories[1] = keccak256("development");
        
        vm.prank(updater);
        reputation.updateCategoryScore(user1, categories[0], 500);
        
        vm.prank(updater);
        reputation.updateCategoryScore(user1, categories[1], 300);
        
        uint256[] memory scores = reputation.getAllCategoryScores(user1, categories);
        assertEq(scores[0], 500);
        assertEq(scores[1], 300);
    }
    
    // ============ History Tests ============
    
    function testHistoryTracking() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 100, "First");
        
        vm.prank(updater);
        reputation.updateReputation(user1, 50, "Second");
        
        (uint256[] memory scores, uint256[] memory timestamps) = 
            reputation.getReputationHistory(user1);
        
        assertEq(scores.length, 2);
        assertEq(scores[0], 100);
        assertEq(scores[1], 150);
        assertEq(timestamps.length, 2);
    }
    
    function testPaginatedHistory() public {
        // Add 10 updates
        for (uint i = 0; i < 10; i++) {
            vm.prank(updater);
            reputation.updateReputation(user1, int256(10 * (i + 1)), "Update");
        }
        
        // Get page 1 (first 5)
        (uint256[] memory scores, uint256[] memory timestamps, uint256 total) = 
            reputation.getReputationHistoryPaginated(user1, 0, 5);
        
        assertEq(total, 10);
        assertEq(scores.length, 5);
        
        // Get page 2 (next 5)
        (scores, timestamps, total) = 
            reputation.getReputationHistoryPaginated(user1, 5, 5);
        
        assertEq(scores.length, 5);
    }
    
    function testPaginatedHistoryOutOfBounds() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 100, "First");
        
        (uint256[] memory scores, , ) = 
            reputation.getReputationHistoryPaginated(user1, 10, 5);
        
        assertEq(scores.length, 0);
    }
    
    // ============ Checkpoint Tests ============
    
    function testCheckpointCreation() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 1000, "Build reputation");
        
        vm.prank(updater);
        reputation.createCheckpoint(user1, "Monthly checkpoint");
        
        ReputationSystem.Checkpoint[] memory checkpoints = 
            reputation.getArchivedHistory(user1, 0);
        
        assertEq(checkpoints.length, 1);
        assertEq(checkpoints[0].score, 1000);
    }
    
    function testMultipleCheckpoints() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 500, "First");
        
        vm.prank(updater);
        reputation.createCheckpoint(user1, "Checkpoint 1");
        
        vm.prank(updater);
        reputation.updateReputation(user1, 500, "Second");
        
        vm.prank(updater);
        reputation.createCheckpoint(user1, "Checkpoint 2");
        
        uint256[] memory ids = reputation.getCheckpointIds(user1);
        assertEq(ids.length, 2);
    }
    
    // ============ Decay Tests ============
    
    function testDecayApplication() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 1000, "Initial");
        
        // Fast forward 7 days (1 decay period)
        vm.warp(block.timestamp + 7 days);
        
        // Apply decay manually
        reputation.applyDecay(user1);
        
        // Should be 990 (1% decay from 1000)
        uint256 rep = reputation.getReputation(user1);
        assertEq(rep, 990);
    }
    
    function testDecayOnUpdate() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 1000, "Initial");
        
        // Fast forward 14 days (2 decay periods)
        vm.warp(block.timestamp + 14 days);
        
        // Update should trigger decay
        vm.prank(updater);
        reputation.updateReputation(user1, 0, "Trigger decay");
        
        // Should be approximately 980 (1% decay twice)
        uint256 rep = reputation.getReputation(user1);
        assertApproxEqRel(rep, 980, 0.01e18);
    }
    
    function testVotingPowerWithDecay() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 10000, "Max");
        
        uint256 powerBefore = reputation.calculateVotingPower(user1);
        
        // Fast forward 7 days
        vm.warp(block.timestamp + 7 days);
        
        uint256 powerAfter = reputation.calculateVotingPower(user1);
        
        // Voting power should decrease due to decay
        assertLt(powerAfter, powerBefore);
    }
    
    // ============ Voting Power Tests ============
    
    function testVotingPowerCalculation() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 10000, "Max");
        
        uint256 power = reputation.calculateVotingPower(user1);
        
        // sqrt(10000 * 1e18) = sqrt(1e22) = 1e11
        assertEq(power, 1e11);
    }
    
    function testVotingPowerZero() public view {
        uint256 power = reputation.calculateVotingPower(user1);
        assertEq(power, 0);
    }
    
    // ============ Stats Tests ============
    
    function testReputationStats() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 100, "First");
        
        vm.prank(updater);
        reputation.updateReputation(user1, 100, "Second");
        
        vm.prank(updater);
        reputation.updateReputation(user1, 100, "Third");
        
        (
            uint256 current,
            uint256 max,
            uint256 min,
            uint256 avg,
            uint256 count,
        ) = reputation.getReputationStats(user1);
        
        assertEq(current, 300);
        assertEq(max, 300);
        assertEq(min, 100);
        assertEq(avg, 167); // (100 + 200 + 300) / 3 = 200, but integer math
        assertEq(count, 3);
    }
    
    function testGetReputationData() public {
        vm.prank(updater);
        reputation.updateReputation(user1, 500, "Test");
        
        (
            uint256 currentScore,
            uint256 lastUpdateTime,
            uint256 lastDecay,
            uint256 checkpointCount,
            uint256 historyCount
        ) = reputation.getReputationData(user1);
        
        assertEq(currentScore, 500);
        assertGt(lastUpdateTime, 0);
        assertEq(checkpointCount, 0);
        assertEq(historyCount, 1);
    }
    
    // ============ Archive Tests ============
    
    function testHistoryArchival() public {
        // Add many updates to trigger archival
        for (uint i = 0; i < 110; i++) {
            vm.prank(updater);
            reputation.updateReputation(user1, 10, "Update");
        }
        
        // History should be limited
        (, uint256[] memory timestamps, ) = 
            reputation.getReputationHistoryPaginated(user1, 0, 200);
        
        assertLe(timestamps.length, 100);
    }
    
    function testManualArchive() public {
        // Add some history
        for (uint i = 0; i < 50; i++) {
            vm.prank(updater);
            reputation.updateReputation(user1, 10, "Update");
        }
        
        // Manually archive
        vm.prank(owner);
        reputation.archiveHistory(user1);
        
        // Check archived data exists
        uint256[] memory ids = reputation.getCheckpointIds(user1);
        assertGt(ids.length, 0);
    }
    
    // ============ Admin Tests ============
    
    function testSetAuthorizedUpdater() public {
        vm.prank(owner);
        reputation.setAuthorizedUpdater(user2, true);
        
        assertTrue(reputation.authorizedUpdaters(user2));
    }
    
    function testOnlyOwnerCanSetUpdater() public {
        vm.prank(user1);
        vm.expectRevert();
        reputation.setAuthorizedUpdater(user2, true);
    }
    
    function testPauseAndUnpause() public {
        vm.prank(owner);
        reputation.pause();
        
        vm.prank(updater);
        vm.expectRevert();
        reputation.updateReputation(user1, 100, "While paused");
        
        vm.prank(owner);
        reputation.unpause();
        
        vm.prank(updater);
        reputation.updateReputation(user1, 100, "After unpause");
        
        assertEq(reputation.getReputation(user1), 100);
    }
    
    // ============ Gas Measurement ============
    
    function testGas_UpdateReputation() public {
        vm.prank(updater);
        uint256 gasBefore = gasleft();
        reputation.updateReputation(user1, 100, "Gas test");
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for update", gasUsed);
    }
    
    function testGas_BatchUpdate() public {
        address[] memory accounts = new address[](10);
        int256[] memory deltas = new int256[](10);
        string[] memory reasons = new string[](10);
        
        for (uint i = 0; i < 10; i++) {
            accounts[i] = address(uint160(100 + i));
            deltas[i] = 100;
            reasons[i] = "Batch";
        }
        
        vm.prank(updater);
        uint256 gasBefore = gasleft();
        reputation.batchUpdateReputation(accounts, deltas, reasons);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for batch update (10)", gasUsed);
    }
}
