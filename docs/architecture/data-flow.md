# 数据流

> **跨层数据流动与事件系统**

---

## 数据流概览

```
User Input
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 4: Gateway                                            │
│ - Protocol Conversion                                       │
│ - Authentication                                            │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Agent Runtime                                      │
│ - Intent Recognition                                        │
│ - Skill Routing                                             │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Social Brain                                       │
│ - Memory Retrieval                                          │
│ - Emotional Processing                                      │
│ - Reasoning                                                 │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Kernel                                             │
│ - Task Scheduling                                           │
│ - Resource Allocation                                       │
│ - Security Check                                            │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 0: Blockchain                                         │
│ - State Persistence                                         │
│ - Value Transfer                                            │
└─────────────────────────────────────────────────────────────┘
```

---

## 典型场景数据流

### 场景 1: 用户与 Agent 对话

```
User: "你好"
    │
    ▼
Gateway (REST/WebSocket)
    │ - 解析请求
    ▼
Agent Runtime
    │ - 加载会话上下文
    ▼
Social Brain
    │ - 检索相关记忆
    │ - 分析情绪
    │ - 生成响应
    ▼
Agent Runtime
    │ - 格式化输出
    ▼
User: "你好！我是你的助手。"
```

### 场景 2: Agent 执行 Skill

```
Agent
    │
    ▼
Kernel (Capability Check)
    │ - 验证 L3_NetworkOut
    ▼
WASM Sandbox
    │ - 执行 Skill 代码
    ▼
External API
    │ - HTTP 请求
    ▼
WASM Sandbox
    │ - 处理响应
    ▼
Agent
    │ - 返回结果
```

### 场景 3: A2A 商业交易

```
Agent A (Buyer)
    │
    ▼
A2A Protocol
    │ - 构建 Propose 消息
    ▼
P2P Network
    │ - 发送消息
    ▼
Agent B (Seller)
    │ - 接收消息
    │ - 评估提议
    │ - 发送 Accept
    ▼
P2P Network
    ▼
Agent A
    │ - 创建托管
    ▼
Blockchain
    │ - 锁定资金
    ▼
Agent B
    │ - 执行任务
    │ - 请求结算
    ▼
Agent A
    │ - 验收并释放托管
    ▼
Blockchain
    │ - 转移资金
```

---

## 事件系统

### 事件总线架构

```
Publisher ──► Event Bus ──► Subscribers
                 │
    ┌────────────┼────────────┐
    │            │            │
    ▼            ▼            ▼
 Handler 1   Handler 2   Handler 3
```

### 核心事件

| 事件 | 来源 | 订阅者 |
|------|------|--------|
| AgentCreated | Kernel | Runtime, Blockchain |
| TaskCompleted | Kernel | Runtime, Metrics |
| MemoryStored | Social Brain | Storage |
| PaymentReceived | Blockchain | Runtime, Notification |
| ProposalAccepted | A2A | Both Agents |

### 事件示例

```rust
pub enum Event {
    AgentCreated {
        agent_id: AgentId,
        owner: Address,
        timestamp: u64,
    },
    
    TaskCompleted {
        task_id: TaskId,
        agent_id: AgentId,
        result: TaskResult,
    },
    
    PaymentReceived {
        from: Address,
        to: Address,
        amount: U256,
        token: Address,
    },
}
```

---

## 数据持久化

### 存储分层

| 层级 | 存储 | 用途 |
|------|------|------|
| 热数据 | Redis | 会话、缓存 |
| 温数据 | PostgreSQL | 配置、元数据 |
| 冷数据 | S3 | 日志、备份 |
| 向量数据 | Qdrant | 记忆嵌入 |

### 数据流

```
Agent
    │
    ├────► Redis (短期)
    │
    ├────► PostgreSQL (长期)
    │
    ├────► Qdrant (向量)
    │
    └────► Blockchain (价值)
```

---

## 异步处理

### 消息队列

```
Producer ──► Queue ──► Consumer
              │
              ├─► Retry (失败)
              │
              └─► DLQ (死信)
```

### 使用场景

- 审计日志写入
- 邮件发送
- 区块链同步
- 索引更新

---

**最后更新**: 2026-03-13
