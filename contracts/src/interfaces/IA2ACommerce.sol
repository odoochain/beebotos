// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title IA2ACommerce
 * @notice Interface for Agent-to-Agent commerce protocol
 */
interface IA2ACommerce {
    struct Deal {
        bytes32 dealId;
        address buyer;
        address seller;
        bytes32 serviceId;
        uint256 price;
        address paymentToken;
        uint256 createdAt;
        uint256 expiresAt;
        DealStatus status;
        bytes32 escrowId;
    }
    
    struct ServiceListing {
        bytes32 serviceId;
        address provider;
        string metadataURI;
        uint256 price;
        address paymentToken;
        bool isActive;
        uint256 totalSales;
        uint256 ratingSum;
        uint256 ratingCount;
    }
    
    enum DealStatus {
        Pending, Funded, InProgress, Delivered, Completed, Disputed, Refunded, Cancelled
    }
    
    event ServiceListed(bytes32 indexed serviceId, address indexed provider, uint256 price, address paymentToken);
    event ServiceUpdated(bytes32 indexed serviceId, uint256 price, bool isActive);
    event DealCreated(bytes32 indexed dealId, bytes32 indexed serviceId, address indexed buyer);
    event DealFunded(bytes32 indexed dealId, bytes32 indexed escrowId);
    event DealCompleted(bytes32 indexed dealId, address indexed buyer, address indexed seller);
    event DealCancelled(bytes32 indexed dealId, address indexed cancelledBy);
    
    function listService(string calldata metadataURI, uint256 price, address paymentToken) external returns (bytes32);
    function createDeal(bytes32 serviceId, uint256 expiresAt) external returns (bytes32);
    function fundDeal(bytes32 dealId) external payable;
    function completeDeal(bytes32 dealId) external;
}
