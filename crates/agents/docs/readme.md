
## beebotos-agents 编译和使用指南

**beebotos-agents** 是 BeeBotOS 的 **Agent 运行时**，提供完整的自主智能体功能，包括 A2A 协议、MCP 集成、任务调度、子代理创建等。

---

### 📦 编译命令

#### 1. 编译整个项目（包含 agents）
```bash
# 项目根目录
cargo build --release

# 编译后的库
# target/release/libbeebotos_agents.rlib
```

#### 2. 只编译 Agents crate
```bash
# 编译 beebotos-agents
cargo build --release -p beebotos-agents

# 调试模式
cargo build -p beebotos-agents
```

#### 3. 运行测试
```bash
# 运行单元测试
cargo test -p beebotos-agents

# 带日志输出
RUST_LOG=debug cargo test -p beebotos-agents -- --nocapture
```

#### 4. Feature 标志编译
```bash
# 默认特性（WASM 运行时）
cargo build -p beebotos-agents

# 启用 A2A 服务器功能（需要 axum）
cargo build -p beebotos-agents --features a2a-server

# 禁用 WASM 运行时
cargo build -p beebotos-agents --no-default-features
```

---

### 🚀 使用方法

#### 作为库依赖

在 `Cargo.toml` 中添加：
```toml
[dependencies]
beebotos-agents = { path = "crates/agents" }

# 或使用特定特性
beebotos-agents = { path = "crates/agents", features = ["a2a-server"] }
```

---

### 💻 编程示例

#### 1. 创建基础 Agent

```rust
use beebotos_agents::{AgentBuilder, Agent, Task};
use beebotos_agents::a2a::A2AClient;
use beebotos_agents::mcp::MCPManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 方法1: 使用 Builder 模式
    let mut agent = AgentBuilder::new("Assistant")
        .description("A helpful AI assistant")
        .with_capability("chat")
        .with_capability("code")
        .with_model("openai", "gpt-4")
        .build();

    // 初始化 Agent
    agent.initialize().await?;
    
    println!("Agent initialized! State: {:?}", agent.get_state());
    
    // 执行任务
    let task = Task {
        id: "task-001".to_string(),
        task_type: "chat".to_string(),
        input: "Hello!".to_string(),
        parameters: Default::default(),
    };
    
    let result = agent.execute_task(task).await?;
    println!("Task result: {}", result.output);
    
    Ok(())
}
```

---

#### 2. 配置 A2A 协议（Agent-to-Agent）

```rust
use beebotos_agents::a2a::A2AClient;
use beebotos_agents::a2a::message::{A2AMessage, MessageType, MessagePriority};

// 创建 A2A 客户端
let a2a_client = A2AClient::new().expect("Failed to create A2A client");

// 查找其他 Agent
let agents = a2a_client.discovery().list_agents().await;
println!("Available agents: {:?}", agents);

// 发送消息给其他 Agent
use chrono::Utc;

let message = A2AMessage {
    id: uuid::Uuid::new_v4().to_string(),
    msg_type: MessageType::Request,
    priority: MessagePriority::High,
    from: beebotos_agents::types::AgentId::from_string("my-agent"),
    to: Some(beebotos_agents::types::AgentId::from_string("other-agent")),
    payload: beebotos_agents::a2a::message::MessagePayload::Request {
        action: "help".to_string(),
        params: serde_json::json!({"task": "analyze"}).as_object().unwrap().clone(),
    },
    timestamp: Utc::now(),
    ttl: Some(300),
    signature: None, // 🟠 HIGH FIX: signature is Option<Vec<u8>>
};

let response = a2a_client.send_message(message, "other-agent").await?;
println!("Response: {:?}", response);
```

---

#### 3. 创建和调度子 Agent（Non-blocking Spawn）

```rust
use beebotos_agents::spawning::{SpawnEngine, SpawnConfig, SpawnResult};
use beebotos_agents::spawning::nonblocking::SpawnStatus;

// 创建子 Agent
let spawn_config = SpawnConfig {
    parent_id: "parent-agent".to_string(),
    name: "DataProcessor".to_string(),
    capabilities: vec!["data-analysis".to_string()],
    resources: ResourceQuota {
        cpu_percent: 50.0,
        memory_mb: 512,
        max_tasks: 10,
    },
};

// 非阻塞创建 - 立即返回
let spawn_result: SpawnResult = SpawnEngine::spawn(spawn_config).await;

match spawn_result.status {
    SpawnStatus::Accepted { estimated_time_ms } => {
        println!("Spawn accepted! ETA: {}ms", estimated_time_ms);
        println!("Child session key: {}", spawn_result.session_key);
    }
    SpawnStatus::Queued { position } => {
        println!("Queued at position: {}", position);
    }
    SpawnStatus::Rejected { reason } => {
        println!("Spawn rejected: {}", reason);
    }
}

// 监控子 Agent 状态
let monitor = spawn_result.monitor;
while let Some(update) = monitor.recv().await {
    println!("Spawn update: {:?}", update);
    if update.is_complete() {
        break;
    }
}
```

---

#### 4. 使用 MCP（Model Context Protocol）

```rust
use beebotos_agents::mcp::{MCPManager, MCPClient, Tool};

// 创建 MCP 管理器
let mut mcp_manager = MCPManager::new();

// 添加 MCP 服务器
mcp_manager.add_server("filesystem", MCPClient::new("http://localhost:3001"));
mcp_manager.add_server("database", MCPClient::new("http://localhost:3002"));

// 初始化所有 MCP 连接
mcp_manager.initialize_all().await?;

// 发现可用工具
let tools: Vec<Tool> = mcp_manager.discover_tools().await;
for tool in tools {
    println!("Tool: {} - {}", tool.name, tool.description);
}

// 执行工具调用
let result = mcp_manager
    .call_tool("filesystem", "read_file", serde_json::json!({"path": "data/tmp/test.txt"}))
    .await?;
println!("Tool result: {}", result);
```

