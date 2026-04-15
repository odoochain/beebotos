
## beebotos-sdk 编译和使用指南

**beebotos-sdk** 是 BeeBotOS 的 **软件开发工具包**，用于构建 Agent 和应用程序。支持 Rust 和 WASM，提供类型定义、客户端 API、执行上下文管理等功能。

---

### 📦 编译命令

#### 1. 编译整个项目（包含 SDK）
```bash
# 项目根目录
cargo build --release

# 编译后的库
# target/release/libbeebotos_sdk.rlib
```

#### 2. 只编译 SDK crate
```bash
# 编译 beebotos-sdk
cargo build --release -p beebotos-sdk

# 调试模式
cargo build -p beebotos-sdk
```

#### 3. 不同 Feature 编译
```bash
# 默认特性（std + HTTP 客户端）
cargo build -p beebotos-sdk

# WASM 目标（无 std）
cargo build -p beebotos-sdk --target wasm32-unknown-unknown --no-default-features --features wasm

# 纯 std（无 HTTP）
cargo build -p beebotos-sdk --no-default-features --features std
```

#### 4. 运行测试
```bash
# 运行单元测试
cargo test -p beebotos-sdk

# 带日志输出
RUST_LOG=debug cargo test -p beebotos-sdk -- --nocapture
```

---

### 🚀 使用方法

#### 作为库依赖

在 `Cargo.toml` 中添加：
```toml
[dependencies]
beebotos-sdk = { path = "crates/sdk" }

# WASM 项目使用
beebotos-sdk = { path = "crates/sdk", default-features = false, features = ["wasm"] }
```

---

### 💻 编程示例

#### 1. 初始化 SDK 客户端

```rust
use beebotos_sdk::{init, SdkConfig, BeeBotOSClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 方法1: 使用默认配置
    let config = SdkConfig::default();
    
    // 方法2: 自定义配置
    let config = SdkConfig::new("https://api.beebotos.dev")
        .with_api_key("your-api-key")
        .with_timeout(60); // 60秒超时
    
    // 初始化客户端
    let client = init(config).await?;
    
    // 检查服务健康
    let healthy = client.health().await?;
    println!("Service healthy: {}", healthy);
    
    Ok(())
}
```

---

#### 2. Agent 执行上下文

```rust
use beebotos_sdk::{AgentContext, AgentId, types::{SessionId, TaskId}};

fn main() {
    // 创建根 Agent 上下文
    let agent_id = AgentId::new();
    let ctx = AgentContext::new(agent_id.clone());
    
    println!("Agent ID: {}", ctx.agent_id);
    println!("Session ID: {}", ctx.session_id);
    println!("Session Key: {}", ctx.session_key());
    println!("Workspace: {}", ctx.workspace_path());
    
    // 检查是否为根 Agent
    assert!(ctx.is_root());
    assert!(!ctx.is_subagent());
    
    // 添加能力
    let ctx = ctx
        .with_capability("web_search")
        .with_capability("code_execution")
        .with_metadata("project", "my_app")
        .with_metadata("version", "1.0.0");
    
    // 检查能力
    if ctx.has_capability("web_search") {
        println!("Agent can perform web search");
    }
    
    // 获取资源配额
    let quota = ctx.effective_quota();
    println!("Max tokens: {}", quota.max_tokens);
    println!("Max execution time: {}ms", quota.max_execution_time_ms);
}
```

---

#### 3. 创建子 Agent（嵌套上下文）

```rust
use beebotos_sdk::{AgentContext, AgentId, ContextBuilder, types::TaskId};

fn main() {
    // 父 Agent
    let parent_id = AgentId::new();
    let parent_ctx = AgentContext::new(parent_id)
        .with_capability("spawn")
        .with_task(TaskId::new());
    
    // 创建子 Agent 上下文
    let child_ctx = AgentContext::new(AgentId::new())
        .with_parent(&parent_ctx);
    
    println!("Parent: {:?}", parent_ctx.agent_id);
    println!("Child: {:?}", child_ctx.agent_id);
    println!("Depth: {}", child_ctx.depth); // 1
    
    // 子 Agent 继承父 Agent 的能力
    assert!(child_ctx.has_capability("spawn"));
    
    // 子 Agent 资源配额减半
    assert_eq!(child_ctx.quota.max_tokens, parent_ctx.quota.max_tokens / 2);
    
    // 使用 Builder 模式
    let child_ctx2 = ContextBuilder::new(AgentId::new())
        .with_parent(parent_ctx.clone())
        .with_capability("data_analysis")
        .with_scope(beebotos_sdk::context::ExecutionScope::Sandbox)
        .build();
}
```

