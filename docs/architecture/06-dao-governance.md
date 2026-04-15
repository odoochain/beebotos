# DAO 治理

> **去中心化自治组织架构**

---

## 治理架构

```
┌─────────────────────────────────────────────────────────────┐
│                    DAO Governance                           │
├─────────────────────────────────────────────────────────────┤
│  ┌────────────┐  ┌────────────┐  ┌────────────┐            │
│  │  Proposal  │  │   Voting   │  │ Delegation │            │
│  │  提案系统  │  │  投票系统  │  │  委托代理  │            │
│  └─────┬──────┘  └─────┬──────┘  └─────┬──────┘            │
│        │               │               │                    │
│        └───────────────┼───────────────┘                    │
│                        │                                     │
│  ┌─────────────────────┴─────────────────────┐              │
│  │              Treasury                     │              │
│  │              财库管理                     │              │
│  └───────────────────────────────────────────┘              │
└─────────────────────────────────────────────────────────────┘
```

---

## 混合治理模式

### 参与者类型

| 类型 | 权重 | 说明 |
|------|------|------|
| 人类持币者 | 代币持有量 | 主要决策权 |
| AI Agent | 委托权重 | 专业领域投票 |
| 核心团队 | 过渡期保留 | 渐进式去中心化 |

### 投票权重计算

```
权重 = 代币余额 × 质押时间系数 × 声誉系数

质押时间系数: 1.0 ~ 2.0
声誉系数: 0.5 ~ 1.5
```

---

## 提案类型

| 类型 | 门槛 | 时间 |
|------|------|------|
| 参数变更 | 5% | 3 天 |
| 国库支出 | 10% | 5 天 |
| 合约升级 | 20% | 7 天 |
| 紧急行动 | 多签 | 即时 |

---

## 提案生命周期

```
Created → Pending → Active → Succeeded → Queued → Executed
              ↓         ↓           ↓
           Cancelled  Defeated    Expired
```

---

## 委托机制

### 委托给 Agent

```solidity
function delegate(
    address delegatee,
    uint256 amount,
    uint256 duration
) external;
```

### Agent 投票策略

```rust
pub trait VotingStrategy {
    fn evaluate(&self, proposal: &Proposal) -> VoteDecision;
}

pub struct ConservativeStrategy;

impl VotingStrategy for ConservativeStrategy {
    fn evaluate(&self, proposal: &Proposal) -> VoteDecision {
        match proposal.proposal_type {
            ProposalType::ParameterChange => {
                if proposal.risk_score < 0.3 {
                    VoteDecision::For
                } else {
                    VoteDecision::Against
                }
            }
            // ...
        }
    }
}
```

---

## 声誉系统

### 声誉分数

```
初始分数: 50
范围: 0 ~ 100

加分项:
- 完成任务 +5
- 获得好评 +3
- 参与治理 +1

减分项:
- 违规 -10
- 差评 -5
- 恶意行为 -20
```

---

**最后更新**: 2026-03-13
