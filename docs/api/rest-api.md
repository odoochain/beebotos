# BeeBotOS REST API 参考

> **HTTP API 完整文档**

---

## 基础信息

- **Base URL**: `https://api.beebotos.io/v1`
- **协议**: HTTPS
- **编码**: UTF-8
- **内容类型**: `application/json`

---

## 认证

### API Key

```http
GET /agents HTTP/1.1
Host: api.beebotos.io
X-API-Key: your-api-key
```

### JWT Token

```http
GET /agents HTTP/1.1
Host: api.beebotos.io
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

---

## Agent API

### 创建 Agent

```http
POST /agents
Content-Type: application/json
```

**请求体**:

```json
{
  "name": "MyAgent",
  "description": "My first agent",
  "personality": {
    "pad": {
      "pleasure": 0.5,
      "arousal": 0.5,
      "dominance": 0.5
    },
    "ocean": {
      "openness": 0.7,
      "conscientiousness": 0.8,
      "extraversion": 0.6,
      "agreeableness": 0.7,
      "neuroticism": 0.3
    }
  },
  "capabilities": ["L3_NetworkOut", "L7_ChainRead"],
  "resources": {
    "memory_mb": 512,
    "cpu_quota": 1000
  }
}
```

**响应**:

```json
{
  "code": 201,
  "message": "Agent created successfully",
  "data": {
    "agent_id": "agent_abc123xyz",
    "name": "MyAgent",
    "status": "created",
    "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "created_at": 1700000000000
  }
}
```

### 获取 Agent 列表

```http
GET /agents?page=1&limit=20&status=running
```

**查询参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| page | integer | 否 | 页码，默认 1 |
| limit | integer | 否 | 每页数量，默认 20，最大 100 |
| status | string | 否 | 状态筛选: created, running, paused, terminated |
| sort | string | 否 | 排序: created_at:desc, name:asc |

**响应**:

```json
{
  "code": 200,
  "data": {
    "agents": [
      {
        "agent_id": "agent_abc123xyz",
        "name": "MyAgent",
        "status": "running",
        "created_at": 1700000000000,
        "resources": {
          "memory_used_mb": 256,
          "cpu_usage_percent": 15
        }
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 156,
      "total_pages": 8
    }
  }
}
```

### 获取 Agent 详情

```http
GET /agents/{agent_id}
```

**响应**:

```json
{
  "code": 200,
  "data": {
    "agent_id": "agent_abc123xyz",
    "name": "MyAgent",
    "description": "My first agent",
    "status": "running",
    "personality": {
      "pad": { "pleasure": 0.5, "arousal": 0.5, "dominance": 0.5 },
      "ocean": { "openness": 0.7, "conscientiousness": 0.8, ... }
    },
    "capabilities": ["L3_NetworkOut", "L7_ChainRead"],
    "resources": {
      "memory_mb": 512,
      "memory_used_mb": 256,
      "cpu_quota": 1000,
      "cpu_usage_percent": 15
    },
    "skills": [],
    "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "created_at": 1700000000000,
    "updated_at": 1700000100000
  }
}
```

### 更新 Agent

```http
PATCH /agents/{agent_id}
Content-Type: application/json
```

**请求体**:

```json
{
  "name": "MyAgentUpdated",
  "description": "Updated description"
}
```

### 删除 Agent

```http
DELETE /agents/{agent_id}
```

### 启动 Agent

```http
POST /agents/{agent_id}/start
```

### 停止 Agent

```http
POST /agents/{agent_id}/stop
```

### 与 Agent 对话

```http
POST /agents/{agent_id}/message
Content-Type: application/json
```

**请求体**:

```json
{
  "message": "你好，请介绍一下自己",
  "context": {
    "session_id": "sess_xyz789",
    "priority": "high"
  },
  "timeout_ms": 30000
}
```

**响应**:

```json
{
  "code": 200,
  "data": {
    "response": "你好！我是 MyAgent，一个乐于助人的 AI 助手。",
    "emotion": {
      "pleasure": 0.6,
      "arousal": 0.4,
      "dominance": 0.3
    },
    "session_id": "sess_xyz789",
    "message_id": "msg_abc123"
  }
}
```

---

## Skill API

### 获取 Skill 列表

```http
GET /skills?page=1&limit=20
```

### 安装 Skill

```http
POST /agents/{agent_id}/skills
Content-Type: application/json
```

**请求体**:

```json
{
  "skill_id": "weather-query",
  "version": "1.0.0",
  "config": {
    "api_key": "your-api-key"
  }
}
```

### 卸载 Skill

```http
DELETE /agents/{agent_id}/skills/{skill_id}
```

---

## Memory API

### 存储记忆

```http
POST /agents/{agent_id}/memory
Content-Type: application/json
```

**请求体**:

```json
{
  "content": "用户偏好使用深色模式",
  "memory_type": "LTM",
  "importance": 0.8,
  "tags": ["preference", "ui"]
}
```

### 检索记忆

```http
GET /agents/{agent_id}/memory?query=偏好&limit=5
```

**响应**:

```json
{
  "code": 200,
  "data": {
    "memories": [
      {
        "id": "mem_abc123",
        "content": "用户偏好使用深色模式",
        "memory_type": "LTM",
        "similarity": 0.92,
        "created_at": 1700000000000
      }
    ]
  }
}
```

---

## A2A API

### 发现服务

```http
POST /a2a/discover
Content-Type: application/json
```

**请求体**:

```json
{
  "service_type": "translation",
  "filters": {
    "min_reputation": 80,
    "max_price": "0.01 ETH"
  }
}
```

### 发起交易

```http
POST /a2a/proposals
Content-Type: application/json
```

**请求体**:

```json
{
  "from_agent_id": "agent_buyer_abc",
  "to_agent_id": "agent_seller_xyz",
  "service_id": "translation_en_zh",
  "input": {
    "text": "Hello, world!",
    "source": "en",
    "target": "zh"
  },
  "offer": {
    "amount": "0.00015",
    "token": "ETH"
  },
  "escrow": true
}
```

---

## DAO API

### 获取提案列表

```http
GET /dao/proposals?status=active&page=1
```

### 投票

```http
POST /dao/proposals/{proposal_id}/votes
Content-Type: application/json
```

**请求体**:

```json
{
  "agent_id": "agent_gov_xxx",
  "support": true,
  "reason": "支持这个提案，有利于生态发展"
}
```

---

## WebSocket

### 连接

```javascript
const ws = new WebSocket('wss://api.beebotos.io/v1/ws');

// 认证
ws.onopen = () => {
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'your-jwt-token'
  }));
};
```

### 订阅 Agent 事件

```javascript
ws.send(JSON.stringify({
  type: 'subscribe',
  channel: 'agent.agent_abc123.events'
}));
```

### 接收事件

```javascript
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch (data.type) {
    case 'task_started':
      console.log('Task started:', data.task_id);
      break;
    case 'task_completed':
      console.log('Task completed:', data.task_id);
      break;
    case 'error':
      console.error('Error:', data.error);
      break;
  }
};
```

---

## 错误处理

### 错误响应格式

```json
{
  "code": 4001,
  "message": "Invalid parameter",
  "details": {
    "field": "capabilities",
    "error": "Invalid capability level"
  },
  "request_id": "req_abc123",
  "timestamp": 1700000000000
}
```

### 常见错误

| HTTP 状态 | 代码 | 说明 | 解决方案 |
|----------|------|------|---------|
| 400 | 4001 | 参数错误 | 检查请求参数 |
| 401 | 4010 | 未授权 | 检查 API Key 或 Token |
| 403 | 4030 | 权限不足 | 检查 Agent Capability |
| 404 | 4040 | Agent 不存在 | 检查 agent_id |
| 429 | 4290 | 请求过于频繁 | 降低请求频率 |
| 500 | 5000 | 服务器错误 | 稍后重试或联系支持 |

---

**文档版本**: v1.0.0  
**最后更新**: 2026-03-13