---

#### 5. 任务调度和队列管理

```rust
use beebotos_agents::queue::manager::QueueManager;
use beebotos_agents::queue::cron::CronQueue;
use beebotos_agents::scheduling::{HeartbeatScheduler, CronScheduler};

// 创建队列管理器
let queue_manager = QueueManager::new();

// 添加主任务队列
queue_manager.create_main_queue("default").await;

// 添加 Cron 调度队列
let cron_queue = CronQueue::new();
cron_queue.add_job("0 */5 * * * *", || async {
    println!("Running scheduled task every 5 minutes");
}).await;

// 心跳调度
let heartbeat = HeartbeatScheduler::new();
heartbeat.start_interval(std::time::Duration::from_secs(30), || async {
    println!("Heartbeat check");
}).await;
```

---

#### 6. 会话管理和隔离

```rust
use beebotos_agents::session::{SessionKey, SessionType, SessionContext};
use beebotos_agents::session::isolation::IsolationConfig;

// 创建隔离会话
let session_key = SessionKey::generate();
let isolation = IsolationConfig {
    namespace: "agent-session".to_string(),
    resource_limits: ResourceLimits {
        cpu_quota: 100000,
        memory_limit: 1073741824, // 1GB
    },
    network_policy: NetworkPolicy::Restricted,
};

let context = SessionContext::new(session_key.clone(), SessionType::Isolated(isolation));

// 在隔离环境中执行任务
let result = context.execute(async {
    // 任务逻辑
    "Task completed in isolated environment"
}).await;

// 持久化会话
let persistence = SessionPersistence::new();
persistence.save(&session_key, &context).await?;
```

---

### 📋 核心功能模块

| 模块 | 路径 | 功能 |
|------|------|------|
| **a2a** | `src/a2a/` | Agent-to-Agent 通信协议 |
| **mcp** | `src/mcp/` | Model Context Protocol 集成 |
| **spawning** | `src/spawning/` | 子 Agent 非阻塞创建 |
| **scheduling** | `src/scheduling/` | 定时任务和心跳调度 |
| **queue** | `src/queue/` | 多队列并发管理 |
| **session** | `src/session/` | 会话隔离和持久化 |
| **skills** | `src/skills/` | WASM 技能加载执行 |
| **runtime** | `src/runtime/` | Agent 运行时环境 |
| **memory** | `src/memory/` | 记忆系统（QMD）|
| **models** | `src/models/` | LLM 提供商路由 |
| **communication** | `src/communication/` | 多平台通信（Telegram/Discord 等）|
| **browser** | `src/browser/` | 浏览器自动化（CDP）|
| **consensus** | `src/consensus/` | 多 Agent 共识机制 |

---

### ⚙️ Feature 标志

| Feature | 说明 |
|---------|------|
| `wasm-runtime` (默认) | 启用 WASM 技能运行时 |
| `a2a-server` | 启用 A2A HTTP 服务器（需 axum）|

**使用示例：**
```toml
[dependencies]
# 基础使用
beebotos-agents = { path = "crates/agents", default-features = false }

# 完整功能
beebotos-agents = { path = "crates/agents", features = ["a2a-server"] }
```

---

### 📁 项目结构

```
crates/agents/
├── Cargo.toml
└── src/
    ├── lib.rs              # 库入口
    ├── agent.rs            # Agent 核心 trait
    ├── error.rs            # 错误定义
    ├── types.rs            # 公共类型
    ├── config.rs           # 配置管理
    ├── a2a/                # A2A 协议
    │   ├── protocol.rs     # 协议定义
    │   ├── message.rs      # 消息格式
    │   ├── discovery.rs    # 服务发现
    │   ├── security.rs     # 安全加密
    │   └── transport.rs    # 传输层
    ├── mcp/                # MCP 客户端
    ├── spawning/           # 子 Agent 创建
    ├── session/            # 会话管理
    ├── queue/              # 任务队列
    ├── scheduling/         # 调度器
    ├── skills/             # 技能系统
    ├── runtime/            # 运行时
    ├── memory/             # 记忆系统
    ├── models/             # 模型路由
    ├── communication/      # 通信平台
    ├── browser/            # 浏览器控制
    ├── consensus/          # 共识算法
    └── ...
```

---

### 🛠 技术栈

| 组件 | 用途 |
|------|------|
| **tokio** | 异步运行时 |
| **async-trait** | 异步 trait |
| **wasmtime** | WASM 执行引擎 |
| **ethers** | 区块链交互 |
| **serde** | 序列化 |
| **reqwest** | HTTP 客户端 |
| **uuid** | 唯一标识 |
| **blake3** | 哈希计算 |
| **cron** | Cron 表达式解析 |

---

### ⚠️ 注意事项

1. **WASM 技能** - 需要启用 `wasm-runtime` feature 才能加载外部技能
2. **A2A 服务器** - 需要启用 `a2a-server` feature 并添加 axum 依赖
3. **资源限制** - 子 Agent 创建时会检查父 Agent 的资源配额
4. **会话隔离** - 使用 Linux namespace/cgroup 实现（需要特权）

需要我帮你实现具体的 Agent 功能或提供其他使用示例吗？



