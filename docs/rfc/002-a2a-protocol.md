# RFC-002: A2A (Agent-to-Agent) 协议规范

> **状态**: 已接受 (Accepted)  
> **作者**: BeeBotOS Core Team  
> **创建日期**: 2026-01-20  
> **最后更新**: 2026-03-13  
> **版本**: v1.0

---

## 目录

1. [摘要](#摘要)
2. [动机](#动机)
3. [设计目标](#设计目标)
4. [协议概述](#协议概述)
5. [消息格式](#消息格式)
6. [通信流程](#通信流程)
7. [协商机制](#协商机制)
8. [支付与结算](#支付与结算)
9. [争议处理](#争议处理)
10. [安全考量](#安全考量)
11. [实现参考](#实现参考)

---

## 摘要

本 RFC 定义了 A2A (Agent-to-Agent) 协议，一个用于 AI Agent 之间进行商业交易的标准协议。A2A 协议支持服务发现、价格协商、任务执行、支付结算和争议仲裁，为 Web4.0 时代的 Agent 经济提供基础设施。

**核心要点**:
- 标准化的消息格式和通信流程
- 支持多轮协商的价格发现机制
- 内置的托管支付系统
- 去中心化的争议仲裁

---

## 动机

### 为什么需要 A2A 协议

当前的 AI Agent 系统是孤立的：
- 无法相互通信
- 无法交换价值
- 无法形成协作网络

A2A 协议旨在解决这些问题，构建一个：
- **开放的 Agent 经济** - 任何 Agent 都可以提供服务
- **自主的商业网络** - Agent 可以自主交易
- **可信的协作环境** - 通过区块链保障交易安全

### 应用场景

1. **DeFi 分析 Agent** 向 **交易 Agent** 提供市场分析报告
2. **翻译 Agent** 为 **内容创作 Agent** 提供翻译服务
3. **数据抓取 Agent** 向 **分析 Agent** 提供原始数据

---

## 设计目标

### 必须实现 (MUST)

- **M1**: 支持服务发现和广播
- **M2**: 支持多轮价格协商
- **M3**: 支持托管支付和结算
- **M4**: 支持争议仲裁
- **M5**: 端到端加密通信

### 应该实现 (SHOULD)

- **S1**: 支持批量交易
- **S2**: 支持订阅模式
- **S3**: 支持声誉系统

### 可以实现 (MAY)

- **C1**: 支持跨链结算
- **C2**: 支持零知识证明隐私保护

---

## 协议概述

### 协议栈

```
┌─────────────────────────────────────┐
│  Application Layer                  │
│  - Service Discovery                │
│  - Negotiation                      │
│  - Settlement                       │
├─────────────────────────────────────┤
│  Message Layer                      │
│  - Message Encoding                 │
│  - Intent Types                     │
│  - Payload Format                   │
├─────────────────────────────────────┤
│  Session Layer                      │
│  - Conversation Management          │
│  - State Machine                    │
├─────────────────────────────────────┤
│  Transport Layer                    │
│  - libp2p                           │
│  - WebSocket                        │
│  - gRPC                             │
├─────────────────────────────────────┤
│  Security Layer                     │
│  - TLS 1.3                          │
│  - End-to-End Encryption            │
│  - Digital Signatures               │
└─────────────────────────────────────┘
```

### 角色定义

| 角色 | 描述 | 职责 |
|------|------|------|
| **Service Provider** | 服务提供方 | 提供服务，执行任务 |
| **Service Consumer** | 服务消费方 | 发现服务，发起请求 |
| **Arbitrator** | 仲裁者 | 处理争议，做出裁决 |

---

## 消息格式

### 基础消息结构

```protobuf
// a2a/message.proto
syntax = "proto3";

message A2AMessage {
  // 消息头
  MessageHeader header = 1;
  
  // 消息体
  oneof body {
    DiscoverMessage discover = 10;
    AdvertiseMessage advertise = 11;
    ProposeMessage propose = 12;
    NegotiateMessage negotiate = 13;
    AcceptMessage accept = 14;
    RejectMessage reject = 15;
    ExecuteMessage execute = 16;
    CompleteMessage complete = 17;
    SettleMessage settle = 18;
    DisputeMessage dispute = 19;
  }
  
  // 签名
  Signature signature = 100;
}

message MessageHeader {
  // 协议版本
  string version = 1;
  
  // 消息 ID
  string message_id = 2;
  
  // 会话 ID
  string conversation_id = 3;
  
  // 发送方
  string sender = 4;
  
  // 接收方
  string recipient = 5;
  
  // 回复的消息 ID
  string in_reply_to = 6;
  
  // 时间戳 (Unix 毫秒)
  int64 timestamp = 7;
  
  // 消息过期时间
  int64 expires_at = 8;
  
  // 优先级
  Priority priority = 9;
}

enum Priority {
  LOW = 0;
  NORMAL = 1;
  HIGH = 2;
  CRITICAL = 3;
}
```

### 具体消息类型

#### 1. 发现 (Discover)

```protobuf
message DiscoverMessage {
  // 查询条件
  repeated ServiceFilter filters = 1;
  
  // 期望的响应数量
  int32 limit = 2;
}

message ServiceFilter {
  // 服务类型
  string service_type = 1;
  
  // 最小声誉分数
  double min_reputation = 2;
  
  // 最高价格
  string max_price = 3;
  
  // 交付时间要求
  string max_delivery_time = 4;
}
```

#### 2. 广播 (Advertise)

```protobuf
message AdvertiseMessage {
  // 服务列表
  repeated Service services = 1;
  
  // 提供者信息
  ProviderInfo provider = 2;
}

message Service {
  // 服务 ID
  string service_id = 1;
  
  // 服务类型
  string service_type = 2;
  
  // 服务名称
  string name = 3;
  
  // 描述
  string description = 4;
  
  // 价格
  Price price = 5;
  
  // 交付时间
  string delivery_time = 6;
  
  // 服务元数据
  map<string, string> metadata = 7;
}

message Price {
  // 金额
  string amount = 1;
  
  // 代币地址 (空表示 ETH)
  string token = 2;
  
  // 计费单位
  string unit = 3;
}
```

#### 3. 提议 (Propose)

```protobuf
message ProposeMessage {
  // 请求的服务
  string service_id = 1;
  
  // 服务输入
  bytes input = 2;
  
  // 输入格式
  string input_type = 3;
  
  // 期望输出格式
  string output_type = 4;
  
  // 报价
  Price offer = 5;
  
  // 支付条款
  PaymentTerms payment = 6;
  
  // 交付要求
  DeliveryRequirements delivery = 7;
}

message PaymentTerms {
  // 是否使用托管
  bool escrow = 1;
  
  // 托管合约地址
  string escrow_address = 2;
  
  // 支付时间
  PaymentTiming timing = 3;
  
  // 退款条件
  repeated RefundCondition refund_conditions = 4;
}

enum PaymentTiming {
  UPFRONT = 0;      // 预付
  ON_DELIVERY = 1;  // 交付时
  ON_ACCEPTANCE = 2; // 验收后
}
```

#### 4. 协商 (Negotiate)

```protobuf
message NegotiateMessage {
  // 原提议 ID
  string original_proposal_id = 1;
  
  // 修改项
  repeated Modification modifications = 2;
  
  // 修改原因
  string reason = 3;
  
  // 是否最终报价
  bool final_offer = 4;
}

message Modification {
  // 修改的字段
  string field = 1;
  
  // 原值
  string old_value = 2;
  
  // 新值
  string new_value = 3;
}
```

#### 5. 接受 (Accept)

```protobuf
message AcceptMessage {
  // 接受的提议 ID
  string proposal_id = 1;
  
  // 接受的条款
  AcceptedTerms terms = 2;
  
  // 预计开始时间
  int64 start_time = 3;
}
```

#### 6. 执行 (Execute)

```protobuf
message ExecuteMessage {
  // 任务 ID
  string task_id = 1;
  
  // 执行状态
  ExecutionStatus status = 2;
  
  // 进度 (0-100)
  int32 progress = 3;
  
  // 状态说明
  string message = 4;
  
  // 预计完成时间
  int64 estimated_completion = 5;
}

enum ExecutionStatus {
  PENDING = 0;
  IN_PROGRESS = 1;
  PAUSED = 2;
  COMPLETED = 3;
  FAILED = 4;
}
```

#### 7. 完成 (Complete)

```protobuf
message CompleteMessage {
  // 任务 ID
  string task_id = 1;
  
  // 执行结果
  ExecutionResult result = 2;
  
  // 输出数据
  bytes output = 3;
  
  // 输出类型
  string output_type = 4;
  
  // 执行元数据
  ExecutionMetadata metadata = 5;
}

message ExecutionResult {
  bool success = 1;
  string error_message = 2;
  repeated string warnings = 3;
}
```

#### 8. 结算 (Settle)

```protobuf
message SettleMessage {
  // 任务 ID
  string task_id = 1;
  
  // 满意度评分 (1-5)
  int32 satisfaction = 2;
  
  // 评价
  string review = 3;
  
  // 是否释放托管资金
  bool release_escrow = 4;
  
  // 支付金额 (可能与原价不同)
  string final_payment = 5;
}
```

#### 9. 争议 (Dispute)

```protobuf
message DisputeMessage {
  // 争议的任务 ID
  string task_id = 1;
  
  // 争议原因
  DisputeReason reason = 2;
  
  // 详细描述
  string description = 3;
  
  // 证据
  repeated Evidence evidence = 4;
  
  // 期望的解决方案
  string requested_resolution = 5;
}

enum DisputeReason {
  QUALITY_NOT_MET = 0;
  DELIVERY_DELAYED = 1;
  COMMUNICATION_ISSUE = 2;
  SCOPE_MISMATCH = 3;
  OTHER = 4;
}
```

---

## 通信流程

### 标准交易流程

```
阶段 1: 发现
─────────────
Consumer → Discover → Network
Network → [Multiple Advertise] → Consumer

阶段 2: 协商
─────────────
Consumer → Propose → Provider
Provider → Negotiate → Consumer (可选)
Consumer → Accept → Provider
  或
Consumer → Reject → Provider

阶段 3: 执行
─────────────
Provider → Execute (IN_PROGRESS) → Consumer
Provider → [Progress Updates] → Consumer
Provider → Execute (COMPLETED) → Consumer
  或
Provider → Execute (FAILED) → Consumer

阶段 4: 结算
─────────────
Consumer → Settle → Provider
Provider → Confirm → Consumer

阶段 5: 争议 (如需要)
─────────────
Consumer/Provider → Dispute → Arbitrator
Arbitrator → Resolution → Both
```

### 状态机

```
                    ┌─────────────┐
         ┌─────────►│   START     │
         │          └──────┬──────┘
         │                 │ Discover/Advertise
         │                 ▼
         │          ┌─────────────┐
         │          │  NEGOTIATING │◄──────┐
         │          └──────┬──────┘       │
         │                 │ Propose      │
         │                 ▼              │
         │          ┌─────────────┐       │
         │          │   PENDING   │       │
         │          └──────┬──────┘       │
         │                 │ Accept       │
         │                 ▼              │
         │          ┌─────────────┐       │
         │          │  EXECUTING  │       │
         │          └──────┬──────┘       │
         │         ┌───────┴───────┐      │
         │         │               │      │
         │         ▼               ▼      │
         │   ┌─────────┐     ┌─────────┐  │
         └───┤ REJECTED │     │COMPLETED│  │
             └─────────┘     └────┬────┘  │
                                   │       │
                    ┌──────────────┼───────┘
                    │              │
                    ▼              ▼
              ┌─────────┐    ┌─────────┐
              │ DISPUTED │    │SETTLED  │
              └────┬────┘    └─────────┘
                   │
                   ▼
              ┌─────────┐
              │RESOLVED │
              └─────────┘
```

---

## 协商机制

### 价格发现

1. **买方发起**: 提出初始价格
2. **卖方响应**: 接受、拒绝或还价
3. **多轮协商**: 最多 5 轮
4. **超时机制**: 每轮 24 小时超时

### 协商策略

```rust
pub trait NegotiationStrategy {
    /// 评估报价
    fn evaluate_offer(&self, offer: &Offer) -> EvaluationResult;
    
    /// 生成还价
    fn counter_offer(&self, offer: &Offer) -> Option<Offer>;
    
    /// 是否接受
    fn should_accept(&self, offer: &Offer) -> bool;
}

/// 保守策略
pub struct ConservativeStrategy {
    min_acceptable: Price,
    max_rounds: u8,
}

impl NegotiationStrategy for ConservativeStrategy {
    fn evaluate_offer(&self, offer: &Offer) -> EvaluationResult {
        if offer.price >= self.min_acceptable {
            EvaluationResult::Acceptable
        } else if offer.price >= self.min_acceptable * 0.8 {
            EvaluationResult::Negotiable
        } else {
            EvaluationResult::TooLow
        }
    }
    
    fn counter_offer(&self, offer: &Offer) -> Option<Offer> {
        Some(Offer {
            price: self.min_acceptable * 1.1,
            ..offer.clone()
        })
    }
}
```

---

## 支付与结算

### 托管机制

```solidity
// Escrow.sol
contract A2AEscrow {
    struct Escrow {
        address payer;
        address payee;
        uint256 amount;
        address token;
        EscrowStatus status;
        uint256 created_at;
        uint256 expires_at;
    }
    
    mapping(bytes32 => Escrow) public escrows;
    
    function createEscrow(
        bytes32 taskId,
        address payee,
        uint256 amount,
        address token,
        uint256 duration
    ) external payable returns (bytes32 escrowId) {
        // 转移资金到托管
        if (token == address(0)) {
            require(msg.value == amount, "Incorrect ETH amount");
        } else {
            IERC20(token).transferFrom(msg.sender, address(this), amount);
        }
        
        escrowId = keccak256(abi.encodePacked(taskId, msg.sender, block.timestamp));
        
        escrows[escrowId] = Escrow({
            payer: msg.sender,
            payee: payee,
            amount: amount,
            token: token,
            status: EscrowStatus.ACTIVE,
            created_at: block.timestamp,
            expires_at: block.timestamp + duration
        });
    }
    
    function release(bytes32 escrowId) external {
        Escrow storage e = escrows[escrowId];
        require(msg.sender == e.payer, "Only payer can release");
        require(e.status == EscrowStatus.ACTIVE, "Escrow not active");
        
        e.status = EscrowStatus.RELEASED;
        
        // 转移资金给收款方
        _transfer(e.payee, e.amount, e.token);
    }
    
    function refund(bytes32 escrowId) external {
        Escrow storage e = escrows[escrowId];
        require(block.timestamp > e.expires_at, "Escrow not expired");
        require(e.status == EscrowStatus.ACTIVE, "Escrow not active");
        
        e.status = EscrowStatus.REFUNDED;
        
        // 退还资金给付款方
        _transfer(e.payer, e.amount, e.token);
    }
}
```

### 支付模式

| 模式 | 说明 | 适用场景 |
|------|------|---------|
| 预付 | 任务开始前全额支付 | 信任度高的小额交易 |
| 里程碑 | 按阶段支付 | 长期项目 |
| 后付 | 任务完成后支付 | 买方市场 |
| 托管 | 资金锁定，完成后释放 | 新合作方 |

---

## 争议处理

### 仲裁流程

```
1. 发起争议
   └─> 提交证据和诉求

2. 选择仲裁员
   ├─> 双方共同选择
   ├─> 随机分配
   └─> AI 仲裁员

3. 证据提交期
   └─> 双方提交证据 (7天)

4. 仲裁裁决
   └─> 仲裁员做出决定

5. 执行裁决
   └─> 托管资金按裁决分配
```

### AI 仲裁员

```rust
pub struct AIArbitrator {
    model: Box<dyn LanguageModel>,
}

impl Arbitrator for AIArbitrator {
    async fn arbitrate(&self, dispute: &Dispute) -> Resolution {
        // 分析证据
        let evidence_analysis = self.analyze_evidence(&dispute.evidence).await;
        
        // 评估双方论点
        let argument_assessment = self.assess_arguments(
            &dispute.plaintiff_argument,
            &dispute.defendant_argument
        ).await;
        
        // 参考历史案例
        let precedents = self.find_precedents(&dispute).await;
        
        // 做出裁决
        let decision = self.make_decision(
            evidence_analysis,
            argument_assessment,
            precedents
        ).await;
        
        Resolution {
            outcome: decision.outcome,
            compensation: decision.compensation,
            reasoning: decision.reasoning,
        }
    }
}
```

---

## 安全考量

### 1. 消息签名

所有消息必须签名：

```rust
impl A2AMessage {
    pub fn sign(&mut self, keypair: &Keypair) {
        let payload = self.serialize_for_signing();
        self.signature = keypair.sign(&payload);
    }
    
    pub fn verify(&self, public_key: &PublicKey) -> bool {
        let payload = self.serialize_for_signing();
        public_key.verify(&payload, &self.signature)
    }
}
```

### 2. 重放攻击防护

- 消息包含时间戳
- 消息包含唯一 ID
- 验证时间窗口 (±5 分钟)

### 3. 端到端加密

```rust
// 使用 X25519 + AES-256-GCM
pub fn encrypt_message(
    message: &A2AMessage,
    recipient_key: &PublicKey,
    sender_key: &Keypair,
) -> EncryptedMessage {
    // ECDH 密钥交换
    let shared_secret = sender_key.diffie_hellman(recipient_key);
    
    // 派生加密密钥
    let encryption_key = hkdf(&shared_secret, b"a2a-encryption");
    
    // AES-256-GCM 加密
    let ciphertext = aes_gcm_encrypt(&message.serialize(), &encryption_key);
    
    EncryptedMessage {
        ciphertext,
        sender_public_key: sender_key.public_key(),
        nonce: generate_nonce(),
    }
}
```

---

## 实现参考

### Rust SDK

```rust
// crates/agents/src/a2a/client.rs
pub struct A2AClient {
    transport: Box<dyn Transport>,
    identity: AgentIdentity,
    crypto: CryptoProvider,
}

impl A2AClient {
    /// 发现服务
    pub async fn discover(&self, filters: &[ServiceFilter]) -> Result<Vec<Service>> {
        let msg = A2AMessage::discover(filters);
        let responses = self.transport.broadcast(msg).await?;
        Ok(self.parse_advertisements(responses))
    }
    
    /// 发起交易
    pub async fn propose(
        &self,
        provider: &AgentId,
        service: &Service,
        input: &[u8],
    ) -> Result<Proposal> {
        let msg = A2AMessage::propose(service, input);
        let response = self.transport.send_to(provider, msg).await?;
        Ok(Proposal::from_message(response)?)
    }
}
```

### 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_transaction_flow() {
        // 创建 Provider
        let provider = create_agent_with_service().await;
        
        // 创建 Consumer
        let consumer = create_agent().await;
        
        // 发现服务
        let services = consumer.discover(&[]).await.unwrap();
        assert!(!services.is_empty());
        
        // 发起提议
        let proposal = consumer.propose(
            &provider.id(),
            &services[0],
            b"test input"
        ).await.unwrap();
        
        // Provider 接受
        provider.accept(&proposal.id).await.unwrap();
        
        // 执行任务
        provider.execute(&proposal.id).await.unwrap();
        
        // 结算
        consumer.settle(&proposal.id, 5).await.unwrap();
        
        // 验证
        assert_eq!(provider.get_completed_tasks(), 1);
    }
}
```

---

## 更新历史

| 日期 | 版本 | 更新内容 |
|------|------|---------|
| 2026-01-20 | v0.1 | 初始草案 |
| 2026-02-15 | v0.9 | 增加争议处理机制 |
| 2026-03-13 | v1.0 | 正式发布 |

---

## 参考文档

- [FIPA ACL Specification](http://www.fipa.org/specs/fipa00061/SC00061G.html)
- [Hyperledger Indy](https://hyperledger-indy.readthedocs.io/)
- [Ocean Protocol](https://oceanprotocol.com/)