---

#### 4. 核心类型使用

```rust
use beebotos_sdk::types::{
    AgentId, SessionId, TaskId, RunId,
    AgentConfig, ModelConfig, TaskConfig,
    Priority, TaskStatus, Capabilities,
    TokenUsage, CostInfo,
};

fn main() {
    // ID 类型
    let agent_id = AgentId::new();
    let session_id = SessionId::new();
    let task_id = TaskId::new();
    let run_id = RunId::new();
    
    println!("Agent: {}", agent_id);
    
    // 从字符串创建
    let agent_id = AgentId::from("agent-123");
    
    // Agent 配置
    let config = AgentConfig {
        name: "DataAnalyzer".to_string(),
        description: "Analyzes data and generates reports".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec!["read_csv".to_string(), "chart".to_string()],
        model: ModelConfig {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
        },
    };
    
    // 任务配置
    let task_config = TaskConfig {
        task_type: "data_analysis".to_string(),
        priority: Priority::High,
        timeout_ms: 600_000, // 10分钟
        max_retries: 3,
    };
    
    // 能力配置
    let caps = Capabilities {
        can_spawn: true,
        can_pay: false,
        can_access_internet: true,
        can_use_filesystem: true,
        custom: vec!["gpu_compute".to_string()],
    };
    
    // Token 使用统计
    let usage = TokenUsage::new(1000, 500);
    println!("Total tokens: {}", usage.total_tokens);
    
    // 成本信息
    let cost = CostInfo {
        token_cost: 0.03,
        compute_cost: 0.01,
        storage_cost: 0.001,
        total: 0.041,
        currency: "USD".to_string(),
    };
}
```

---

#### 5. WASM 技能开发

```rust
// 在 WASM 技能中使用 SDK（无 std）
#![no_std]

use beebotos_sdk::types::{AgentId, TaskId};

// WASM 入口函数
#[no_mangle]
pub extern "C" fn process_task(input_ptr: *const u8, input_len: usize) -> i32 {
    // 反序列化输入
    let input = unsafe {
        core::slice::from_raw_parts(input_ptr, input_len)
    };
    
    // 处理任务...
    
    0 // 成功
}

// 使用 SDK 类型进行序列化
#[derive(serde::Serialize)]
struct TaskOutput {
    task_id: TaskId,
    success: bool,
    output: String,
}
```

---

### 📋 核心功能模块

| 模块 | 文件 | 功能 |
|------|------|------|
| **client** | `client.rs` | BeeBotOS HTTP 客户端 |
| **context** | `context.rs` | Agent 执行上下文管理 |
| **types** | `types.rs` | 核心类型定义 |
| **error** | `error.rs` | 错误处理 |
| **message** | `message.rs` | 消息类型 |
| **storage** | `storage.rs` | 存储接口 |
| **http** | `http.rs` | HTTP 工具 |

---

### ⚙️ Feature 标志

| Feature | 说明 |
|---------|------|
| `std` (默认) | 标准库支持 + HTTP 客户端 |
| `wasm` | WASM 目标（无 std）|
| `tracing` | 日志追踪 |

---

### 📁 项目结构

```
crates/sdk/
├── Cargo.toml
└── src/
    ├── lib.rs          # 库入口
    ├── client.rs       # SDK 客户端
    ├── context.rs      # Agent 上下文
    ├── types.rs        # 核心类型
    ├── error.rs        # 错误定义
    ├── message.rs      # 消息类型
    ├── storage.rs      # 存储接口
    └── http.rs         # HTTP 工具
```

---

### 🛠 技术栈

| 组件 | 用途 |
|------|------|
| **serde** | 序列化 |
| **uuid** | 唯一标识 |
| **reqwest** | HTTP 客户端 |
| **tracing** | 日志追踪 |
| **thiserror** | 错误处理 |

---

### ⚠️ 注意事项

1. **WASM 目标** - 使用 `wasm` feature 时禁用 std
2. **API Key** - 生产环境需要配置 API 密钥
3. **超时设置** - 根据任务复杂度调整超时时间
4. **资源配额** - 子 Agent 自动继承减半的配额

需要我帮你实现具体的 SDK 功能或提供其他使用示例吗？


