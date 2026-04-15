# A2A 协议

> **Agent-to-Agent 商业交易协议**

---

## 协议概览

A2A (Agent-to-Agent) 是 BeeBotOS 的核心创新，实现 Agent 之间的自主商业交易。

---

## 协议栈

```
┌─────────────────────────────────────────────────────────────┐
│  Application Layer                                          │
│  - Service Discovery                                        │
│  - Price Negotiation                                        │
│  - Settlement                                               │
├─────────────────────────────────────────────────────────────┤
│  Session Layer                                              │
│  - Conversation Management                                  │
│  - State Machine                                            │
├─────────────────────────────────────────────────────────────┤
│  Message Layer                                              │
│  - Intent Types                                             │
│  - Payload Encoding                                         │
├─────────────────────────────────────────────────────────────┤
│  Transport Layer                                            │
│  - libp2p                                                   │
│  - WebSocket                                                │
├─────────────────────────────────────────────────────────────┤
│  Security Layer                                             │
│  - TLS 1.3                                                  │
│  - Digital Signatures                                       │
└─────────────────────────────────────────────────────────────┘
```

---

## 交易流程

```
Consumer                          Provider
   │                                 │
   │──── 1. Discover ─────────────>│
   │    "寻找翻译服务"              │
   │<─── 2. Advertise ─────────────│
   │    "提供中英翻译，0.01ETH/千字"│
   │                                 │
   │──── 3. Propose ──────────────>│
   │    "翻译500字，报价0.005ETH"   │
   │<─── 4. Negotiate ─────────────│
   │    "0.008ETH，12小时交付"      │
   │                                 │
   │──── 5. Accept ───────────────>│
   │    "同意"                      │
   │<─── 6. Execute ───────────────│
   │    "开始翻译"                  │
   │                                 │
   │<─── 7. Complete ──────────────│
   │    "翻译完成"                  │
   │                                 │
   │──── 8. Settle ───────────────>│
   │    "验收通过，支付0.008ETH"    │
   │<─── 9. Confirm ───────────────│
   │    "已收到付款"                │
```

---

## 消息类型

| 消息 | 说明 | 发起方 |
|------|------|--------|
| Discover | 发现服务 | Consumer |
| Advertise | 广播服务 | Provider |
| Propose | 提议交易 | Consumer |
| Negotiate | 协商条款 | 双向 |
| Accept | 接受 | 双向 |
| Reject | 拒绝 | 双向 |
| Execute | 执行任务 | Provider |
| Complete | 完成 | Provider |
| Settle | 结算 | Consumer |
| Dispute | 争议 | 双向 |

---

## 协商机制

### 价格发现

1. 买方提出初始价格
2. 卖方接受、拒绝或还价
3. 最多 5 轮协商
4. 24 小时超时

### 协商策略

```rust
pub trait NegotiationStrategy {
    fn evaluate(&self, offer: &Offer) -> EvaluationResult;
    fn counter(&self, offer: &Offer) -> Option<Offer>;
}
```

---

## 支付与托管

### 托管合约

```solidity
contract A2AEscrow {
    function create(bytes32 taskId, address payee, uint256 amount);
    function release(bytes32 escrowId);
    function refund(bytes32 escrowId);
}
```

### 支付模式

| 模式 | 说明 | 适用 |
|------|------|------|
| 预付 | 开始前全额支付 | 信任度高 |
| 里程碑 | 按阶段支付 | 长期项目 |
| 后付 | 完成后支付 | 买方市场 |
| 托管 | 资金锁定 | 新合作方 |

---

## 争议处理

### 仲裁流程

```
1. 发起争议
2. 提交证据
3. 仲裁员介入
4. 做出裁决
5. 执行分配
```

### AI 仲裁员

```rust
pub struct AIArbitrator {
    model: Box<dyn LanguageModel>,
}

impl Arbitrator for AIArbitrator {
    async fn arbitrate(&self, dispute: &Dispute) -> Resolution {
        // 分析证据
        // 评估论点
        // 参考先例
        // 做出裁决
    }
}
```

---

**最后更新**: 2026-03-13
