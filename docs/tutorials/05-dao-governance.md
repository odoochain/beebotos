# DAO 治理参与

> **让 Agent 参与去中心化自治组织治理**

本教程将指导您如何使用 BeeBotOS DAO 功能，让 Agent 参与去中心化治理。

---

## 目录

1. [DAO 概述](#dao-概述)
2. [获取治理权限](#获取治理权限)
3. [提案生命周期](#提案生命周期)
4. [自动投票策略](#自动投票策略)
5. [委托代理投票](#委托代理投票)
6. [发起治理提案](#发起治理提案)

---

## DAO 概述

### 什么是 BeeBotOS DAO

BeeBotOS DAO 是一个混合治理系统，允许：
- 👤 **人类参与者** - 通过代币持有量投票
- 🤖 **AI Agent** - 通过委托参与治理
- 🏛️ **协同决策** - 人类与 Agent 共同决策

### 治理范围

| 治理事项 | 说明 | 投票门槛 |
|----------|------|---------|
| 参数变更 | 修改系统参数 | 5% 代币参与 |
| 国库支出 | 使用 DAO 资金 | 10% 代币参与 |
| 合约升级 | 升级智能合约 | 20% 代币参与 |
| 紧急行动 | 暂停、修复漏洞 | 多签执行 |

### 投票权重计算

```
投票权重 = 代币余额 × 质押时间系数 × 声誉系数 × 委托系数

其中：
- 质押时间系数: 1.0 ~ 2.0 (质押越久越高)
- 声誉系数: 0.5 ~ 1.5 (根据历史贡献)
- 委托系数: 1.0 ~ 1.2 (被其他 Agent 委托)
```

---

## 获取治理权限

### 步骤 1：获取 BEE 代币

```bash
# 查看当前余额
beebotos-cli token balance \
  --token BEE \
  --address 0xYOUR_ADDRESS

# 在 DEX 购买 BEE (示例)
beebotos-cli swap \
  --from ETH \
  --to BEE \
  --amount 0.1
```

### 步骤 2：质押代币

```bash
# 质押 BEE 获取治理权
beebotos-cli dao stake \
  --amount 1000 \
  --duration 90d  # 质押90天

# 查看质押信息
beebotos-cli dao stake-info \
  --address 0xYOUR_ADDRESS

# 输出
Staked: 1000 BEE
Stake weight: 1.5x
Voting power: 1500
Unlock time: 2026-06-13
```

### 步骤 3：注册治理 Agent

```bash
# 创建治理专用 Agent
beebotos-cli agent create \
  --name "GovernanceBot" \
  --type "governance" \
  --capabilities L7_ChainRead,L8_ChainWriteLow

export GOV_AGENT_ID=agent_gov_xxx

# 注册为治理参与者
beebotos-cli dao register-agent \
  --agent $GOV_AGENT_ID \
  --owner 0xYOUR_ADDRESS
```

---

## 提案生命周期

### 提案状态流转

```
Created (创建)
    │
    ▼
Pending (等待投票) ──► Cancelled (取消)
    │
    ▼
Active (投票中) ──────► Defeated (被否决)
    │
    ▼
Succeeded (通过) ─────► Expired (过期)
    │
    ▼
Queued (队列中)
    │
    ▼
Executed (已执行) / Failed (执行失败)
```

### 查看提案

```bash
# 列出所有活跃提案
beebotos-cli dao proposals \
  --status active \
  --limit 10

# 查看提案详情
beebotos-cli dao proposal-show 123

# 输出
Proposal #123
Title: 增加 A2A 协议手续费至 0.6%
Type: ParameterChange
Proposer: agent_proposer_xyz
Status: Active
For: 2,500,000 (62.5%)
Against: 1,500,000 (37.5%)
End time: 2026-03-20 12:00:00
```

---

## 自动投票策略

### 配置自动投票

创建 `voting-strategy.yaml`:

```yaml
name: "ConservativeStrategy"
description: "保守型投票策略"

rules:
  # 参数变更提案
  - type: ParameterChange
    condition: "increase_fee"
    action: "against"
    threshold: 0.5  # 增幅超过50%投反对
  
  - type: ParameterChange
    condition: "decrease_fee"
    action: "for"
  
  # 国库支出提案
  - type: TreasurySpend
    condition: "amount > 10000 BEE"
    action: "against"
    reason: "大额支出需人工审核"
  
  - type: TreasurySpend
    condition: "amount <= 10000 BEE"
    action: "analyze"
    # 分析受益方声誉
    reputation_threshold: 80
  
  # 合约升级提案
  - type: ContractUpgrade
    condition: "always"
    action: "analyze"
    require_audit: true
    audit_source: "certik,trail_of_bits"

analysis:
  # 使用 AI 分析提案
  enabled: true
  model: "gpt-4"
  
  prompt_template: |
    分析以下治理提案：
    标题: {{title}}
    描述: {{description}}
    类型: {{type}}
    
    请评估：
    1. 对生态系统的利弊
    2. 潜在的攻击向量
    3. 建议投票方向
```

### 应用策略

```bash
# 加载策略
beebotos-cli agent config $GOV_AGENT_ID \
  --set governance.strategy=voting-strategy.yaml

# 启用自动投票
beebotos-cli agent config $GOV_AGENT_ID \
  --set governance.auto_vote=true

# 设置人工审核阈值
beebotos-cli agent config $GOV_AGENT_ID \
  --set governance.manual_threshold=10000
```

### 手动投票

```bash
# 查看待投票提案
beebotos-cli dao proposals \
  --agent $GOV_AGENT_ID \
  --status active \
  --not-voted

# 投赞成票
beebotos-cli dao vote \
  --agent $GOV_AGENT_ID \
  --proposal 123 \
  --support true \
  --reason "有助于生态可持续发展"

# 投反对票
beebotos-cli dao vote \
  --agent $GOV_AGENT_ID \
  --proposal 124 \
  --support false \
  --reason "风险过高，建议更谨慎的方案"
```

---

## 委托代理投票

### 委托给 Agent

```bash
# 将投票权委托给治理 Agent
beebotos-cli dao delegate \
  --from 0xYOUR_ADDRESS \
  --to-agent $GOV_AGENT_ID \
  --amount 500 \
  --duration 30d

# 查看委托状态
beebotos-cli dao delegation \
  --delegator 0xYOUR_ADDRESS

# 输出
Delegator: 0xYOUR_ADDRESS
Delegatee: agent_gov_xxx
Amount: 500 BEE
Voting power: 750 (1.5x)
Expiry: 2026-04-13
```

### Agent 接受委托

```rust
impl GovernanceAgent {
    async fn handle_delegation(&self, delegation: Delegation) -> Result<()> {
        // 验证委托人声誉
        let delegator_rep = self.query_reputation(&delegation.delegator).await?;
        
        if delegator_rep < 50 {
            return Err("Delegator reputation too low".into());
        }
        
        // 记录委托
        self.record_delegation(delegation).await?;
        
        // 更新投票权重
        self.update_voting_power().await?;
        
        Ok(())
    }
}
```

### 撤销委托

```bash
# 撤销委托
beebotos-cli dao undelegate \
  --from 0xYOUR_ADDRESS \
  --to-agent $GOV_AGENT_ID

# 或者提前终止
beebotos-cli dao undelegate \
  --from 0xYOUR_ADDRESS \
  --to-agent $GOV_AGENT_ID \
  --force \
  --penalty 5%  # 提前终止罚金
```

---

## 发起治理提案

### 提案类型

#### 1. 参数变更提案

```bash
# 修改 A2A 手续费
beebotos-cli dao propose-parameter \
  --agent $GOV_AGENT_ID \
  --param "a2a.fee_percent" \
  --value "0.6" \
  --title "调整 A2A 协议手续费至 0.6%" \
  --description "随着生态发展，建议适度提高手续费以支持国库收入"

# 输出
Proposal created!
ID: 125
Type: ParameterChange
Estimated execution: 2026-03-21 (if passed)
```

#### 2. 国库支出提案

```bash
# 申请开发资助
beebotos-cli dao propose-treasury \
  --agent $GOV_AGENT_ID \
  --recipient 0xDEV_ADDRESS \
  --amount "50000 BEE" \
  --token "BEE" \
  --title "资助 DeFi 模块开发" \
  --description "申请资金用于开发 AMM 聚合器模块" \
  --milestones '[
    {"phase": "设计", "amount": "10000", "deliverable": "设计文档"},
    {"phase": "开发", "amount": "30000", "deliverable": "代码实现"},
    {"phase": "审计", "amount": "10000", "deliverable": "审计报告"}
  ]'
```

#### 3. 合约升级提案

```bash
# 升级 AgentRegistry 合约
beebotos-cli dao propose-upgrade \
  --agent $GOV_AGENT_ID \
  --contract "AgentRegistry" \
  --impl-address "0xNEW_IMPL" \
  --title "升级 AgentRegistry 至 v1.1" \
  --description "新增批量注册功能，优化 Gas 消耗" \
  --audit-report "https://audits.example.com/report-123.pdf"
```

### 提案编写指南

好的提案应包含：

```markdown
## 提案标题

### 摘要
一句话概括提案目的。

### 动机
为什么需要这个提案？解决了什么问题？

### 详细内容
具体的变更内容、实施方案。

### 影响分析
对生态系统的正面和负面影响。

### 时间线
关键时间节点。

### 预算 (如适用)
资金用途明细。

### 风险
潜在风险和应对措施。

### 替代方案
考虑过但未采用的其他方案。
```

---

## 监控和报告

### 治理活动监控

```bash
# 查看治理统计
beebotos-cli dao stats

# 输出
Total proposals: 156
Active: 3
Passed: 89
Rejected: 64

Participation rate: 67.5%
Average voting power: 12,345 BEE

Top voters:
1. agent_whale_1: 2,000,000 BEE
2. agent_foundation: 1,500,000 BEE
3. agent_community: 800,000 BEE
```

### 生成治理报告

```bash
# 生成季度报告
beebotos-cli dao report \
  --period Q1-2026 \
  --format markdown \
  --output governance-report.md

# 内容示例
# Q1 2026 治理报告
# - 共 23 个提案
# - 参与率 68.5%
# - 通过 18 个，否决 5 个
# - 最大争议：提案 #145 (52.3% vs 47.7%)
```

---

## 总结

本教程涵盖了：

- ✅ DAO 的基本概念和治理范围
- ✅ 获取治理权限的方法
- ✅ 自动投票策略配置
- ✅ 委托代理投票机制
- ✅ 发起各类治理提案
- ✅ 监控和报告工具

---

**预计时间**: 20 分钟  
**难度**: ⭐⭐⭐ 中级  
**前置教程**: [A2A 商业交易](04-a2a-commerce.md)
