
## beebotos-gateway 编译和使用指南

**beebotos-gateway** 是 BeeBotOS 的 **API 网关服务**，提供 HTTP REST API 和 WebSocket 实时通信，是前端和 CLI 与后端服务交互的统一入口。

---

### 📦 编译命令

#### 1. 编译整个项目（包含 gateway）
```bash
# 项目根目录
cargo build --release

# 编译后的可执行文件
./target/release/beebotos-gateway
```

#### 2. 只编译 Gateway
```bash
# 编译 beebotos-gateway 包
cargo build --release -p beebotos-gateway

# 或指定二进制文件
cargo build --release --bin beebotos-gateway
```

#### 3. 安装到系统 PATH
```bash
# 安装到 ~/.cargo/bin
cargo install --path apps/gateway

# 现在可以直接使用
beebotos-gateway
```

#### 4. 开发调试模式
```bash
# 快速编译（无优化，用于开发）
cargo build -p beebotos-gateway

# 运行
cargo run -p beebotos-gateway
```

---

### 🚀 使用方法

#### 启动服务

```bash
# 默认启动（端口 8080）
./target/release/beebotos-gateway

# 或使用 cargo
cargo run -p beebotos-gateway
```

#### 环境变量配置

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `PORT` | 服务端口 | `8080` |
| `DATABASE_URL` | 数据库连接 | `postgres://localhost/beebot` |
| `REDIS_URL` | Redis 连接 | `redis://localhost` |
| `RUST_LOG` | 日志级别 | `info` |

**示例：**
```bash
# 指定端口启动
PORT=3000 ./target/release/beebotos-gateway

# 或带日志级别
RUST_LOG=debug cargo run -p beebotos-gateway
```

---

### 📋 API 端点清单

#### 🏥 **健康检查**

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 健康状态 |
| GET | `/status` | 系统状态 |

**示例：**
```bash
curl http://localhost:8080/health
# {"status": "healthy"}

curl http://localhost:8080/status
# {"version": "1.0.0", "status": "operational"}
```

---

#### 🤖 **Agent 管理** (`/api/v1/agents`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/agents` | 列出所有 Agent |
| POST | `/api/v1/agents` | 创建新 Agent |
| GET | `/api/v1/agents/:id` | 获取 Agent 详情 |
| DELETE | `/api/v1/agents/:id` | 删除 Agent |
| POST | `/api/v1/agents/:id/start` | 启动 Agent |
| POST | `/api/v1/agents/:id/stop` | 停止 Agent |
| POST | `/api/v1/agents/:id/spawn` | 创建子 Agent |

**示例：**
```bash
# 列出所有 Agent
curl http://localhost:8080/api/v1/agents

# 创建新 Agent
curl -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "MyAssistant",
    "description": "智能助手",
    "capabilities": ["chat", "code"],
    "model": {"provider": "openai", "model": "gpt-4"}
  }'

# 获取 Agent 详情
curl http://localhost:8080/api/v1/agents/agent-001

# 删除 Agent
curl -X DELETE http://localhost:8080/api/v1/agents/agent-001

# 启动/停止 Agent
curl -X POST http://localhost:8080/api/v1/agents/agent-001/start
curl -X POST http://localhost:8080/api/v1/agents/agent-001/stop

# 创建子 Agent
curl -X POST http://localhost:8080/api/v1/agents/agent-001/spawn \
  -d '{"goal": "完成数据分析任务"}'
```

---

#### 🧠 **认知功能** (`/api/v1/brain`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/brain/:id/emotion` | 情绪状态 |
| GET | `/api/v1/brain/:id/personality` | 人格特质 |
| POST | `/api/v1/brain/:id/memory` | 查询记忆 |
| GET | `/api/v1/brain/:id/neural` | 神经网络状态 |
| POST | `/api/v1/brain/:id/evolve` | 触发进化 |

---

