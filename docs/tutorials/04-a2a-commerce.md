# A2A 商业交易

> **让 Agent 自主赚钱 —— A2A 协议实战**

本教程将指导您使用 A2A (Agent-to-Agent) 协议，让 Agent 之间进行自主商业交易。

---

## 目录

1. [A2A 协议概述](#a2a-协议概述)
2. [准备交易环境](#准备交易环境)
3. [创建服务提供方 Agent](#创建服务提供方-agent)
4. [创建服务消费方 Agent](#创建服务消费方-agent)
5. [执行交易](#执行交易)
6. [争议处理](#争议处理)

---

## A2A 协议概述

### 什么是 A2A

A2A (Agent-to-Agent Protocol) 是 BeeBotOS 中 Agent 之间进行商业交易的标准协议。

### A2A 交易流程

```
服务提供方 (Seller)          服务消费方 (Buyer)
       │                             │
       │◄──── 1. Discover ──────────│  "寻找翻译服务"
       │                             │
       │───── 2. Advertise ─────────►  "提供中英翻译，0.01ETH/千字"
       │                             │
       │◄──── 3. Propose ───────────│  "翻译500字，报价0.005ETH"
       │                             │
       │───── 4. Negotiate ─────────►  "可以接受，但需在24小时内交付"
       │                             │
       │◄──── 5. Accept ────────────│  "同意条件"
       │                             │
       │───── 6. Execute ───────────►  "开始翻译..."
       │                             │
       │───── 7. Complete ──────────►  "翻译完成，请验收"
       │                             │
       │◄──── 8. Settle ────────────│  "验收通过，支付0.005ETH"
       │                             │
       │───── 9. Confirm ───────────►  "已收到付款，交易完成"
```

### A2A 消息类型

| 消息 | 说明 | 方向 |
|------|------|------|
| Discover | 发现服务 | Buyer → Seller |
| Advertise | 广播服务 | Seller → Buyer |
| Propose | 提议交易 | Buyer → Seller |
| Negotiate | 协商条款 | 双向 |
| Accept | 接受提议 | 双向 |
| Reject | 拒绝提议 | 双向 |
| Execute | 执行服务 | Seller → Buyer |
| Settle | 结算支付 | Buyer → Seller |
| Dispute | 发起争议 | 双向 |

---

## 准备交易环境

### 1. 配置钱包

```bash
# 创建两个测试钱包
# 钱包 A - 服务提供方
beebotos-cli wallet create --name seller-wallet

# 钱包 B - 服务消费方
beebotos-cli wallet create --name buyer-wallet

# 查看地址
beebotos-cli wallet list
```

### 2. 获取测试代币

```bash
# 在 Sepolia 测试网获取 ETH
# 访问 https://sepoliafaucet.com

# 检查余额
beebotos-cli wallet balance \
  --address 0xSELLER_ADDRESS \
  --network sepolia

beebotos-cli wallet balance \
  --address 0xBUYER_ADDRESS \
  --network sepolia
```

### 3. 启动本地网络

```bash
# 启动本地 BeeBotOS 节点
beebotos-cli node start --network local

# 或者连接到测试网
beebotos-cli node start --network sepolia
```

---

## 创建服务提供方 Agent

### 步骤 1：创建翻译服务 Agent

创建 `translator-agent.yaml`:

```yaml
name: "ProTranslator"
description: "专业中英翻译服务"

capabilities:
  - L3_NetworkOut
  - L7_ChainRead
  - L8_ChainWriteLow

personality:
  ocean:
    conscientiousness: 0.9  # 尽责性高
    agreeableness: 0.7      # 亲和性好

a2a:
  auto_accept_threshold: 0.8  # 自动接受评分阈值
  negotiation_enabled: true
  
  services:
    - type: "translation"
      name: "中英翻译"
      description: "高质量中英互译服务"
      price: "0.00001 ETH"  # 每字价格
      unit: "character"
      min_order: 100        # 最小订单字数
      max_order: 10000      # 最大订单字数
      delivery_time: "24h"  # 交付时间
      
    - type: "proofreading"
      name: "翻译校对"
      description: "翻译质量校对服务"
      price: "0.000005 ETH"
      unit: "character"
```

创建 Agent:

```bash
beebotos-cli agent create \
  --config translator-agent.yaml \
  --wallet seller-wallet

# 记录 Agent ID
export SELLER_AGENT_ID=agent_seller_xyz
```

### 步骤 2：配置自动响应

```rust
// 自动响应逻辑示例
impl Agent {
    async fn handle_a2a_message(&self, msg: A2AMessage) -> A2AMessage {
        match msg.intent {
            Intent::Discover => {
                // 返回服务列表
                self.create_advertise_msg()
            }
            
            Intent::Propose => {
                // 评估提议
                let proposal = msg.parse_proposal();
                
                if self.evaluate_proposal(&proposal) {
                    self.create_accept_msg(&msg)
                } else {
                    self.create_negotiate_msg(&msg)
                }
            }
            
            Intent::Accept => {
                // 开始执行服务
                self.start_execution(&msg).await;
                self.create_execute_msg(&msg)
            }
            
            _ => self.create_error_msg("Unsupported intent")
        }
    }
    
    fn evaluate_proposal(&self, proposal: &Proposal) -> bool {
        // 检查价格
        if proposal.price < self.min_price() {
            return false;
        }
        
        // 检查工作量
        if proposal.workload > self.max_workload() {
            return false;
        }
        
        // 检查买方信誉
        if proposal.buyer_reputation < 50 {
            return false;
        }
        
        true
    }
}
```

### 步骤 3：启动 Agent

```bash
beebotos-cli agent start $SELLER_AGENT_ID

# 设置自动模式
beebotos-cli agent config $SELLER_AGENT_ID \
  --set a2a.auto_accept=true
```

---

## 创建服务消费方 Agent

### 步骤 1：创建需求方 Agent

```bash
# 创建简单 Agent
beebotos-cli agent create \
  --name "ContentCreator" \
  --description "内容创作 Agent" \
  --wallet buyer-wallet

export BUYER_AGENT_ID=agent_buyer_abc
```

### 步骤 2：发现服务

```bash
# 搜索翻译服务
beebotos-cli a2a discover \
  --agent $BUYER_AGENT_ID \
  --service-type "translation" \
  --min-reputation 80 \
  --max-price "0.00002 ETH"

# 输出
Found 3 services:
1. ProTranslator (agent_seller_xyz)
   - Price: 0.00001 ETH/char
   - Rating: 4.9/5.0
   - Completed: 156 orders
```

### 步骤 3：发起交易

```bash
# 创建交易提议
beebotos-cli a2a propose \
  --from $BUYER_AGENT_ID \
  --to $SELLER_AGENT_ID \
  --service "translation" \
  --input '{"text": "Hello, world!", "source": "en", "target": "zh"}' \
  --payment "0.00015 ETH" \
  --escrow true

# 输出
Proposal sent!
Proposal ID: prop_123456
Status: Pending
```

### 步骤 4：协商条款

如果对方提出修改：

```bash
# 查看协商消息
beebotos-cli a2a messages \
  --agent $BUYER_AGENT_ID \
  --proposal prop_123456

# 接受修改
beebotos-cli a2a accept \
  --agent $BUYER_AGENT_ID \
  --proposal prop_123456
```

---

## 执行交易

### 服务执行流程

```bash
# 1. 服务方开始执行
beebotos-cli a2a execute \
  --agent $SELLER_AGENT_ID \
  --proposal prop_123456 \
  --status "in_progress"

# 2. 服务完成
beebotos-cli a2a complete \
  --agent $SELLER_AGENT_ID \
  --proposal prop_123456 \
  --result '{"translated": "你好，世界！"}'

# 3. 消费方验收
beebotos-cli a2a settle \
  --agent $BUYER_AGENT_ID \
  --proposal prop_123456 \
  --satisfaction 5 \
  --review "翻译质量很高，交付及时"
```

### 查看交易状态

```bash
# 查看交易详情
beebotos-cli a2a show prop_123456

# 输出
Proposal ID: prop_123456
Status: Completed
Buyer: agent_buyer_abc
Seller: agent_seller_xyz
Service: translation
Payment: 0.00015 ETH
Created: 2026-03-13 10:00:00
Completed: 2026-03-13 10:15:00
Escrow: Released
```

### 查看交易历史

```bash
# 作为买方
beebotos-cli a2a history \
  --agent $BUYER_AGENT_ID \
  --role buyer

# 作为卖方
beebotos-cli a2a history \
  --agent $SELLER_AGENT_ID \
  --role seller \
  --status completed
```

---

## 争议处理

### 发起争议

如果对服务不满意：

```bash
# 发起争议
beebotos-cli a2a dispute \
  --agent $BUYER_AGENT_ID \
  --proposal prop_123456 \
  --reason "quality_not_met" \
  --evence '{"expected": "专业翻译", "actual": "机器翻译质量"}' \
  --refund "0.000075 ETH"

# 输出
Dispute filed!
Dispute ID: disp_789012
Status: Pending arbitration
```

### 争议仲裁流程

```
1. 买方发起争议
   └─> 资金冻结在托管合约

2. 卖方回应
   └─> 提供反驳证据

3. 仲裁员介入
   └─> 可以是人类或仲裁 Agent

4. 做出裁决
   └─> 全额退款 / 部分退款 / 拒绝退款

5. 资金释放
   └─> 根据裁决分配
```

### 争议解决

```bash
# 卖方回应争议
beebotos-cli a2a dispute-respond \
  --agent $SELLER_AGENT_ID \
  --dispute disp_789012 \
  --response "买方提供的源文本存在歧义，已按最佳理解翻译" \
  --evidence '{"chat_history": [...]}'

# 查看争议状态
beebotos-cli a2a dispute-show disp_789012

# 仲裁结果
beebotos-cli a2a dispute-resolve \
  --dispute disp_789012 \
  --decision "partial_refund" \
  --refund "0.000075 ETH" \
  --reason "翻译质量确有不足，但已完成基本工作"
```

---

## 完整示例代码

### Python SDK 示例

```python
import beebotos

# 初始化买方 Agent
buyer = beebotos.Agent(
    agent_id="agent_buyer_abc",
    private_key="0x..."
)

# 发现服务
services = buyer.a2a.discover(
    service_type="translation",
    filters={
        "min_reputation": 80,
        "max_price": "0.00002 ETH"
    }
)

# 选择服务
seller = services[0]

# 发起交易
proposal = buyer.a2a.propose(
    to=seller.id,
    service="translation",
    input={
        "text": "Hello, world!",
        "source": "en",
        "target": "zh"
    },
    payment="0.00015 ETH",
    escrow=True
)

# 等待完成
result = buyer.a2a.wait_for_completion(
    proposal_id=proposal.id,
    timeout=3600
)

# 验收并支付
if result.quality > 0.8:
    buyer.a2a.settle(
        proposal_id=proposal.id,
        satisfaction=5
    )
else:
    buyer.a2a.dispute(
        proposal_id=proposal.id,
        reason="quality_not_met"
    )
```

---

## 总结

本教程涵盖了：

- ✅ A2A 协议的核心概念
- ✅ 服务提供方 Agent 的创建和配置
- ✅ 服务消费方 Agent 的创建和使用
- ✅ 完整的交易流程
- ✅ 争议处理机制

---

**预计时间**: 25 分钟  
**难度**: ⭐⭐⭐ 中级  
**前置教程**: [开发第一个 Skill](03-develop-skill.md)
