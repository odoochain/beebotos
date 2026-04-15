// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../../src/a2a/A2ACommerce.sol";
import "../../src/a2a/DealEscrow.sol";

/**
 * @title A2ACommerceTest
 * @dev Comprehensive tests for A2ACommerce (Target: 90%+ coverage)
 */
contract A2ACommerceTest is Test {
    A2ACommerce public commerce;
    DealEscrow public escrow;
    
    address public owner = address(1);
    address public provider = address(2);
    address public buyer = address(3);
    address public feeRecipient = address(4);
    
    address public token;
    
    bytes32 public serviceId;
    bytes32 public dealId;
    
    function setUp() public {
        vm.startPrank(owner);
        
        // Deploy escrow first
        escrow = new DealEscrow();
        escrow.initialize(address(0), feeRecipient, 250); // 2.5% fee
        
        // Deploy commerce
        commerce = new A2ACommerce();
        commerce.initialize(address(escrow));
        
        // Set commerce in escrow
        escrow.setA2ACommerce(address(commerce));
        
        vm.stopPrank();
        
        // Fund accounts
        vm.deal(provider, 100 ether);
        vm.deal(buyer, 100 ether);
    }
    
    // ============ Initialization Tests ============
    
    function testInitialization() public view {
        assertEq(address(commerce.escrow()), address(escrow));
        assertEq(commerce.owner(), owner);
        assertEq(commerce.platformFeeBasisPoints(), 250);
        assertEq(commerce.feeRecipient(), owner);
    }
    
    function testCannotInitializeTwice() public {
        vm.prank(owner);
        vm.expectRevert("A2ACommerce: already initialized");
        commerce.initialize(address(escrow));
    }
    
    function testCannotInitializeWithZeroEscrow() public {
        A2ACommerce newCommerce = new A2ACommerce();
        vm.prank(owner);
        vm.expectRevert("A2ACommerce: zero escrow");
        newCommerce.initialize(address(0));
    }
    
    // ============ List Service Tests ============
    
    function testListService() public {
        vm.prank(provider);
        serviceId = commerce.listService(
            "ipfs://metadata",
            1 ether,
            address(0) // ETH
        );
        
        assertTrue(serviceId != bytes32(0));
        
        IA2ACommerce.ServiceListing memory service = commerce.getService(serviceId);
        assertEq(service.serviceId, serviceId);
        assertEq(service.provider, provider);
        assertEq(service.metadataURI, "ipfs://metadata");
        assertEq(service.price, 1 ether);
        assertEq(service.paymentToken, address(0));
        assertTrue(service.isActive);
    }
    
    function testListServiceEmitsEvent() public {
        vm.prank(provider);
        vm.expectEmit(true, true, false, true);
        emit IA2ACommerce.ServiceListed(bytes32(0), provider, 1 ether, address(0));
        commerce.listService("ipfs://metadata", 1 ether, address(0));
    }
    
    function testListServiceZeroPrice() public {
        vm.prank(provider);
        vm.expectRevert("A2ACommerce: price must be > 0");
        commerce.listService("ipfs://metadata", 0, address(0));
    }
    
    function testListServiceEmptyMetadata() public {
        vm.prank(provider);
        vm.expectRevert("A2ACommerce: metadata required");
        commerce.listService("", 1 ether, address(0));
    }
    
    function testListServiceWhenPaused() public {
        vm.prank(owner);
        commerce.pause();
        
        vm.prank(provider);
        vm.expectRevert();
        commerce.listService("ipfs://metadata", 1 ether, address(0));
    }
    
    function testListServiceGeneratesUniqueIds() public {
        vm.prank(provider);
        bytes32 id1 = commerce.listService("meta1", 1 ether, address(0));
        
        vm.prank(provider);
        bytes32 id2 = commerce.listService("meta2", 2 ether, address(0));
        
        assertTrue(id1 != id2);
    }
    
    // ============ Update Service Tests ============
    
    function testUpdateService() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(provider);
        commerce.updateService(serviceId, 2 ether, true);
        
        IA2ACommerce.ServiceListing memory service = commerce.getService(serviceId);
        assertEq(service.price, 2 ether);
        assertTrue(service.isActive);
    }
    
    function testUpdateServiceEmitsEvent() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(provider);
        vm.expectEmit(true, false, false, true);
        emit IA2ACommerce.ServiceUpdated(serviceId, 2 ether, true);
        commerce.updateService(serviceId, 2 ether, true);
    }
    
    function testUpdateServiceNotProvider() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        vm.expectRevert("A2ACommerce: not provider");
        commerce.updateService(serviceId, 2 ether, true);
    }
    
    function testUpdateServiceWhenPaused() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(owner);
        commerce.pause();
        
        vm.prank(provider);
        vm.expectRevert();
        commerce.updateService(serviceId, 2 ether, true);
    }
    
    // ============ Remove Service Tests ============
    
    function testRemoveService() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(provider);
        commerce.removeService(serviceId);
        
        IA2ACommerce.ServiceListing memory service = commerce.getService(serviceId);
        assertFalse(service.isActive);
    }
    
    function testRemoveServiceEmitsEvent() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(provider);
        vm.expectEmit(true, false, false, true);
        emit IA2ACommerce.ServiceUpdated(serviceId, 0, false);
        commerce.removeService(serviceId);
    }
    
    function testRemoveServiceNotProvider() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        vm.expectRevert("A2ACommerce: not provider");
        commerce.removeService(serviceId);
    }
    
    // ============ Create Deal Tests ============
    
    function testCreateDeal() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        assertTrue(dealId != bytes32(0));
        
        IA2ACommerce.Deal memory deal = commerce.getDeal(dealId);
        assertEq(deal.dealId, dealId);
        assertEq(deal.buyer, buyer);
        assertEq(deal.seller, provider);
        assertEq(deal.serviceId, serviceId);
        assertEq(deal.price, 1 ether);
        assertEq(deal.paymentToken, address(0));
        assertEq(uint(deal.status), uint(IA2ACommerce.DealStatus.Pending));
    }
    
    function testCreateDealEmitsEvent() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        vm.expectEmit(true, true, true, false);
        emit IA2ACommerce.DealCreated(bytes32(0), serviceId, buyer);
        commerce.createDeal(serviceId, block.timestamp + 1 days);
    }
    
    function testCreateDealServiceNotActive() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(provider);
        commerce.removeService(serviceId);
        
        vm.prank(buyer);
        vm.expectRevert("A2ACommerce: service not active");
        commerce.createDeal(serviceId, block.timestamp + 1 days);
    }
    
    function testCreateDealInvalidExpiration() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        vm.expectRevert("A2ACommerce: invalid expiration");
        commerce.createDeal(serviceId, block.timestamp - 1);
    }
    
    function testCreateDealCannotBuyOwnService() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(provider);
        vm.expectRevert("A2ACommerce: cannot buy own service");
        commerce.createDeal(serviceId, block.timestamp + 1 days);
    }
    
    // ============ Fund Deal Tests ============
    
    function testFundDealETH() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        uint256 buyerBalanceBefore = buyer.balance;
        
        vm.prank(buyer);
        commerce.fundDeal{value: 1 ether}(dealId);
        
        IA2ACommerce.Deal memory deal = commerce.getDeal(dealId);
        assertTrue(deal.escrowId != bytes32(0));
        assertEq(uint(deal.status), uint(IA2ACommerce.DealStatus.Funded));
        
        assertEq(buyer.balance, buyerBalanceBefore - 1 ether);
    }
    
    function testFundDealEmitsEvent() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        vm.expectEmit(true, false, false, false);
        emit IA2ACommerce.DealFunded(dealId, bytes32(0));
        commerce.fundDeal{value: 1 ether}(dealId);
    }
    
    function testFundDealInvalidStatus() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        commerce.fundDeal{value: 1 ether}(dealId);
        
        vm.prank(buyer);
        vm.expectRevert("A2ACommerce: invalid status");
        commerce.fundDeal{value: 1 ether}(dealId);
    }
    
    function testFundDealExpired() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 hours);
        
        vm.warp(block.timestamp + 2 hours);
        
        vm.prank(buyer);
        vm.expectRevert("A2ACommerce: deal expired");
        commerce.fundDeal{value: 1 ether}(dealId);
    }
    
    function testFundDealInsufficientETH() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        vm.expectRevert("A2ACommerce: insufficient ETH");
        commerce.fundDeal{value: 0.5 ether}(dealId);
    }
    
    function testFundDealRefundsExcess() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        uint256 buyerBalanceBefore = buyer.balance;
        
        vm.prank(buyer);
        commerce.fundDeal{value: 1.5 ether}(dealId);
        
        // Should refund 0.5 ether
        assertEq(buyer.balance, buyerBalanceBefore - 1 ether);
    }
    
    // ============ Complete Deal Tests ============
    
    function testCompleteDeal() public {
        // Setup
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        commerce.fundDeal{value: 1 ether}(dealId);
        
        // Mark as in progress (normally done by provider)
        // For this test, we assume the deal transitions to InProgress
        
        // Complete
        uint256 providerBalanceBefore = provider.balance;
        
        vm.prank(buyer);
        commerce.completeDeal(dealId);
        
        IA2ACommerce.Deal memory deal = commerce.getDeal(dealId);
        assertEq(uint(deal.status), uint(IA2ACommerce.DealStatus.Completed));
        
        // Provider should receive payment minus fee
        assertGt(provider.balance, providerBalanceBefore);
    }
    
    function testCompleteDealEmitsEvent() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        commerce.fundDeal{value: 1 ether}(dealId);
        
        vm.prank(buyer);
        vm.expectEmit(true, true, true, false);
        emit IA2ACommerce.DealCompleted(dealId, buyer, provider);
        commerce.completeDeal(dealId);
    }
    
    function testCompleteDealNotBuyer() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        commerce.fundDeal{value: 1 ether}(dealId);
        
        vm.prank(provider);
        vm.expectRevert("A2ACommerce: not buyer");
        commerce.completeDeal(dealId);
    }
    
    // ============ Cancel Deal Tests ============
    
    function testCancelDealPending() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        commerce.cancelDeal(dealId);
        
        IA2ACommerce.Deal memory deal = commerce.getDeal(dealId);
        assertEq(uint(deal.status), uint(IA2ACommerce.DealStatus.Cancelled));
    }
    
    function testCancelDealFunded() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        commerce.fundDeal{value: 1 ether}(dealId);
        
        uint256 buyerBalanceBefore = buyer.balance;
        
        vm.prank(buyer);
        commerce.cancelDeal(dealId);
        
        // Should get refund
        assertEq(buyer.balance, buyerBalanceBefore + 1 ether);
    }
    
    function testCancelDealEmitsEvent() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        vm.expectEmit(true, true, false, false);
        emit IA2ACommerce.DealCancelled(dealId, buyer);
        commerce.cancelDeal(dealId);
    }
    
    function testCancelDealNotAuthorized() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        address randomUser = address(999);
        vm.prank(randomUser);
        vm.expectRevert("A2ACommerce: not authorized");
        commerce.cancelDeal(dealId);
    }
    
    function testCancelDealByProvider() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(provider);
        commerce.cancelDeal(dealId);
        
        IA2ACommerce.Deal memory deal = commerce.getDeal(dealId);
        assertEq(uint(deal.status), uint(IA2ACommerce.DealStatus.Cancelled));
    }
    
    function testCancelDealByOwner() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(owner);
        commerce.cancelDeal(dealId);
        
        IA2ACommerce.Deal memory deal = commerce.getDeal(dealId);
        assertEq(uint(deal.status), uint(IA2ACommerce.DealStatus.Cancelled));
    }
    
    // ============ Platform Fee Tests ============
    
    function testCalculatePlatformFee() public view {
        uint256 fee = commerce.calculatePlatformFee(1 ether);
        // 2.5% of 1 ether = 0.025 ether
        assertEq(fee, 0.025 ether);
    }
    
    function testSetPlatformFee() public {
        vm.prank(owner);
        commerce.setPlatformFee(500); // 5%
        
        assertEq(commerce.platformFeeBasisPoints(), 500);
    }
    
    function testSetPlatformFeeMaxLimit() public {
        vm.prank(owner);
        vm.expectRevert("A2ACommerce: max 10%");
        commerce.setPlatformFee(1001); // > 10%
    }
    
    function testOnlyOwnerCanSetPlatformFee() public {
        vm.prank(provider);
        vm.expectRevert();
        commerce.setPlatformFee(500);
    }
    
    // ============ Fee Recipient Tests ============
    
    function testSetFeeRecipient() public {
        address newRecipient = address(999);
        
        vm.prank(owner);
        commerce.setFeeRecipient(newRecipient);
        
        assertEq(commerce.feeRecipient(), newRecipient);
    }
    
    function testSetFeeRecipientZeroAddress() public {
        vm.prank(owner);
        vm.expectRevert("A2ACommerce: zero address");
        commerce.setFeeRecipient(address(0));
    }
    
    function testOnlyOwnerCanSetFeeRecipient() public {
        vm.prank(provider);
        vm.expectRevert();
        commerce.setFeeRecipient(address(999));
    }
    
    // ============ Escrow Tests ============
    
    function testSetEscrow() public {
        DealEscrow newEscrow = new DealEscrow();
        newEscrow.initialize(address(commerce), feeRecipient, 250);
        
        vm.prank(owner);
        commerce.setEscrow(address(newEscrow));
        
        assertEq(address(commerce.escrow()), address(newEscrow));
    }
    
    function testSetEscrowZeroAddress() public {
        vm.prank(owner);
        vm.expectRevert("A2ACommerce: zero escrow");
        commerce.setEscrow(address(0));
    }
    
    function testOnlyOwnerCanSetEscrow() public {
        vm.prank(provider);
        vm.expectRevert();
        commerce.setEscrow(address(999));
    }
    
    // ============ Pause/Unpause Tests ============
    
    function testPause() public {
        vm.prank(owner);
        commerce.pause();
        
        assertTrue(commerce.paused());
    }
    
    function testUnpause() public {
        vm.prank(owner);
        commerce.pause();
        
        vm.prank(owner);
        commerce.unpause();
        
        assertFalse(commerce.paused());
    }
    
    function testOnlyOwnerCanPause() public {
        vm.prank(provider);
        vm.expectRevert();
        commerce.pause();
    }
    
    // ============ Emergency Withdraw Tests ============
    
    function testEmergencyWithdrawETH() public {
        // Fund contract
        vm.deal(address(commerce), 1 ether);
        
        uint256 ownerBalanceBefore = owner.balance;
        
        vm.prank(owner);
        commerce.emergencyWithdraw(address(0), owner, 1 ether);
        
        assertEq(owner.balance, ownerBalanceBefore + 1 ether);
        assertEq(address(commerce).balance, 0);
    }
    
    function testEmergencyWithdrawZeroAddress() public {
        vm.prank(owner);
        vm.expectRevert("A2ACommerce: zero address");
        commerce.emergencyWithdraw(address(0), address(0), 1 ether);
    }
    
    function testOnlyOwnerCanEmergencyWithdraw() public {
        vm.prank(provider);
        vm.expectRevert();
        commerce.emergencyWithdraw(address(0), provider, 1 ether);
    }
    
    // ============ Query Tests ============
    
    function testGetBuyerDeals() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        bytes32 deal1 = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        bytes32 deal2 = commerce.createDeal(serviceId, block.timestamp + 2 days);
        
        bytes32[] memory deals = commerce.getBuyerDeals(buyer);
        
        assertEq(deals.length, 2);
        assertTrue(deals[0] == deal1 || deals[1] == deal1);
        assertTrue(deals[0] == deal2 || deals[1] == deal2);
    }
    
    function testGetProviderServices() public {
        vm.prank(provider);
        bytes32 service1 = commerce.listService("meta1", 1 ether, address(0));
        
        vm.prank(provider);
        bytes32 service2 = commerce.listService("meta2", 2 ether, address(0));
        
        bytes32[] memory services = commerce.getProviderServices(provider);
        
        assertEq(services.length, 2);
        assertTrue(services[0] == service1 || services[1] == service1);
        assertTrue(services[0] == service2 || services[1] == service2);
    }
    
    // ============ Gas Measurement Tests ============
    
    function testGas_ListService() public {
        vm.prank(provider);
        uint256 gasBefore = gasleft();
        commerce.listService("ipfs://metadata", 1 ether, address(0));
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for listService", gasUsed);
    }
    
    function testGas_CreateDeal() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        uint256 gasBefore = gasleft();
        commerce.createDeal(serviceId, block.timestamp + 1 days);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for createDeal", gasUsed);
    }
    
    function testGas_FundDeal() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.prank(buyer);
        uint256 gasBefore = gasleft();
        commerce.fundDeal{value: 1 ether}(dealId);
        uint256 gasUsed = gasBefore - gasleft();
        
        emit log_named_uint("Gas used for fundDeal", gasUsed);
    }
    
    // ============ Edge Cases ============
    
    function testFullDealLifecycle() public {
        // List service
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        // Create deal
        vm.prank(buyer);
        dealId = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        // Fund deal
        vm.prank(buyer);
        commerce.fundDeal{value: 1 ether}(dealId);
        
        // Complete deal
        vm.prank(buyer);
        commerce.completeDeal(dealId);
        
        // Verify
        IA2ACommerce.Deal memory deal = commerce.getDeal(dealId);
        assertEq(uint(deal.status), uint(IA2ACommerce.DealStatus.Completed));
        
        IA2ACommerce.ServiceListing memory service = commerce.getService(serviceId);
        assertEq(service.totalSales, 1);
    }
    
    function testCreateMultipleDealsSameService() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        address[] memory buyers = new address[](5);
        for (uint i = 0; i < 5; i++) {
            buyers[i] = address(uint160(1000 + i));
            vm.deal(buyers[i], 2 ether);
            
            vm.prank(buyers[i]);
            commerce.createDeal(serviceId, block.timestamp + 1 days);
        }
        
        IA2ACommerce.ServiceListing memory service = commerce.getService(serviceId);
        assertEq(service.totalSales, 0); // Not completed yet
    }
    
    function testDealIdUniqueness() public {
        vm.prank(provider);
        serviceId = commerce.listService("ipfs://metadata", 1 ether, address(0));
        
        vm.prank(buyer);
        bytes32 deal1 = commerce.createDeal(serviceId, block.timestamp + 1 days);
        
        vm.warp(block.timestamp + 1);
        
        vm.prank(buyer);
        bytes32 deal2 = commerce.createDeal(serviceId, block.timestamp + 2 days);
        
        assertTrue(deal1 != deal2);
    }
}
