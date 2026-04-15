// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/**
 * @title SafeMath
 * @dev Math operations with safety checks
 * @notice Not needed for Solidity >=0.8.0 but useful for explicit overflow checks
 */
library SafeMath {
    /**
     * @dev Returns the addition of two unsigned integers, reverting on overflow.
     */
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        uint256 c = a + b;
        require(c >= a, "SafeMath: addition overflow");
        return c;
    }
    
    /**
     * @dev Returns the subtraction of two unsigned integers, reverting on overflow.
     */
    function sub(uint256 a, uint256 b) internal pure returns (uint256) {
        require(b <= a, "SafeMath: subtraction overflow");
        return a - b;
    }
    
    /**
     * @dev Returns the multiplication of two unsigned integers, reverting on overflow.
     */
    function mul(uint256 a, uint256 b) internal pure returns (uint256) {
        if (a == 0) return 0;
        uint256 c = a * b;
        require(c / a == b, "SafeMath: multiplication overflow");
        return c;
    }
    
    /**
     * @dev Returns the integer division of two unsigned integers.
     */
    function div(uint256 a, uint256 b) internal pure returns (uint256) {
        require(b > 0, "SafeMath: division by zero");
        return a / b;
    }
    
    /**
     * @dev Returns the remainder of dividing two unsigned integers.
     */
    function mod(uint256 a, uint256 b) internal pure returns (uint256) {
        require(b > 0, "SafeMath: modulo by zero");
        return a % b;
    }
    
    /**
     * @dev Returns the square root of a number
     */
    function sqrt(uint256 a) internal pure returns (uint256) {
        if (a == 0) return 0;
        uint256 x = a / 2 + 1;
        uint256 y = (x + a / x) / 2;
        while (x > y) {
            x = y;
            y = (x + a / x) / 2;
        }
        return x;
    }
    
    /**
     * @dev Returns the minimum of two numbers
     */
    function min(uint256 a, uint256 b) internal pure returns (uint256) {
        return a < b ? a : b;
    }
    
    /**
     * @dev Returns the maximum of two numbers
     */
    function max(uint256 a, uint256 b) internal pure returns (uint256) {
        return a > b ? a : b;
    }
    
    /**
     * @dev Converts a uint256 to int256 safely
     */
    function toInt256(uint256 value) internal pure returns (int256) {
        require(value <= uint256(type(int256).max), "SafeMath: value doesn't fit in int256");
        return int256(value);
    }
}
