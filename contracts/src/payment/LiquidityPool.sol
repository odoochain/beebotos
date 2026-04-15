// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title LiquidityPool
 * @dev Simple AMM liquidity pool for token swaps
 */
contract LiquidityPool is ReentrancyGuard {
    using SafeERC20 for IERC20;
    
    struct Pool {
        address token0;
        address token1;
        uint112 reserve0;
        uint112 reserve1;
        uint256 totalSupply;
        mapping(address => uint256) balanceOf;
    }
    
    mapping(bytes32 => Pool) public pools;
    
    uint256 public constant MINIMUM_LIQUIDITY = 10**3;
    uint256 public constant FEE_NUMERATOR = 3; // 0.3%
    uint256 public constant FEE_DENOMINATOR = 1000;
    
    event PoolCreated(address indexed token0, address indexed token1, bytes32 poolId);
    event LiquidityAdded(
        bytes32 indexed poolId,
        address indexed provider,
        uint256 amount0,
        uint256 amount1,
        uint256 liquidity
    );
    event LiquidityRemoved(
        bytes32 indexed poolId,
        address indexed provider,
        uint256 amount0,
        uint256 amount1,
        uint256 liquidity
    );
    event Swap(
        bytes32 indexed poolId,
        address indexed sender,
        uint256 amountIn,
        uint256 amountOut,
        address indexed tokenIn
    );
    
    /**
     * @dev Create a new liquidity pool
     */
    function createPool(address tokenA, address tokenB) external returns (bytes32) {
        require(tokenA != tokenB, "Identical tokens");
        require(tokenA != address(0) && tokenB != address(0), "Zero address");
        
        (address token0, address token1) = tokenA < tokenB
            ? (tokenA, tokenB)
            : (tokenB, tokenA);
        
        bytes32 poolId = keccak256(abi.encodePacked(token0, token1));
        require(pools[poolId].token0 == address(0), "Pool exists");
        
        Pool storage pool = pools[poolId];
        pool.token0 = token0;
        pool.token1 = token1;
        
        emit PoolCreated(token0, token1, poolId);
        
        return poolId;
    }
    
    /**
     * @dev Add liquidity to a pool
     */
    function addLiquidity(
        bytes32 poolId,
        uint256 amount0Desired,
        uint256 amount1Desired,
        uint256 amount0Min,
        uint256 amount1Min,
        address to
    )
        external
        nonReentrant
        returns (uint256 liquidity)
    {
        Pool storage pool = pools[poolId];
        require(pool.token0 != address(0), "Pool not found");
        
        // Calculate optimal amounts
        (uint256 amount0, uint256 amount1) = _calculateLiquidity(
            pool,
            amount0Desired,
            amount1Desired,
            amount0Min,
            amount1Min
        );
        
        // Transfer tokens
        IERC20(pool.token0).safeTransferFrom(msg.sender, address(this), amount0);
        IERC20(pool.token1).safeTransferFrom(msg.sender, address(this), amount1);
        
        // Mint LP tokens
        if (pool.totalSupply == 0) {
            liquidity = sqrt(amount0 * amount1) - MINIMUM_LIQUIDITY;
            pool.balanceOf[address(0)] = MINIMUM_LIQUIDITY; // Lock minimum liquidity
        } else {
            liquidity = min(
                (amount0 * pool.totalSupply) / pool.reserve0,
                (amount1 * pool.totalSupply) / pool.reserve1
            );
        }
        
        require(liquidity > 0, "Insufficient liquidity");
        pool.balanceOf[to] += liquidity;
        pool.totalSupply += liquidity;
        
        // Update reserves
        pool.reserve0 += uint112(amount0);
        pool.reserve1 += uint112(amount1);
        
        emit LiquidityAdded(poolId, msg.sender, amount0, amount1, liquidity);
    }
    
    /**
     * @dev Remove liquidity from a pool
     */
    function removeLiquidity(
        bytes32 poolId,
        uint256 liquidity,
        uint256 amount0Min,
        uint256 amount1Min,
        address to
    )
        external
        nonReentrant
        returns (uint256 amount0, uint256 amount1)
    {
        Pool storage pool = pools[poolId];
        require(pool.token0 != address(0), "Pool not found");
        
        // Calculate amounts
        amount0 = (liquidity * pool.reserve0) / pool.totalSupply;
        amount1 = (liquidity * pool.reserve1) / pool.totalSupply;
        
        require(amount0 >= amount0Min && amount1 >= amount1Min, "Slippage exceeded");
        
        // Burn LP tokens
        pool.balanceOf[msg.sender] -= liquidity;
        pool.totalSupply -= liquidity;
        
        // Update reserves
        pool.reserve0 -= uint112(amount0);
        pool.reserve1 -= uint112(amount1);
        
        // Transfer tokens
        IERC20(pool.token0).safeTransfer(to, amount0);
        IERC20(pool.token1).safeTransfer(to, amount1);
        
        emit LiquidityRemoved(poolId, msg.sender, amount0, amount1, liquidity);
    }
    
    /**
     * @dev Swap tokens
     */
    function swap(
        bytes32 poolId,
        address tokenIn,
        uint256 amountIn,
        uint256 amountOutMin,
        address to
    )
        external
        nonReentrant
        returns (uint256 amountOut)
    {
        Pool storage pool = pools[poolId];
        require(pool.token0 != address(0), "Pool not found");
        require(tokenIn == pool.token0 || tokenIn == pool.token1, "Invalid token");
        
        bool isToken0 = tokenIn == pool.token0;
        (uint112 reserveIn, uint112 reserveOut) = isToken0
            ? (pool.reserve0, pool.reserve1)
            : (pool.reserve1, pool.reserve0);
        
        // Calculate output with fee
        uint256 amountInWithFee = amountIn * (FEE_DENOMINATOR - FEE_NUMERATOR);
        amountOut = (amountInWithFee * reserveOut) /
            (reserveIn * FEE_DENOMINATOR + amountInWithFee);
        
        require(amountOut >= amountOutMin, "Insufficient output");
        
        // Transfer input
        IERC20(tokenIn).safeTransferFrom(msg.sender, address(this), amountIn);
        
        // Transfer output
        address tokenOut = isToken0 ? pool.token1 : pool.token0;
        IERC20(tokenOut).safeTransfer(to, amountOut);
        
        // Update reserves
        if (isToken0) {
            pool.reserve0 += uint112(amountIn);
            pool.reserve1 -= uint112(amountOut);
        } else {
            pool.reserve1 += uint112(amountIn);
            pool.reserve0 -= uint112(amountOut);
        }
        
        emit Swap(poolId, msg.sender, amountIn, amountOut, tokenIn);
    }
    
    /**
     * @dev Calculate optimal liquidity amounts
     */
    function _calculateLiquidity(
        Pool storage pool,
        uint256 amount0Desired,
        uint256 amount1Desired,
        uint256 amount0Min,
        uint256 amount1Min
    )
        internal
        view
        returns (uint256 amount0, uint256 amount1)
    {
        if (pool.reserve0 == 0 && pool.reserve1 == 0) {
            (amount0, amount1) = (amount0Desired, amount1Desired);
        } else {
            uint256 amount1Optimal = (amount0Desired * pool.reserve1) / pool.reserve0;
            if (amount1Optimal <= amount1Desired) {
                require(amount1Optimal >= amount1Min, "Insufficient amount1");
                (amount0, amount1) = (amount0Desired, amount1Optimal);
            } else {
                uint256 amount0Optimal = (amount1Desired * pool.reserve0) / pool.reserve1;
                require(amount0Optimal >= amount0Min, "Insufficient amount0");
                (amount0, amount1) = (amount0Optimal, amount1Desired);
            }
        }
    }
    
    /**
     * @dev Get pool reserves
     */
    function getReserves(bytes32 poolId)
        external
        view
        returns (uint112 reserve0, uint112 reserve1)
    {
        Pool storage pool = pools[poolId];
        return (pool.reserve0, pool.reserve1);
    }
    
    function sqrt(uint256 x) internal pure returns (uint256) {
        if (x == 0) return 0;
        uint256 z = (x + 1) / 2;
        uint256 y = x;
        while (z < y) {
            y = z;
            z = (x / z + z) / 2;
        }
        return y;
    }
    
    function min(uint256 a, uint256 b) internal pure returns (uint256) {
        return a < b ? a : b;
    }
}
