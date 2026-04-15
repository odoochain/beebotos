# BeeBotOS 认证与授权

> **完整的认证和授权指南**

---

## 概述

BeeBotOS 提供多种认证方式，以满足不同场景的安全需求。

| 方式 | 适用场景 | 安全等级 |
|------|---------|---------|
| API Key | 开发测试 | ⭐⭐ |
| JWT Token | 生产应用 | ⭐⭐⭐⭐ |
| Web3 Signature | Web3 应用 | ⭐⭐⭐⭐⭐ |
| OAuth 2.0 | 第三方集成 | ⭐⭐⭐⭐ |

---

## API Key

### 获取 API Key

1. 登录 BeeBotOS 控制台
2. 进入 Settings > API Keys
3. 点击 "Create New Key"
4. 复制并保存 Key (只显示一次)

### 使用 API Key

```bash
# HTTP Header
curl -H "X-API-Key: bk_live_abc123xyz" \
  https://api.beebotos.io/v1/agents
```

```javascript
const client = new BeeBotOS({
  apiKey: 'bk_live_abc123xyz'
});
```

### API Key 类型

| 类型 | 前缀 | 用途 | 限制 |
|------|------|------|------|
| 测试 Key | `bk_test_` | 开发测试 | 限制频率 |
| 生产 Key | `bk_live_` | 生产环境 | 按套餐限制 |
| 只读 Key | `bk_read_` | 只读访问 | 无写入权限 |

---

## JWT Token

### 获取 Token

**方式 1: 钱包签名**

```bash
curl -X POST https://api.beebotos.io/v1/auth/challenge \
  -H "Content-Type: application/json" \
  -d '{
    "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
  }'

# 响应: {"challenge": "Sign this message to authenticate: 0xabc..."}

# 使用钱包签名 challenge 后
curl -X POST https://api.beebotos.io/v1/auth/verify \
  -H "Content-Type: application/json" \
  -d '{
    "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "signature": "0x..."
  }'
```

**响应**:

```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

**方式 2: 邮箱+密码**

```bash
curl -X POST https://api.beebotos.io/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "your-password"
  }'
```

### 使用 Token

```bash
# Authorization Header
curl -H "Authorization: Bearer eyJhbG..." \
  https://api.beebotos.io/v1/agents
```

```javascript
const client = new BeeBotOS({
  accessToken: 'eyJhbG...'
});
```

### 刷新 Token

```bash
curl -X POST https://api.beebotos.io/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{
    "refresh_token": "eyJhbG..."
  }'
```

---

## Web3 签名认证

### 完整流程

```javascript
import { ethers } from 'ethers';
import { BeeBotOS } from '@beebotos/sdk';

async function authenticateWithWeb3() {
  // 1. 获取 challenge
  const challenge = await fetch('https://api.beebotos.io/v1/auth/challenge', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ 
      address: await signer.getAddress() 
    })
  }).then(r => r.json());
  
  // 2. 签名
  const signature = await signer.signMessage(challenge.message);
  
  // 3. 验证
  const auth = await fetch('https://api.beebotos.io/v1/auth/verify', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ 
      address: await signer.getAddress(),
      signature 
    })
  }).then(r => r.json());
  
  // 4. 使用 Token
  const client = new BeeBotOS({ accessToken: auth.access_token });
  
  return client;
}
```

### EIP-4361 (Sign-In with Ethereum)

BeeBotOS 支持 SIWE 标准：

```javascript
import { SiweMessage } from 'siwe';

const message = new SiweMessage({
  domain: 'api.beebotos.io',
  address: await signer.getAddress(),
  statement: 'Sign in to BeeBotOS',
  uri: 'https://beebotos.io',
  version: '1',
  chainId: 1,
  nonce: await fetchNonce()
});

const signature = await signer.signMessage(message.prepareMessage());
```

---

## OAuth 2.0

### 支持的平台

- GitHub
- Google
- Twitter

### 授权流程

```
1. 重定向用户到授权页面
   GET https://api.beebotos.io/v1/auth/oauth/github
   
2. 用户授权后重定向到回调 URL
   ?code=xxx&state=yyy
   
3. 使用 code 换取 Token
   POST /v1/auth/oauth/token
   
4. 获取到 access_token
```

### 实现示例

```javascript
// 1. 跳转授权
window.location.href = 'https://api.beebotos.io/v1/auth/oauth/github' +
  '?client_id=your-app-id' +
  '&redirect_uri=https://your-app.com/callback' +
  '&state=random-state';

// 2. 回调处理
app.get('/callback', async (req, res) => {
  const { code, state } = req.query;
  
  // 3. 换取 Token
  const token = await fetch('https://api.beebotos.io/v1/auth/oauth/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      grant_type: 'authorization_code',
      code,
      client_id: 'your-app-id',
      client_secret: 'your-app-secret'
    })
  }).then(r => r.json());
  
  // 保存 token
  saveToken(token.access_token);
});
```

---

## 权限控制

### Capability 权限

Agent 的权限通过 Capability 系统控制：

```yaml
# 创建 Agent 时指定权限
capabilities:
  - L0_LocalCompute    # 基础计算
  - L3_NetworkOut      # 网络访问
  - L7_ChainRead       # 区块链读取
```

### API 权限检查

```bash
# 检查当前权限
curl https://api.beebotos.io/v1/auth/permissions \
  -H "Authorization: Bearer eyJhbG..."

# 响应
{
  "capabilities": ["L0", "L3", "L7"],
  "rate_limit": {
    "requests_per_minute": 1000,
    "remaining": 987
  }
}
```

### 权限不足错误

```json
{
  "code": 4030,
  "message": "Insufficient capability",
  "details": {
    "required": "L8_ChainWriteLow",
    "current": ["L0", "L3", "L7"]
  }
}
```

---

## 安全最佳实践

### 1. 密钥管理

```bash
# 不要硬编码密钥
# ❌ 错误
const apiKey = 'bk_live_abc123';

# ✅ 正确
const apiKey = process.env.BEEBOTOS_API_KEY;
```

### 2. 使用 HTTPS

```bash
# 始终使用 HTTPS
# ❌ 错误
curl http://api.beebotos.io/v1/agents

# ✅ 正确
curl https://api.beebotos.io/v1/agents
```

### 3. Token 过期处理

```javascript
class BeeBotOSClient {
  async request(config) {
    try {
      return await this.axios(config);
    } catch (error) {
      if (error.response?.status === 401) {
        // Token 过期，刷新
        await this.refreshToken();
        return this.axios(config);
      }
      throw error;
    }
  }
}
```

### 4. 最小权限原则

```yaml
# 只授予必要的权限
# ❌ 过多权限
capabilities: [L0, L1, L2, L3, L4, L5, L6, L7, L8, L9]

# ✅ 最小权限
capabilities: [L0, L3, L7]
```

---

## 常见问题

### Q: Token 有效期多久？

**A**: 
- Access Token: 1 小时
- Refresh Token: 30 天
- API Key: 无过期 (可手动撤销)

### Q: 如何撤销泄露的 Key？

**A**:
```bash
# 控制台撤销
# 或通过 API
 curl -X DELETE https://api.beebotos.io/v1/auth/keys/{key_id} \
   -H "Authorization: Bearer eyJhbG..."
```

### Q: 支持多设备登录吗？

**A**: 支持，每个设备独立的 Token。

---

**文档版本**: v1.0.0  
**最后更新**: 2026-03-13
