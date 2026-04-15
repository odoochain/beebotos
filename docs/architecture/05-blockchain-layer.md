# 区块链层

> **Layer 0: 基础设施与跨链互操作**

---

## 多链架构

```
┌─────────────────────────────────────────────────────────────┐
│                   Cross-Chain Hub                           │
└─────────────┬───────────────────────────────┬───────────────┘
              │                               │
    ┌─────────┴──────────┐         ┌─────────┴──────────┐
    │   EVM Chains       │         │   Non-EVM Chains   │
    ├────────────────────┤         ├────────────────────┤
    │  Ethereum          │         │  Solana            │
    │  BSC               │         │  Bitcoin (future)  │
    │  Polygon           │         └────────────────────┘
    │  Arbitrum          │
    │  Optimism          │
    └────────────────────┘
```

---

## 支持的链

| 链 | 类型 | 特点 | 用途 |
|----|------|------|------|
| Ethereum | EVM | 安全最高 | 主网部署 |
| BSC | EVM | 成本低 | 高频交易 |
| Polygon | EVM | 速度快 | 日常使用 |
| Solana | Non-EVM | 高性能 | 高频场景 |

---

## 核心合约

### AgentRegistry

```solidity
contract AgentRegistry {
    struct Agent {
        bytes32 agentId;
        address owner;
        string metadataURI;
        uint256 reputation;
        bool isActive;
    }
    
    mapping(bytes32 => Agent) public agents;
    mapping(address => bytes32[]) public ownerAgents;
    
    event AgentRegistered(bytes32 indexed agentId, address indexed owner);
}
```

### SkillRegistry

```solidity
contract SkillRegistry {
    struct Skill {
        bytes32 skillId;
        bytes32 agentId;
        string name;
        uint256 price;
        uint256 usageCount;
    }
    
    mapping(bytes32 => Skill) public skills;
}
```

### CrossChainBridge

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
        uint256 amount
    );
}
```

---

## 跨链互操作

### 跨链消息传递

```
Chain A → Bridge Contract → Relayer → Bridge Contract → Chain B
              (lock)          (verify)      (mint/release)
```

### 支持的资产

- ETH/WETH
- USDC/USDT
- BEE (原生代币)
- 任意 ERC-20

---

## 身份系统 (DID)

### DID 结构

```
did:beebotos:{agent_id}:{chain}:{address}

示例:
did:beebotos:agent_abc123:eth:0x742d...
```

### 凭证验证

```solidity
contract DIDRegistry {
    mapping(bytes32 => DIDDocument) public documents;
    
    function verify(bytes32 agentId, bytes memory proof) external view returns (bool);
}
```

---

**最后更新**: 2026-03-13
