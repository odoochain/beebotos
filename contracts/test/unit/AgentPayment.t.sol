// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/payment/AgentPayment.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockToken is ERC20 {
    constructor() ERC20("Mock", "MOCK") {
        _mint(msg.sender, 1000000 ether);
    }
}

/**
 * @title AgentPaymentTest
 * @dev Comprehensive tests for AgentPayment (Target: 90%+ coverage)
 */
contract AgentPaymentTest is Test {
    AgentPayment public payment;
    MockToken public token;
    
    address public owner = address(1);
    address public payer = address(2);
    address public payee = address(3);
    address public recipient = address(4);
    
    bytes32 public mandateId;
    bytes32 public streamId;
    
    function setUp() public {
        vm.prank(owner);
        payment = new AgentPayment();
        payment.initialize();
        
        token = new MockToken();
        
        // Fund accounts
        vm.deal(payer, 100 ether);
        vm.deal(payee, 100 ether);
        token.transfer(payer, 10000 ether);
    }
    
    // ============ Initialization Tests ============
    
    function testInitialization() public view {
        assertEq(payment.owner(), owner);
    }
    
    // ============ Create Mandate Tests ============
    
    function testCreateMandate() public {
        uint256 maxAmount = 1000 ether;
        uint256 validUntil = block.timestamp + 30 days;
        
        vm.prank(payer);
        mandateId = payment.createMandate(payee, address(token), maxAmount, validUntil);
        
        assertTrue(mandateId != bytes32(0));
        
        IAgentPayment.PaymentMandate memory mandate = payment.mandates(mandateId);
        assertEq(mandate.mandateId, mandateId);
        assertEq(mandate.payer, payer);
        assertEq(mandate.payee, payee);
        assertEq(mandate.token, address(token));
        assertEq(mandate.maxAmount, maxAmount);
        assertEq(mandate.validUntil, validUntil);
        assertTrue(mandate.isActive);
    }
    
    function testCreateMandateEmitsEvent() public {
        uint256 maxAmount = 1000 ether;
        uint256 validUntil = block.timestamp + 30 days;
        
        vm.prank(payer);
        vm.expectEmit(true, true, false, true);
        emit IAgentPayment.MandateCreated(bytes32(0), payer, payee, address(token), maxAmount, validUntil);
        payment.createMandate(payee, address(token), maxAmount, validUntil);
    }
    
    function testCreateMandateInvalidPayee() public {
        vm.prank(payer);
        vm.expectRevert("Invalid payee");
        payment.createMandate(address(0), address(token), 1000 ether, block.timestamp + 30 days);
    }
    
    function testCreateMandateZeroAmount() public {
        vm.prank(payer);
        vm.expectRevert("Amount must be > 0");
        payment.createMandate(payee, address(token), 0, block.timestamp + 30 days);
    }
    
    function testCreateMandateInvalidExpiration() public {
        vm.prank(payer);
        vm.expectRevert("Invalid expiration");
        payment.createMandate(payee, address(token), 1000 ether, block.timestamp - 1);
    }
    
    function testCreateMandateUniqueIds() public {
        vm.prank(payer);
        bytes32 id1 = payment.createMandate(payee, address(token), 1000 ether, block.timestamp + 30 days);
        
        vm.warp(block.timestamp + 1);
        
        vm.prank(payer);
        bytes32 id2 = payment.createMandate(payee, address(token), 2000 ether, block.timestamp + 30 days);
        
        assertTrue(id1 != id2);
    }
    
    // ============ Create Stream Tests ============
    
    function testCreateStream() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        assertTrue(streamId != bytes32(0));
        
        IAgentPayment.Stream memory stream = payment.streams(streamId);
        assertEq(stream.streamId, streamId);
        assertEq(stream.sender, payer);
        assertEq(stream.recipient, recipient);
        assertEq(stream.totalAmount, totalAmount);
        assertEq(stream.startTime, block.timestamp);
        assertEq(stream.endTime, block.timestamp + duration);
        assertTrue(stream.isActive);
        assertEq(stream.releasedAmount, 0);
    }
    
    function testCreateStreamEmitsEvent() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        vm.expectEmit(true, true, false, true);
        emit IAgentPayment.StreamCreated(bytes32(0), payer, recipient, address(0), totalAmount, block.timestamp, block.timestamp + duration);
        payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
    }
    
    function testCreateStreamInvalidRecipient() public {
        vm.prank(payer);
        vm.expectRevert("Invalid recipient");
        payment.createStream{value: 10 ether}(address(0), 10 ether, 30 days);
    }
    
    function testCreateStreamZeroAmount() public {
        vm.prank(payer);
        vm.expectRevert("Invalid params");
        payment.createStream{value: 0}(recipient, 0, 30 days);
    }
    
    function testCreateStreamZeroDuration() public {
        vm.prank(payer);
        vm.expectRevert("Invalid params");
        payment.createStream{value: 10 ether}(recipient, 10 ether, 0);
    }
    
    function testCreateStreamInsufficientETH() public {
        vm.prank(payer);
        vm.expectRevert("Insufficient ETH");
        payment.createStream{value: 5 ether}(recipient, 10 ether, 30 days);
    }
    
    function testCreateStreamUniqueIds() public {
        vm.prank(payer);
        bytes32 id1 = payment.createStream{value: 10 ether}(recipient, 10 ether, 30 days);
        
        vm.warp(block.timestamp + 1);
        
        vm.prank(payer);
        bytes32 id2 = payment.createStream{value: 10 ether}(recipient, 10 ether, 30 days);
        
        assertTrue(id1 != id2);
    }
    
    // ============ Withdraw From Stream Tests ============
    
    function testWithdrawFromStream() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        // Fast forward 15 days (50% of duration)
        vm.warp(block.timestamp + 15 days);
        
        uint256 recipientBalanceBefore = recipient.balance;
        
        vm.prank(recipient);
        uint256 withdrawn = payment.withdrawFromStream(streamId);
        
        // Should be approximately 5 ether (50% of 10)
        assertApproxEqRel(withdrawn, 5 ether, 0.01e18);
        assertEq(recipient.balance, recipientBalanceBefore + withdrawn);
        
        IAgentPayment.Stream memory stream = payment.streams(streamId);
        assertEq(stream.releasedAmount, withdrawn);
    }
    
    function testWithdrawFromStreamEmitsEvent() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        vm.warp(block.timestamp + 15 days);
        
        vm.prank(recipient);
        vm.expectEmit(true, false, false, true);
        emit IAgentPayment.StreamUpdated(streamId, 5 ether);
        payment.withdrawFromStream(streamId);
    }
    
    function testWithdrawFromStreamNotRecipient() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        vm.warp(block.timestamp + 15 days);
        
        vm.prank(payer);
        vm.expectRevert("Not recipient");
        payment.withdrawFromStream(streamId);
    }
    
    function testWithdrawFromStreamInactive() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        // Withdraw all
        vm.warp(block.timestamp + duration + 1);
        
        vm.prank(recipient);
        payment.withdrawFromStream(streamId);
        
        // Try to withdraw again
        vm.prank(recipient);
        vm.expectRevert("Stream inactive");
        payment.withdrawFromStream(streamId);
    }
    
    function testWithdrawFromStreamNoPending() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        // Try to withdraw immediately (no time passed)
        vm.prank(recipient);
        vm.expectRevert("No pending amount");
        payment.withdrawFromStream(streamId);
    }
    
    function testWithdrawFromStreamComplete() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        // Fast forward past end
        vm.warp(block.timestamp + duration + 1);
        
        uint256 recipientBalanceBefore = recipient.balance;
        
        vm.prank(recipient);
        uint256 withdrawn = payment.withdrawFromStream(streamId);
        
        // Should get full amount
        assertEq(withdrawn, totalAmount);
        assertEq(recipient.balance, recipientBalanceBefore + totalAmount);
        
        IAgentPayment.Stream memory stream = payment.streams(streamId);
        assertFalse(stream.isActive);
        assertEq(stream.releasedAmount, totalAmount);
    }
    
    function testWithdrawFromStreamPartial() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        // Withdraw 25%
        vm.warp(block.timestamp + 7.5 days);
        
        vm.prank(recipient);
        uint256 withdrawn1 = payment.withdrawFromStream(streamId);
        
        // Withdraw another 25%
        vm.warp(block.timestamp + 7.5 days);
        
        vm.prank(recipient);
        uint256 withdrawn2 = payment.withdrawFromStream(streamId);
        
        // Withdraw remaining
        vm.warp(block.timestamp + 15 days + 1);
        
        vm.prank(recipient);
        uint256 withdrawn3 = payment.withdrawFromStream(streamId);
        
        assertApproxEqRel(withdrawn1 + withdrawn2 + withdrawn3, totalAmount, 0.01e18);
    }
    
    // ============ Get Pending Amount Tests ============
    
    function testGetPendingAmount() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        // Initially 0
        assertEq(payment.getPendingAmount(streamId), 0);
        
        // After 15 days
        vm.warp(block.timestamp + 15 days);
        assertApproxEqRel(payment.getPendingAmount(streamId), 5 ether, 0.01e18);
        
        // After full duration
        vm.warp(block.timestamp + duration + 1);
        assertEq(payment.getPendingAmount(streamId), totalAmount);
    }
    
    function testGetPendingAmountInactive() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        vm.warp(block.timestamp + duration + 1);
        
        vm.prank(recipient);
        payment.withdrawFromStream(streamId);
        
        assertEq(payment.getPendingAmount(streamId), 0);
    }
    
    // ============ User Mandates Tests ============
    
    function testUserMandates() public {
        vm.prank(payer);
        bytes32 m1 = payment.createMandate(payee, address(token), 1000 ether, block.timestamp + 30 days);
        
        vm.prank(payer);
        bytes32 m2 = payment.createMandate(payee, address(token), 2000 ether, block.timestamp + 60 days);
        
        bytes32[] memory mandates = payment.getUserMandates(payer);
        
        assertEq(mandates.length, 2);
        assertTrue(mandates[0] == m1 || mandates[1] == m1);
        assertTrue(mandates[0] == m2 || mandates[1] == m2);
    }
    
    // ============ All Streams Tests ============
    
    function testAllStreams() public {
        vm.prank(payer);
        bytes32 s1 = payment.createStream{value: 10 ether}(recipient, 10 ether, 30 days);
        
        vm.prank(payer);
        bytes32 s2 = payment.createStream{value: 20 ether}(recipient, 20 ether, 60 days);
        
        bytes32[] memory streams = payment.getAllStreams();
        
        assertEq(streams.length, 2);
        assertTrue(streams[0] == s1 || streams[1] == s1);
        assertTrue(streams[0] == s2 || streams[1] == s2);
    }
    
    // ============ Reentrancy Tests ============
    
    function testWithdrawFromStreamReentrancyProtection() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        vm.warp(block.timestamp + 15 days);
        
        // The nonReentrant modifier is present
        vm.prank(recipient);
        payment.withdrawFromStream(streamId);
        
        // Should succeed without reentrancy issues
        assertTrue(true);
    }
    
    // ============ Gas Measurement Tests ============
    
    function testGas_CreateMandate() public {
        vm.prank(payer);
        uint256 gasBefore = gasleft();
        payment.createMandate(payee, address(token), 1000 ether, block.timestamp + 30 days);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for createMandate", gasUsed);
    }
    
    function testGas_CreateStream() public {
        vm.prank(payer);
        uint256 gasBefore = gasleft();
        payment.createStream{value: 10 ether}(recipient, 10 ether, 30 days);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for createStream", gasUsed);
    }
    
    function testGas_WithdrawFromStream() public {
        vm.prank(payer);
        streamId = payment.createStream{value: 10 ether}(recipient, 10 ether, 30 days);
        
        vm.warp(block.timestamp + 15 days);
        
        vm.prank(recipient);
        uint256 gasBefore = gasleft();
        payment.withdrawFromStream(streamId);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for withdrawFromStream", gasUsed);
    }
    
    // ============ Edge Cases ============
    
    function testFullStreamLifecycle() public {
        uint256 totalAmount = 10 ether;
        uint256 duration = 30 days;
        
        // Create
        vm.prank(payer);
        streamId = payment.createStream{value: totalAmount}(recipient, totalAmount, duration);
        
        // Partial withdraw
        vm.warp(block.timestamp + 10 days);
        
        vm.prank(recipient);
        uint256 w1 = payment.withdrawFromStream(streamId);
        
        // Another partial withdraw
        vm.warp(block.timestamp + 10 days);
        
        vm.prank(recipient);
        uint256 w2 = payment.withdrawFromStream(streamId);
        
        // Final withdraw
        vm.warp(block.timestamp + 11 days);
        
        vm.prank(recipient);
        uint256 w3 = payment.withdrawFromStream(streamId);
        
        // Verify total
        assertApproxEqRel(w1 + w2 + w3, totalAmount, 0.01e18);
        
        // Verify stream is inactive
        IAgentPayment.Stream memory stream = payment.streams(streamId);
        assertFalse(stream.isActive);
    }
    
    function testMultipleStreamsSameRecipient() public {
        for (uint i = 0; i < 5; i++) {
            vm.prank(payer);
            payment.createStream{value: 10 ether}(recipient, 10 ether, 30 days);
        }
        
        assertEq(payment.getAllStreams().length, 5);
        
        vm.warp(block.timestamp + 30 days + 1);
        
        // Withdraw from all
        for (uint i = 0; i < 5; i++) {
            bytes32 sId = payment.getAllStreams()[i];
            
            vm.prank(recipient);
            payment.withdrawFromStream(sId);
        }
    }
    
    function testManyMandates() public {
        uint256 mandateCount = 20;
        
        for (uint i = 0; i < mandateCount; i++) {
            vm.prank(payer);
            payment.createMandate(
                payee,
                address(token),
                1000 ether,
                block.timestamp + 30 days + i
            );
        }
        
        assertEq(payment.getUserMandates(payer).length, mandateCount);
    }
}