#### 🏛️ **DAO 治理** (`/api/v1/dao`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/dao/proposals` | 列出提案 |
| POST | `/api/v1/dao/proposals` | 创建提案 |
| GET | `/api/v1/dao/proposals/:id` | 获取提案详情 |
| POST | `/api/v1/dao/vote` | 投票 |
| GET | `/api/v1/dao/voting-power/:address` | 查询投票权 |
| GET | `/api/v1/dao/treasury` | 金库状态 |

**示例：**
```bash
# 列出提案
curl http://localhost:8080/api/v1/dao/proposals

# 创建提案
curl -X POST http://localhost:8080/api/v1/dao/proposals \
  -H "Content-Type: application/json" \
  -d '{
    "title": "增加新功能",
    "description": "建议添加自动备份",
    "actions": [{"target": "0x...", "value": "0", "calldata": "0x..."}]
  }'

# 投票
curl -X POST http://localhost:8080/api/v1/dao/vote \
  -d '{"proposal_id": 42, "vote_type": "for"}'

# 查询投票权
curl http://localhost:8080/api/v1/dao/voting-power/0x1234...

# 查看金库
curl http://localhost:8080/api/v1/dao/treasury
```

---

#### ⛓️ **区块链** (`/api/v1/chain`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/chain/balance/:address` | 查询余额 |
| GET | `/api/v1/chain/tx/:hash` | 查询交易 |
| POST | `/api/v1/chain/tx` | 发送交易 |

---

#### 🛠️ **技能管理** (`/api/v1/skills`)

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/skills` | 列出技能 |
| POST | `/api/v1/skills` | 上传技能 |
| GET | `/api/v1/skills/:id` | 获取技能详情 |
| POST | `/api/v1/skills/:id/execute` | 执行技能 |

---

### 🔌 WebSocket 实时通信

WebSocket 端点：`ws://localhost:8080/ws`

**支持的事件：**
- Agent 状态更新
- 任务进度推送
- DAO 投票通知
- 系统事件广播

**示例（使用 wscat）：**
```bash
npm install -g wscat
wscat -c ws://localhost:8080/ws

# 发送订阅消息
> {"type": "subscribe", "channel": "agents"}

# 接收实时更新
< {"type": "agent_update", "agent_id": "...", "status": "running"}
```

---

### 🛠 技术栈

| 组件 | 用途 |
|------|------|
| **axum** | Web 框架 |
| **tokio** | 异步运行时 |
| **tower-http** | CORS、日志、压缩中间件 |
| **jsonwebtoken** | JWT 认证 |
| **serde** | JSON 序列化 |
| **uuid** | 唯一 ID 生成 |

---

### 📁 项目结构

```
apps/gateway/
├── Cargo.toml
└── src/
    ├── main.rs           # 服务入口
    ├── lib.rs            # 库 API（备用路由）
    ├── handlers/
    │   ├── mod.rs        # handlers 入口
    │   ├── http/         # HTTP REST API
    │   │   ├── mod.rs
    │   │   ├── agents.rs # Agent 管理
    │   │   ├── brain.rs  # 认知功能
    │   │   ├── chain.rs  # 区块链
    │   │   ├── dao.rs    # DAO 治理
    │   │   └── skills.rs # 技能管理
    │   └── websocket/    # WebSocket 处理
    │       └── mod.rs
    ├── auth.rs           # 认证逻辑
    ├── middleware.rs     # 自定义中间件
    ├── error.rs          # 错误处理
    ├── health.rs         # 健康检查
    └── websocket.rs      # WebSocket 管理
```

---

### ⚠️ 注意事项

1. **当前为 Stub 实现** - API 已定义但大部分返回模拟数据
2. **CORS 已开启** - 默认允许所有来源，生产环境需配置白名单
3. **无持久化** - 当前数据存储在内存中，重启后丢失
4. **认证待实现** - JWT 结构已配置但未启用验证

需要我帮你实现完整的 API 逻辑或添加数据库支持吗？

