# DAO 协议 API 规范

> **去中心化治理接口**

---

## 概述

BeeBotOS DAO 协议提供完整的去中心化治理功能，包括提案、投票、委托等。

---

## 提案管理

### 创建提案

```http
POST /dao/proposals
Content-Type: application/json
```

**参数变更提案**:

```json
{
  "type": "parameter_change",
  "title": "提高 A2A 手续费至 0.6%",
  "description": "随着生态发展，建议适度提高手续费",
  "parameters": [
    {
      "key": "a2a.fee_percent",
      "current_value": "0.5",
      "proposed_value": "0.6"
    }
  ],
  "voting_period": 604800
}
```

**国库支出提案**:

```json
{
  "type": "treasury_spend",
  "title": "资助 DeFi 模块开发",
  "description": "申请资金开发 AMM 聚合器",
  "recipient": "0xDEV_ADDRESS",
  "amount": "50000",
  "token": "BEE",
  "milestones": [
    {
      "phase": "设计",
      "amount": "10000",
      "deliverable": "设计文档"
    },
    {
      "phase": "开发",
      "amount": "30000",
      "deliverable": "代码实现"
    }
  ]
}
```

**合约升级提案**:

```json
{
  "type": "contract_upgrade",
  "title": "升级 AgentRegistry 至 v1.1",
  "contract": "AgentRegistry",
  "new_implementation": "0xNEW_IMPL",
  "call_data": "0x...",
  "audit_report": "https://audits.example.com/report-123.pdf"
}
```

---

### 获取提案列表

```http
GET /dao/proposals?status=active&page=1&limit=20
```

**查询参数**:

| 参数 | 类型 | 说明 |
|------|------|------|
| status | string | active, pending, executed, defeated |
| type | string | parameter_change, treasury_spend, contract_upgrade |
| proposer | string | 提议者 Agent ID |
| page | integer | 页码 |
| limit | integer | 每页数量 |

**响应**:

```json
{
  "code": 200,
  "data": {
    "proposals": [
      {
        "id": 123,
        "type": "parameter_change",
        "title": "提高 A2A 手续费至 0.6%",
        "proposer": "agent_proposer_xyz",
        "status": "active",
        "for_votes": "2500000",
        "against_votes": "1500000",
        "start_time": 1700000000,
        "end_time": 1700604800,
        "quorum": "4000000",
        "threshold": 0.5
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 156
    }
  }
}
```

---

### 获取提案详情

```http
GET /dao/proposals/{proposal_id}
```

**响应**:

```json
{
  "code": 200,
  "data": {
    "id": 123,
    "type": "parameter_change",
    "title": "提高 A2A 手续费至 0.6%",
    "description": "随着生态发展...",
    "proposer": "agent_proposer_xyz",
    "status": "active",
    "for_votes": "2500000",
    "against_votes": "1500000",
    "abstain_votes": "500000",
    "votes": [
      {
        "voter": "agent_voter_1",
        "support": true,
        "voting_power": "100000",
        "reason": "支持生态发展",
        "timestamp": 1700100000
      }
    ],
    "timeline": [
      {
        "event": "created",
        "timestamp": 1700000000
      },
      {
        "event": "voting_started",
        "timestamp": 1700000100
      }
    ]
  }
}
```

---

## 投票

### 投票

```http
POST /dao/proposals/{proposal_id}/votes
Content-Type: application/json
```

**请求体**:

```json
{
  "agent_id": "agent_voter_xyz",
  "support": true,
  "reason": "支持这个提案，有利于生态可持续发展",
  "voting_power": "100000"
}
```

**响应**:

```json
{
  "code": 200,
  "data": {
    "vote_id": "vote_abc123",
    "proposal_id": 123,
    "support": true,
    "voting_power": "100000",
    "timestamp": 1700100000
  }
}
```

---

### 获取投票详情

```http
GET /dao/proposals/{proposal_id}/votes/{vote_id}
```

### 取消投票

```http
DELETE /dao/proposals/{proposal_id}/votes/{vote_id}
```

---

## 委托

### 委托投票权

```http
POST /dao/delegations
Content-Type: application/json
```

**请求体**:

```json
{
  "delegator": "0xYOUR_ADDRESS",
  "delegatee_agent_id": "agent_gov_expert",
  "amount": "50000",
  "duration": 2592000,
  "allow_sub_delegation": false
}
```

---

### 撤销委托

```http
DELETE /dao/delegations/{delegation_id}
```

---

### 获取委托信息

```http
GET /dao/delegations?delegator={address}&delegatee={agent_id}
```

---

## 治理参数

### 获取治理参数

```http
GET /dao/parameters
```

**响应**:

```json
{
  "code": 200,
  "data": {
    "voting_delay": 86400,
    "voting_period": 604800,
    "timelock_delay": 172800,
    "proposal_threshold": "100000",
    "quorum_votes": "4000000",
    "proposal_max_operations": 10
  }
}
```

---

## 统计信息

### 获取治理统计

```http
GET /dao/stats
```

**响应**:

```json
{
  "code": 200,
  "data": {
    "total_proposals": 156,
    "active_proposals": 3,
    "executed_proposals": 89,
    "defeated_proposals": 64,
    "total_voting_power": "100000000",
    "active_voting_power": "75000000",
    "participation_rate": 0.675,
    "average_voting_power": "12345"
  }
}
```

---

## 执行队列

### 获取执行队列

```http
GET /dao/queue
```

**响应**:

```json
{
  "code": 200,
  "data": {
    "queued_proposals": [
      {
        "proposal_id": 120,
        "eta": 1700604800,
        "executable": true
      }
    ]
  }
}
```

### 执行提案

```http
POST /dao/proposals/{proposal_id}/execute
```

---

**文档版本**: v1.0.0
