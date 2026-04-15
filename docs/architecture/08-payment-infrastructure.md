# 支付基础设施

> **Agent 经济系统的金融基础设施**

---

## 支付架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Payment Infrastructure                    │
├─────────────────────────────────────────────────────────────┤
│  ┌────────────┐  ┌────────────┐  ┌────────────┐            │
│  │   Escrow   │  │  Multi-sig │  │  Treasury  │            │
│  │   托管     │  │   多签     │  │   财库     │            │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘            │
│        │               │               │                    │
│        └───────────────┼───────────────┘                    │
│                        │                                     │
│  ┌─────────────────────┴─────────────────────┐              │
│  │              Fee Model                    │              │
│  │              费用模型                     │              │
│  └───────────────────────────────────────────┘              │
└─────────────────────────────────────────────────────────────┘
```

---

## 托管系统

### 托管合约

```solidity
contract A2AEscrow {
    enum Status { ACTIVE, RELEASED, REFUNDED, DISPUTED }
    
    struct Escrow {
        address payer;
        address payee;
        uint256 amount;
        address token;
        Status status;
        uint256 createdAt;
        uint256 expiresAt;
    }
    
    mapping(bytes32 => Escrow) public escrows;
    
    function create(
        bytes32 taskId,
        address payee,
        uint256 amount,
        address token,
        uint256 duration
    ) external payable returns (bytes32 escrowId);
    
    function release(bytes32 escrowId) external;
    function refund(bytes32 escrowId) external;
    function dispute(bytes32 escrowId, string calldata reason) external;
}
```

### 托管状态机

```
ACTIVE → RELEASED (付款方确认)
       → REFUNDED (过期或争议裁决)
       → DISPUTED (争议发起)
              ↓
         RESOLVED → RELEASED/REFUNDED
```

---

## 多签钱包

### Gnosis Safe 集成

```solidity
contract BeeBotOSTreasury is GnosisSafe {
    // 2/3 多签
    // 核心团队 + 社区代表 + 安全专家
    
    function executeTransaction(
        address to,
        uint256 value,
        bytes calldata data
    ) external onlyConfirmed;
}
```

### 签名者配置

| 角色 | 数量 | 权限 |
|------|------|------|
| 核心团队 | 2 | 日常运营 |
| 社区代表 | 2 | 大额支出 |
| 安全专家 | 1 | 紧急操作 |

---

## 费用模型

### 费用类型

| 费用 | 比例 | 用途 |
|------|------|------|
| A2A 交易费 | 0.5% | 协议收入 |
| 技能市场费 | 2% | 开发者激励 |
| Gas 费 | 实际 | 链上执行 |
| 质押奖励 | 动态 | 网络安全 |

### 收入分配

```
协议收入:
├─ 50% 财库 (DAO 治理)
├─ 30% 质押奖励
├─ 15% 开发者基金
└─ 5% 团队运营
```

---

## 跨链支付

### 跨链桥

```solidity
contract CrossChainBridge {
    function bridge(
        uint256 fromChain,
        uint256 toChain,
        address token,
        uint256 amount
    ) external;
    
    event BridgeInitiated(
        bytes32 indexed bridgeId,
        address indexed sender,
        uint256 fromChain,
        uint256 toChain,
        uint256 amount
    );
}
```

### 支持的资产

- ETH/WETH
- USDC/USDT
- BEE (原生代币)
- WBTC

---

**最后更新**: 2026-03-13
