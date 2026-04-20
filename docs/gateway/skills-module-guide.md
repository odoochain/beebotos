# BeeBotOS Skills 模块完整指南

> 本文档涵盖 Skills 模块的架构设计、编译流程、调试方法、测试策略、API 使用及典型应用场景。
>
> 适用版本: BeeBotOS v1.0.0+

---

## 目录

1. [模块概述](#1-模块概述)
2. [架构设计](#2-架构设计)
3. [核心组件](#3-核心组件)
4. [编译指南](#4-编译指南)
5. [调试指南](#5-调试指南)
6. [测试指南](#6-测试指南)
7. [HTTP REST API](#7-http-rest-api)
8. [gRPC API](#8-grpc-api)
9. [典型应用场景](#9-典型应用场景)
10. [配置说明](#10-配置说明)
11. [安全注意事项](#11-安全注意事项)
12. [故障排查](#12-故障排查)

---

## 1. 模块概述

Skills 模块是 BeeBotOS 的 **WASM 技能执行与编排平台**，提供从 Skill 安装、注册、实例化管理到安全执行的全生命周期支持。

### 1.1 演进路线

| 阶段 | 模型 | 特点 |
|------|------|------|
| v0.x | Singleton（单例） | `skill_id` 直接映射到 WASM 执行，无状态 |
| v1.0+ | **Instance-based（实例化）** | `instance_id` 作为运行时实体，支持多租户隔离、状态机、流式输出 |

### 1.2 关键能力

- **WASM 沙箱执行**: 基于 `wasmtime` 的安全运行时，带燃料计量和内存限制
- **多函数路由**: 单个 WASM 模块可导出多个函数，`function_name` 精确路由
- **实例状态机**: `Pending → Running → Paused → Stopped → Error`
- **流式执行**: `StreamFunction` 支持大输出分块推送
- **评分市场**: SQLite 持久化评分，支持 `rate_skill` / `get_skill_ratings`
- **Skill 持久化**: 重启后自动从 `data/skills` 目录恢复已安装技能

---

## 2. 架构设计

### 2.1 分层架构

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 4: API Gateway (HTTP + gRPC)                         │
│  - HTTP REST: /api/v1/skills/*, /api/v1/instances/*        │
│  - gRPC: SkillRegistry service                             │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: Skill Orchestration                               │
│  - InstanceManager: 实例生命周期 + 状态机 + usage 统计      │
│  - SkillRegistry:  Skill 元数据注册表                       │
│  - SkillsHub:        对 Registry 的薄封装                   │
│  - SkillRatingStore: SQLite 评分持久化                      │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Execution Engine                                  │
│  - SkillExecutor:    WASM 编译/实例化/调用/输出读取        │
│  - SkillSecurityValidator: wasmparser 静态安全分析         │
│  - SkillLoader:      从磁盘加载 manifest + WASM            │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: WASM Runtime (beebotos-kernel)                    │
│  - WasmEngine:       wasmtime 引擎 + 模块缓存              │
│  - WasmInstance:     内存读写 + 函数调用                   │
│  - HostFunctions:    beebotos::log/print/time/agent_id     │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 执行数据流

```
Client Request
     │
     ▼
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Resolve    │────▶│  Load Skill  │────▶│  Security   │
│  Instance   │     │  (YAML+WASM) │     │  Validate   │
└─────────────┘     └──────────────┘     └─────────────┘
                                                │
     ▼                                          ▼
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Record     │◀────│  Read Output │◀────│  Call WASM  │
│  Usage      │     │  [len][data] │     │  entry_point│
└─────────────┘     └──────────────┘     └─────────────┘
```

---

## 3. 核心组件

### 3.1 实例管理: `InstanceManager`

**源码**: `crates/agents/src/skills/instance_manager.rs`

```rust
pub struct InstanceManager {
    instances: RwLock<HashMap<String, SkillInstance>>,
    max_instances: usize,      // 默认 1000
    expiry_seconds: u64,       // 默认 3600
}
```

**状态转换规则**:

| From \ To | Pending | Running | Paused | Stopped | Error |
|-----------|---------|---------|--------|---------|-------|
| Pending   | ✓       | ✓       | ✗      | ✗       | ✓     |
| Running   | ✗       | ✓       | ✓      | ✓       | ✓     |
| Paused    | ✗       | ✓       | ✓      | ✓       | ✓     |
| Error     | ✗       | ✗       | ✗      | ✓       | ✓     |

**关键方法**:

```rust
pub async fn create(&self, skill_id, agent_id, config) -> Result<String, InstanceError>
pub async fn update_status(&self, instance_id, new_status) -> Result<(), InstanceError>
pub async fn cleanup_expired(&self) -> usize  // 返回清理数量
```

### 3.2 执行引擎: `SkillExecutor`

**源码**: `crates/agents/src/skills/executor.rs`

**WASM 调用约定**:

```
entry_point(input_ptr: i32, input_len: i32) -> output_ptr: i32

// output_ptr 指向的内存布局:
// [0..4]   output_len: i32 (小端序)
// [4..N]   output_data: [u8; output_len]
```

**参数序列化**:

`map<string, bytes>` → JSON（base64 编码值）→ UTF-8 字符串 → WASM 内存

```rust
// 输入示例
{"__input": "原始字符串", "image": "iVBORw0KGgo..."}

// 输出反序列化: 尝试解析为 map<string, bytes>，失败则保留原始字符串
```

**⚠️ Timeout 注意事项**:

`timeout_ms` 依赖 Tokio 的 async timeout。若 WASM guest 进入**无 yield 的无限循环**，timeout **可能无法及时触发**，因为 wasmtime 执行是同步阻塞的。长期方案需将执行移到 `spawn_blocking`。

### 3.3 安全验证: `SkillSecurityValidator`

**源码**: `crates/agents/src/skills/security.rs`

使用 `wasmparser` 静态分析 WASM 模块：

- 模块大小限制（默认 10MB）
- 版本必须为 1
- 禁止 imports: `env.__syscall`, `env.__wasi`, `wasi_snapshot_preview1`, ...
- 内存限制（默认 128MB）
- 数据段验证

### 3.4 评分存储: `SkillRatingStore`

**源码**: `crates/agents/src/skills/rating.rs`

SQLite 表结构:

```sql
CREATE TABLE skill_ratings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    skill_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK(rating >= 1 AND rating <= 5),
    review TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(skill_id, user_id)
);
```

---

## 4. 编译指南

### 4.1 环境要求

| 依赖 | 版本 | 说明 |
|------|------|------|
| Rust | >= 1.75.0 | 使用 `nightly` 工具链以获得最佳性能 |
| protoc | >= 3.21 | 用于 gRPC proto 代码生成 |
| SQLite | 3.x | 运行时依赖 |

### 4.2 编译命令

```bash
# 编译整个 Workspace（含 skills 模块）
cargo build --workspace

# 仅编译 beebotos-agents（skills 核心库）
cargo build -p beebotos-agents

# 仅编译 beebotos-gateway（HTTP + gRPC 服务入口）
cargo build -p beebotos-gateway

# Release 模式（推荐生产环境）
cargo build -p beebotos-gateway --release

# 运行单元测试
cargo test -p beebotos-agents --lib

# 运行集成测试
cargo test -p beebotos-agents --test skills_integration

# 运行 gRPC 服务测试
cargo test -p beebotos-gateway --bin beebotos-gateway -- grpc::skills::tests
```

### 4.3 Proto 代码生成

Gateway 的 `build.rs` 在编译时自动生成 protobuf Rust 代码:

```bash
# 手动触发（通常不需要）
cd apps/gateway && cargo build

# 生成的代码位于
target/debug/build/beebotos-gateway-*/out/beebotos.skills.registry.rs
target/debug/build/beebotos-gateway-*/out/beebotos.common.rs
```

---

## 5. 调试指南

### 5.1 日志级别

```bash
# 开启 skills 模块详细日志
RUST_LOG=beebotos_agents::skills=debug,beebotos_gateway=info cargo run -p beebotos-gateway

# 开启所有模块 trace 级日志（非常详细，仅调试使用）
RUST_LOG=trace cargo run -p beebotos-gateway
```

### 5.2 关键日志标记

| 日志内容 | 含义 |
|----------|------|
| `✅ SkillRegistry initialized` | Skill 注册表就绪 |
| `✅ SkillExecutor initialized` | WASM 执行引擎就绪 |
| `✅ SkillInstanceManager initialized` | 实例管理器就绪 |
| `✅ Restored N skills from disk` | 重启后从磁盘恢复了 N 个技能 |
| `🚀 Starting gRPC SkillRegistry server on 0.0.0.0:50051` | gRPC 服务已启动 |
| `[WASM] ...` | WASM guest 通过 `beebotos::log` 打印的日志 |

### 5.3 调试技巧

**查看 WASM 导出函数**:

```rust
// 在 executor.rs 中添加调试打印
let exports = instance.exports();
tracing::debug!("WASM exports: {:?}", exports);
```

**检查 WASM 内存布局**:

```rust
let pages = instance.memory_size();
tracing::debug!("WASM memory: {} pages ({} bytes)", pages, pages * 65536);
```

**验证 skill.yaml 格式**:

```bash
# 手动检查已安装 skill 的 manifest
cat data/skills/<skill_id>/skill.yaml
```

---

## 6. 测试指南

### 6.1 测试结构

```
crates/agents/tests/skills_integration.rs     # 集成测试（13 个用例）
apps/gateway/src/grpc/skills.rs               # gRPC 单元测试（2 个用例）
crates/agents/src/skills/
  ├── instance_manager.rs    # 模块内单元测试
  ├── executor.rs            # 模块内单元测试
  └── rating.rs              # 模块内单元测试
```

### 6.2 集成测试清单

```bash
# 运行所有 skills 集成测试
cargo test -p beebotos-agents --test skills_integration
```

| 测试名 | 验证内容 |
|--------|----------|
| `test_skill_loader_loads_manifest` | YAML manifest 解析 |
| `test_skill_registry_registration` | 注册 + 查询 |
| `test_skill_registry_enable_disable` | 启用/禁用状态切换 |
| `test_skill_security_validator_*` | 安全验证（合法/非法/超大） |
| `test_skills_hub_lifecycle` | Hub 封装层 CRUD |
| `test_skill_executor_creation` | Executor 初始化 |
| `test_instance_manager_lifecycle` | 创建/状态转换/删除 |
| `test_instance_manager_invalid_transition` | 非法状态转换被拒绝 |
| `test_instance_manager_usage_stats` | 使用统计计算 |
| `test_instance_manager_list_filtering` | 过滤查询 |
| `test_executor_stream_chunks` | 流式分块输出 |

### 6.3 gRPC 服务测试

```bash
# 直接测试 SkillsGrpcService（无需启动完整服务器）
cargo test -p beebotos-gateway --bin beebotos-gateway -- grpc::skills::tests
```

测试用例:
- `test_instance_lifecycle_grpc`: register → create → get → list → delete
- `test_delete_skill_not_found`: 错误处理验证

### 6.4 手动测试 WASM 执行

```bash
# 准备一个有效的 WASM skill
mkdir -p data/skills/hello

# 编写 skill.yaml
cat > data/skills/hello/skill.yaml << 'EOF'
id: hello
name: Hello Skill
version: 1.0.0
description: A test skill
author: test
license: MIT
capabilities: []
permissions: []
entry_point: handle
EOF

# 放置 WASM 文件（需符合调用约定）
cp your_skill.wasm data/skills/hello/skill.wasm

# 启动 Gateway
cargo run -p beebotos-gateway

# 测试 HTTP 执行
curl -X POST http://localhost:8080/api/v1/skills/hello/execute \
  -H "Content-Type: application/json" \
  -d '{"input": "world"}'
```

---

## 7. HTTP REST API

### 7.1 Skill 管理

#### 安装 Skill
```http
POST /api/v1/skills/install
Content-Type: application/json

{
  "source": "code-generator",
  "agent_id": "agent-001",
  "hub": "clawhub"
}
```

#### 列出 Skills
```http
GET /api/v1/skills?category=analytics&hub=local
```

#### 获取 Skill 详情
```http
GET /api/v1/skills/code-generator
```

#### 卸载 Skill
```http
DELETE /api/v1/skills/code-generator/uninstall
```

#### 执行 Skill（传统模式）
```http
POST /api/v1/skills/code-generator/execute
Content-Type: application/json

{
  "input": "写一个快速排序"
}
```

### 7.2 实例管理（Instance-based）

#### 创建实例
```http
POST /api/v1/instances
Content-Type: application/json

{
  "skill_id": "code-generator",
  "agent_id": "agent-001",
  "config": {
    "model": "gpt-4",
    "temperature": "0.2"
  }
}
```

**响应**:
```json
{
  "instance_id": "550e8400-e29b-41d4-a716-446655440000",
  "skill_id": "code-generator",
  "agent_id": "agent-001",
  "status": "running",
  "config": {"model": "gpt-4", "temperature": "0.2"},
  "started_at": 1713369600,
  "last_active": 1713369600,
  "usage": {
    "total_calls": 0,
    "successful_calls": 0,
    "failed_calls": 0,
    "avg_latency_ms": 0.0
  }
}
```

#### 列出实例
```http
GET /api/v1/instances?agent_id=agent-001&status=running
```

#### 获取实例
```http
GET /api/v1/instances/550e8400-e29b-41d4-a716-446655440000
```

#### 更新实例
```http
PUT /api/v1/instances/550e8400-e29b-41d4-a716-446655440000
Content-Type: application/json

{
  "config_updates": {"temperature": "0.8"},
  "status": "paused"
}
```

#### 删除实例
```http
DELETE /api/v1/instances/550e8400-e29b-41d4-a716-446655440000
```

#### 通过实例执行（推荐）
```http
POST /api/v1/instances/550e8400-e29b-41d4-a716-446655440000/execute
Content-Type: application/json

{
  "function_name": "generate",
  "parameters": {
    "prompt": "写一个快速排序"
  },
  "timeout_ms": 5000,
  "input": "rust"
}
```

---

## 8. gRPC API

### 8.1 服务定义

**Proto 文件**: `proto/skills/registry.proto`

**服务地址**: `localhost:50051`（默认，通过 `server.grpc_port` 配置）

**生成的 Client**: `beebotos::skills::registry::skill_registry_client::SkillRegistryClient`

### 8.2 方法速查表

| 方法 | 请求类型 | 响应类型 | 说明 |
|------|----------|----------|------|
| `RegisterSkill` | `RegisterSkillRequest` | `RegisterSkillResponse` | 注册 + 可选上传 ZIP |
| `GetSkill` | `GetSkillRequest` | `Skill` | 获取 Skill 元数据 |
| `ListSkills` | `ListSkillsRequest` | `ListSkillsResponse` | 列出 + 按类别过滤 |
| `SearchSkills` | `SearchSkillsRequest` | `ListSkillsResponse` | 关键词搜索 |
| `UpdateSkill` | `UpdateSkillRequest` | `Skill` | 更新元数据 |
| `DeleteSkill` | `DeleteSkillRequest` | `DeleteSkillResponse` | 删除 + 磁盘清理 |
| `CreateInstance` | `CreateInstanceRequest` | `SkillInstance` | 创建实例（自动 Running） |
| `GetInstance` | `GetInstanceRequest` | `SkillInstance` | 获取实例详情 |
| `UpdateInstance` | `UpdateInstanceRequest` | `SkillInstance` | 更新配置/状态 |
| `DeleteInstance` | `DeleteInstanceRequest` | `DeleteInstanceResponse` | 删除实例 |
| `ListInstances` | `ListInstancesRequest` | `ListInstancesResponse` | 按 agent/skill/status 过滤 |
| `ExecuteFunction` | `ExecuteFunctionRequest` | `ExecuteFunctionResponse` | 同步执行（带超时） |
| `StreamFunction` | `StreamFunctionRequest` | `stream FunctionOutput` | 流式执行 |
| `RateSkill` | `RateSkillRequest` | `RateSkillResponse` | 评分（upsert） |
| `GetSkillRatings` | `GetSkillRatingsRequest` | `GetSkillRatingsResponse` | 获取评分列表 |

### 8.3 客户端调用示例

```rust
use tonic::transport::Channel;
use beebotos::skills::registry::{
    skill_registry_client::SkillRegistryClient,
    CreateInstanceRequest, ExecuteFunctionRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://localhost:50051").connect().await?;
    let mut client = SkillRegistryClient::new(channel);

    // 创建实例
    let instance = client
        .create_instance(CreateInstanceRequest {
            skill_id: "code-generator".to_string(),
            agent_id: "agent-001".to_string(),
            config: [("model".to_string(), "gpt-4".to_string())].into(),
        })
        .await?
        .into_inner();

    println!("Instance created: {}", instance.instance_id);

    // 执行函数
    let result = client
        .execute_function(ExecuteFunctionRequest {
            instance_id: instance.instance_id,
            function_name: "generate".to_string(),
            parameters: [("prompt".to_string(), b"快速排序".to_vec())].into(),
            timeout_ms: 5000,
        })
        .await?
        .into_inner();

    println!("Output: {:?}", result.results);
    Ok(())
}
```

---

## 9. 典型应用场景

### 9.1 场景: 多租户 AI Agent 平台

100 个 Agent 共享同一个「代码生成 Skill」，每个 Agent 有不同配置。

```rust
// 平台为每个 Agent 创建独立实例
for agent in agents {
    let instance_id = instance_manager
        .create("code-generator", &agent.id, agent.config.clone())
        .await?;
    // 实例状态自动变为 Running
}

// Agent-A 执行（使用自己的 config）
executor.execute_function(
    &skill,
    Some("generate"),
    params,
    Some(5000),
    SkillContext { input: "".into(), parameters: agent_a_config },
).await?;
```

**收益**: 配置隔离、usage 独立统计、可独立暂停/恢复。

### 9.2 场景: DeFAI 交易策略生命周期

```protobuf
// 用户创建策略
CreateInstance {
  skill_id: "dex-trader"
  config: { "chain": "bsc", "max_slippage": "0.5" }
}

// 平台风控暂停
UpdateInstance { instance_id: "...", new_status: PAUSED }

// 用户恢复
UpdateInstance { instance_id: "...", new_status: RUNNING }

// 用户提取收益后关闭
DeleteInstance { instance_id: "..." }
```

### 9.3 场景: 流式日志分析

```protobuf
// StreamFunction
instance_id: "log-analyzer-1"
function_name: "analyze_stream"
parameters: { "log_source": "/var/log/app.log" }

// 服务端流式返回:
// data: "异常1: Connection timeout"
// data: "异常2: Memory spike"
// complete: true
```

### 9.4 场景: Skill 市场评分

```protobuf
// 用户安装后评分
RateSkill {
  skill_id: "data-visualizer"
  user_id: "user-123"
  rating: 5
  review: "图表非常专业！"
}

// 其他用户查看
GetSkillRatings {
  skill_id: "data-visualizer"
  page_size: 10
}
// 返回: average_rating=4.8, total_ratings=156
```

---

## 10. 配置说明

### 10.1 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `BEEBOTOS_SKILLS_DIR` | `data/skills` | Skill 安装目录 |
| `CLAWHUB_URL` | `https://hub.claw.dev/v1` | ClawHub 地址 |
| `CLAWHUB_API_KEY` | - | ClawHub API 密钥 |

### 10.2 TOML 配置

```toml
[server]
host = "0.0.0.0"
port = 8080
grpc_port = 50051          # gRPC SkillRegistry 服务端口
timeout_seconds = 30
max_body_size_mb = 10
```

### 10.3 InstanceManager 限制配置（代码级）

```rust
// 在 AppState::new() 中调整
let instance_manager = Arc::new(
    InstanceManager::with_limits(
        5000,    // max_instances: 最大实例数
        7200,    // expiry_seconds: 2小时无活动自动清理
    )
);
```

---

## 11. 安全注意事项

### 11.1 已修复的漏洞

| 漏洞 | 修复措施 | 位置 |
|------|----------|------|
| 目录遍历 | `validate_skill_id()` 拒绝 `..` / `/` / `\` | `grpc/skills.rs` |
| DoS（大包） | `MAX_PACKAGE_SIZE = 50MB` | `grpc/skills.rs` |
| 非法 WASM | `SkillSecurityValidator` 静态分析 | `security.rs` |

### 11.2 WASM 安全策略默认值

```rust
SkillSecurityPolicy {
    max_module_size: 10 * 1024 * 1024,  // 10MB
    max_memory_mb: 128,
    network_access: false,
    filesystem_access: false,
    env_access: false,
    allowed_imports: vec![
        "env.memory",
        "env.__stack_pointer",
        "env.__memory_base",
        "env.__table_base",
    ],
}
```

### 11.3 运行时安全

- **燃料计量**: `EngineConfig.max_fuel = 10_000_000`，防止无限循环耗尽 CPU
- **内存限制**: `EngineConfig.max_memory_size = 128MB`
- **WASM 沙箱**: 禁止直接访问文件系统、网络、环境变量
- **Host 函数白名单**: 仅暴露 `beebotos::log`, `beebotos::print`, `beebotos::time`, `beebotos::agent_id`

---

## 12. 故障排查

### 12.1 常见问题速查

| 问题 | 可能原因 | 解决方案 |
|------|----------|----------|
| `ExecutionFailed: Function not found` | WASM 未导出该函数名 | 检查 `skill.yaml` 的 `entry_point` 或 `functions` 定义 |
| `ExecutionFailed: Execution timed out` | WASM 执行超过 `timeout_ms` | 增加 timeout，或检查 WASM 是否有死循环 |
| `Security validation failed` | WASM 包含禁止的 imports | 使用 `wasm-objdump -x` 检查 imports |
| `Instance limit reached` | `max_instances` 已满 | 删除旧实例或调大 limit |
| `gRPC server not available` | 端口冲突或未启动 | 检查 `grpc_port` 配置和端口占用 |
| `Skill not found after restart` | 磁盘文件丢失 | 检查 `BEEBOTOS_SKILLS_DIR` 目录权限 |

### 12.2 调试 WASM 输出

```rust
// 在 executor.rs 的 run_wasm_function 中添加
let output_data = instance.read_memory(...)?;
tracing::debug!("WASM raw output ({} bytes): {:?}", output_data.len(), &output_data[..100.min(output_data.len())]);
```

### 12.3 检查 Instance 状态

```bash
# HTTP 方式
curl http://localhost:8080/api/v1/instances

# 查看特定实例
curl http://localhost:8080/api/v1/instances/<instance_id>
```

---

## 附录 A: 相关文件索引

| 文件 | 作用 |
|------|------|
| `crates/agents/src/skills/mod.rs` | 模块导出 |
| `crates/agents/src/skills/instance_manager.rs` | 实例管理 |
| `crates/agents/src/skills/executor.rs` | WASM 执行引擎 |
| `crates/agents/src/skills/loader.rs` | Skill 加载 + Manifest |
| `crates/agents/src/skills/registry.rs` | Skill 注册表 |
| `crates/agents/src/skills/security.rs` | WASM 安全验证 |
| `crates/agents/src/skills/rating.rs` | 评分存储 |
| `crates/agents/src/skills/hub.rs` | Hub 封装 |
| `apps/gateway/src/grpc/skills.rs` | gRPC 服务实现 |
| `apps/gateway/src/handlers/http/skills.rs` | HTTP REST 处理 |
| `apps/gateway/src/main.rs` | Gateway 启动 + Skill 恢复 |
| `proto/skills/registry.proto` | gRPC 协议定义 |
| `migrations_sqlite/007_add_skill_ratings.sql` | 评分表迁移 |

---

*文档版本: 2025-04-17*  
*维护者: BeeBotOS Team*

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

## 输出一份skills模块的编译、调试、测试和应用的完整说明文档，放在目录docs/gateway下面。

文档已写入 `docs/gateway/skills-module-guide.md`，共 **766 行**，涵盖以下 12 个章节：

| 章节 | 内容 |
|------|------|
| 1. 模块概述 | Singleton → Instance-based 演进、关键能力 |
| 2. 架构设计 | 4 层架构图 + 执行数据流 |
| 3. 核心组件 | InstanceManager、SkillExecutor、SecurityValidator、RatingStore 详解 |
| 4. 编译指南 | 环境要求、编译命令、proto 生成 |
| 5. 调试指南 | 日志级别、关键标记、调试技巧 |
| 6. 测试指南 | 测试结构、13 个集成测试、2 个 gRPC 测试、手动测试 WASM |
| 7. HTTP REST API | 11 个端点（Skill 管理 5 个 + Instance 管理 6 个），含请求/响应示例 |
| 8. gRPC API | 15 个 RPC 方法速查表 + Rust 客户端调用示例 |
| 9. 典型应用场景 | 多租户 AI Agent、DeFAI 策略、流式日志、Skill 市场评分 |
| 10. 配置说明 | 环境变量、TOML 配置、InstanceManager 限制参数 |
| 11. 安全注意事项 | 已修复漏洞、安全策略默认值、运行时安全 |
| 12. 故障排查 | 6 个常见问题速查 + WASM 调试技巧 |

附录包含完整的 **相关文件索引表**，方便快速定位源码。


%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
这次请求与之前失败的请求形成完美对比：**超时修改已生效**，Planning 流程完整跑通。以下是 planning 业务过程的详细分析：

---

## 一、Planning 完整业务流程（时间线）

```
15:39:00.471  [用户输入] "查看天气预报和发邮件的工作规划"
15:39:00.473  [SessionManager] 创建新会话 352603c3-...
15:39:00.653  [MessageProcessor] 记忆检索 → 5 条历史记忆，注入 834 chars 上下文
15:39:01.327  [Gateway] ⚡ 前台快速回复已发出（用户先收到轻量确认）
15:39:01.454  [后台] 🤖 Agent 开始处理任务
15:39:01.455  [Kernel] Agent 在沙箱中执行 task 0a7e37fa-...
15:39:01.695  [AgentImpl] 识别任务类型: llm_chat
15:39:01.696  [Planning] 🧠 触发条件满足（消息长度 45 ≥ 阈值）→ 启用 Planning
15:39:01.697  [Planning] 创建 Plan: b5ac2563-...（包含 1 个 step）
15:39:01.697  [Planning] 执行 Step 0: "查看天气预报和发邮件的工作规划"
15:39:01.698  [LLM Service] 发起 LLM 请求（经 Planning 包装后的复杂 prompt）
15:39:25.542  [Failover] ✅ Primary (Kimi) 成功返回
15:39:25.543  [LLM Service] 响应: 2499 chars, 耗时 23.8s, tokens 44输入/926输出
15:39:25.543  [AgentImpl] 任务成功，耗时 23.8s
15:39:25.543  [StateManager] Idle → Working → Idle
15:39:25.734  [后台回复] 完整工作规划方案（约 2499 字符的详细回复）
15:39:26.210  [Gateway] 回复推送到微信
```

---

## 二、Planning 触发机制分析

### 触发条件
```
"Complex LLM task detected (message length: 45), using planning"
```

从代码逻辑推断，Planning 触发采用**启发式规则**：
- **消息长度阈值**：当用户消息超过一定字符数（这里 45 字符）时判定为"复杂任务"
- **意图复杂度**："查看天气预报 **和** 发邮件 **的** 工作规划" 包含多个动词和名词，涉及多任务编排

### 为什么"江西省的面积多少"没有触发 Planning？
- 消息长度仅 9 个字符
- 单一意图（查询事实）
- 直接走 `llm_chat` 最短路径

---

## 三、Planning 执行过程详解

### Step 1: Plan 创建
```
Created plan b5ac2563-... with 1 steps for task 0a7e37fa-...
```

- Planning 引擎将用户意图拆解为**结构化执行计划**
- 本例中只生成了 **1 个 step**（说明意图相对集中，无需多步骤拆解）
- Plan ID: `b5ac2563-efe9-49f5-8888-1213804ad144`

### Step 2: Step 执行
```
Executing plan b5ac2563-... step 0: 查看天气预报和发邮件的工作规划
```

- AgentImpl 按顺序执行 Plan 中的每个 step
- 每个 step 本质上是**一次带上下文的 LLM 调用**
- 但与普通 `llm_chat` 的区别在于：
  - Prompt 经过 Planning 层的系统提示词增强
  - 要求 LLM 以"结构化规划/方案"格式输出
  - 可能注入了任务分解的元指令

### Step 3: LLM 推理与响应
```
📊 Processing LLM request: text='查看天气预报和发邮件的工作规划...', images=0
Request succeeded with provider primary
✅ Received LLM response: length=2499, latency=23845ms, tokens=44/926
```

| 指标 | 数值 | 说明 |
|------|------|------|
| **输入 tokens** | 44 | 用户问题 + 系统提示 + 记忆上下文 |
| **输出 tokens** | 926 | 详细的工作规划方案（约 2500 字符） |
| **推理耗时** | 23.8s | Kimi 生成近 1000 tokens 的复杂内容 |
| **输出速度** | ~39 tokens/s | 符合 kimi-k2.5 长文本生成速度 |

---

## 四、关键对比：修改超时前后的差异

| 维度 | 修改前（30s 超时） | 修改后（60s 超时） |
|------|-------------------|-------------------|
| **Planning 触发** | ✅ 正常触发 | ✅ 正常触发 |
| **Plan 创建** | ✅ 成功创建 1 step | ✅ 成功创建 1 step |
| **LLM 调用** | ❌ 30s 超时中断 | ✅ 23.8s 成功返回 |
| **Failover** | ❌ 备用也失败 | — 无需降级 |
| **任务结果** | ❌ 返回错误给用户 | ✅ 返回完整方案 |
| **用户体验** | 30s 后收到错误 | 24s 后收到详细规划 |

**核心结论**：不是 Planning 逻辑有 bug，也不是 Kimi 服务不可用，而是 **Planning 任务对 LLM 的输出要求更高（926 tokens 长文本生成），导致推理时间超过 30s 阈值**。将 failover 超时从 30s 提升到 60s 后，问题彻底解决。

---

## 五、Planning 生成内容特点（从日志推断）

从后台回复的片段可以看出：

```
"作为您的 BeeBotOS 个人 AI 助手，我为您设计了一套高效的
「天气查看 + 邮件处理」工作规划方案..."

## 🌅 **晨间快速启动模式（5-10分钟）**
```

Planning 层的输出特征：
1. **结构化**：使用 Markdown 标题、列表、时间块
2. **场景化**：按"晨间/日间/晚间"等时间维度组织
3. **可执行性**：每个步骤包含具体时间估算（如 5-10 分钟）
4. **角色一致性**：始终以 "BeeBotOS 个人 AI 助手" 身份回答（来自 `system_prompt` 配置）

---

## 六、Planning 在系统架构中的位置

```
用户消息
    ↓
[Gateway] MessageProcessor
    ├─ 记忆检索（834 chars 上下文）
    ├─ ⚡ 快速回复（轻量确认）
    └─ 派发后台任务
         ↓
[Agent Layer] AgentImpl
    ├─ 判断任务复杂度 → 触发 Planning（本例）
    │      └─ Planning Engine
    │             ├─ 意图分析 → 生成 Plan（1 step）
    │             ├─ 逐 step 执行
    │             │      └─ 调用 LLM Service（23.8s）
    │             └─ 结果聚合
    └─ 普通 llm_chat（简单问题直接调用）
         ↓
[LLM Layer] FailoverProvider（attempt_timeout=60s）
    ├─ Primary: Kimi-k2.5（成功）
    └─ Fallback: （未触发）
```

---

## 七、业务价值总结

Planning 机制在这次请求中体现的价值：

| 能力 | 体现 |
|------|------|
| **任务分解** | 将模糊需求"查看天气预报和发邮件的工作规划"转化为结构化方案 |
| **上下文感知** | 结合 834 chars 历史记忆，保持对话连续性 |
| **长文本生成** | 生成 926 tokens（~2500 字符）的详细规划 |
| **容错恢复** | 超时配置从 30s → 60s 后，复杂任务不再失败 |
| **异步体验** | 前台 0.8s 快速响应 + 后台 24s 深度生成，用户不等待 |

%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

已创建 Skills 模块测试案例文档，保存在：

📄 **`docs/gateway/Skill_Test_Examples.md`**（共 862 行）

文档包含 **6 个完整测试案例样板**，全部基于项目实际 API 编写：

| 案例 | 测试目标 | 核心验证点 |
|------|---------|-----------|
| **案例1** | 本地 Skill 完整生命周期 | `SkillLoader` 加载 → `SkillRegistry` 注册 → `InstanceManager` 实例化 → `SkillExecutor` WASM 执行 |
| **案例2** | InstanceManager 状态机 | Pending→Running→Paused→Stopped 合法流转、非法状态转换拦截、执行统计与过滤查询 |
| **案例3** | ClawHub 下载与安装 | `wiremock` 模拟 Hub Server，`ClawHubClient` 元数据获取 → ZIP 下载 → 内容解压验证 |
| **案例4** | Gateway REST API 集成 | `/api/v1/skills/install`、`/:id/execute`、实例 CRUD 的端到端 HTTP 测试 |
| **案例5** | Skill 安全验证 | `SkillSecurityValidator` 拦截超大模块、非法 WASM 结构、自定义策略生效 |
| **案例6** | Skill 评星与统计 | `SkillRatingStore` 评分提交、汇总计算、分布统计、用户评分更新 |

每个案例均包含：
- **文件放置建议路径**
- **可直接复制使用的完整 Rust 代码**
- **基于真实 API 的断言逻辑**
- **临时环境搭建与清理方案**

附录还提供了**最小 WASM Skill 编译指南**，方便生成真实可执行的测试固件。


