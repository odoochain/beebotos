// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title MockOracle
 * @dev Mock price oracle for testing
 */
contract MockOracle {
    mapping(address => uint256) public prices;
    mapping(address => uint256) public decimals;
    
    address public owner;
    
    event PriceUpdated(address indexed asset, uint256 price);
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    constructor() {
        owner = msg.sender;
    }
    
    /**
     * @dev Set price for an asset
     */
    function setPrice(address asset, uint256 price) external onlyOwner {
        prices[asset] = price;
        emit PriceUpdated(asset, price);
    }
    
    /**
     * @dev Set decimals for an asset
     */
    function setDecimals(address asset, uint256 _decimals) external onlyOwner {
        decimals[asset] = _decimals;
    }
    
    /**
     * @dev Get latest price
     */
    function getLatestPrice(address asset) external view returns (uint256) {
        require(prices[asset] > 0, "Price not set");
        return prices[asset];
    }
    
    /**
     * @dev Get price with decimals
     */
    function getPrice(address asset) external view returns (uint256, uint256) {
        return (prices[asset], decimals[asset]);
    }
}
