# BeeBotOS Kernel 模块详细分析文档

## 目录

1. [概述](#概述)
2. [架构设计](#架构设计)
3. [核心模块详解](#核心模块详解)
4. [系统调用接口](#系统调用接口)
5. [与其他模块的关系](#与其他模块的关系)
6. [函数接口定义](#函数接口定义)
7. [数据流分析](#数据流分析)
8. [安全模型](#安全模型)

---

## 概述

BeeBotOS Kernel 是操作系统的核心层（Layer 1），提供底层资源管理、任务调度、安全隔离和系统调用接口。Kernel 作为整个系统的基础，为上层 Agent Runtime、Social Brain 和 Gateway 提供稳定的运行环境。

### 核心特性

| 特性 | 描述 |
|------|------|
| **抢占式调度** | 基于 CFS（Completely Fair Scheduler）的工作窃取线程池 |
| **Capability 安全** | 11 级能力层级模型（L0-L10）|
| **系统调用** | 29 个系统调用用于 Agent 管理 |
| **WASM 运行时** | 基于 wasmtime 18.0 的 WebAssembly 执行环境 |
| **持久化存储** | 多后端存储支持（RocksDB、内存存储）|
| **IPC 机制** | 基于消息传递的进程间通信 |

---

## 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Gateway    │  │    Agents    │  │  Social Brain    │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
└─────────┼─────────────────┼───────────────────┼────────────┘
          │                 │                   │
          ▼                 ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                      Kernel Layer (Layer 1)                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Scheduler  │  │  Syscalls    │  │   WASM Engine    │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │  Security    │  │  Storage     │  │    IPC/Router    │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Memory     │  │  Resource    │  │    Network       │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Hardware/OS Abstraction                  │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 模块结构

```
crates/kernel/src/
├── lib.rs                 # 核心 Kernel 结构和 Builder 模式
├── boot.rs               # 启动流程
├── error.rs              # 错误类型定义
├── scheduler/            # 任务调度模块
│   ├── mod.rs           # 调度器核心实现
│   ├── task.rs          # 任务定义和状态管理
│   ├── queue.rs         # 任务队列
│   ├── executor.rs      # 线程池执行器
│   ├── fair.rs          # CFS 公平调度
│   └── priority.rs      # 优先级管理
├── syscalls/            # 系统调用模块
│   ├── mod.rs           # 系统调用号和分发器
│   └── handlers.rs      # 系统调用处理函数
├── capabilities/        # 能力系统
│   ├── mod.rs           # CapabilitySet 管理
│   ├── levels.rs        # 11 级能力层级
│   ├── tokens.rs        # 能力令牌
│   └── registry.rs      # 能力注册表
├── security/            # 安全模块
│   ├── mod.rs           # 安全管理器
│   ├── acl.rs           # 访问控制列表
│   ├── audit.rs         # 审计日志
│   └── path.rs          # 路径验证
├── wasm/                # WASM 运行时
│   ├── mod.rs           # WASM 模块入口
│   ├── engine.rs        # wasmtime 引擎封装
│   ├── instance.rs      # 实例管理
│   ├── host_funcs.rs    # 宿主函数
│   └── wasi_ctx.rs      # WASI 上下文
├── ipc/                 # 进程间通信
│   ├── mod.rs           # IPC 模块入口
│   ├── router.rs        # 消息路由器
│   ├── channel.rs       # 通道实现
│   └── shared_memory.rs # 共享内存
├── storage/             # 存储模块
│   ├── mod.rs           # 存储管理器
│   ├── kv_store.rs      # KV 存储
│   ├── blob_store.rs    # 大对象存储
│   └── global.rs        # 全局存储实例
├── resource/            # 资源管理
│   ├── mod.rs           # 资源管理器
│   ├── limit.rs         # 资源限制
│   └── circuit_breaker.rs # 熔断器
├── memory/              # 内存管理
│   ├── mod.rs           # 内存子系统
│   ├── allocator.rs     # 分配器
│   └── paging.rs        # 分页管理
├── network/             # 网络模块
│   └── mod.rs           # P2P 网络栈
└── task/                # 任务管理
    ├── mod.rs           # 任务定义
    ├── process.rs       # 进程管理
    └── thread.rs        # 线程管理
```

---

## 核心模块详解

### 3.1 Kernel 核心结构

```rust
/// BeeBotOS Kernel 主结构
pub struct Kernel {
    scheduler: scheduler::Scheduler,           // 任务调度器
    security: security::SecurityManager,       // 安全管理器
    syscall_dispatcher: syscalls::SyscallDispatcher, // 系统调用分发器
    wasm_engine: Option<wasm::WasmEngine>,     // WASM 引擎
    config: KernelConfig,                      // 内核配置
    running: std::sync::atomic::AtomicBool,    // 运行状态
}

/// 内核配置
pub struct KernelConfig {
    pub scheduler: scheduler::SchedulerConfig,
    pub security_policy: Box<dyn security::SecurityPolicy>,
    pub memory_config: memory::MemoryConfig,
    pub tee_provider: Option<()>,
    pub max_agents: usize,
    pub audit_enabled: bool,
    pub wasm_enabled: bool,
}
```

#### Builder 模式

```rust
let kernel = KernelBuilder::new()
    .with_max_agents(1000)
    .with_scheduler(SchedulerConfig::production())
    .with_security_policy(MyPolicy::new())
    .with_wasm(true)
    .build()?;
```

### 3.2 调度器 (Scheduler)

调度器采用 **CFS（Completely Fair Scheduler）** 算法，结合工作窃取线程池实现高效的任务调度。

#### 核心结构

```rust
pub struct Scheduler {
    config: SchedulerConfig,
    executor: Option<Arc<ThreadPoolExecutor>>,    // 线程池执行器
    tasks: Arc<RwLock<HashMap<TaskId, TaskInfo>>>, // 任务信息表
    handles: Arc<RwLock<HashMap<TaskId, TaskHandle>>>, // 任务句柄
    stats: Arc<RwLock<SchedulerStats>>,            // 统计信息
    shutdown: tokio::sync::watch::Sender<bool>,    // 关闭信号
    next_task_id: AtomicU64,                       // 任务 ID 生成器
}

pub struct TaskInfo {
    pub id: TaskId,
    pub name: String,
    pub priority: Priority,           // RealTime, High, Normal, Low, Background
    pub state: TaskState,             // Ready, Running, Blocked, Zombie
    pub created_at: Instant,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub capabilities: CapabilitySet,  // 任务能力集
    pub resource_limits: ResourceLimits,
}
```

#### 调度配置

```rust
pub struct SchedulerConfig {
    pub max_concurrent: usize,        // 最大并发任务数
    pub time_slice_ms: u64,           // 时间片（抢占）
    pub enable_preemption: bool,      // 启用抢占
    pub default_priority: Priority,   // 默认优先级
    pub num_workers: usize,           // 工作线程数（0=自动）
    pub enable_work_stealing: bool,   // 启用工作窃取
    pub enable_cpu_affinity: bool,    // CPU 亲和性
}
```

#### 优先级枚举

```rust
pub enum Priority {
    RealTime = 0,     // 实时任务，最高优先级
    High = 1,         // 高优先级
    Normal = 2,       // 普通优先级
    Low = 3,          // 低优先级
    Background = 4,   // 后台任务，最低优先级
}
```

### 3.3 能力系统 (Capabilities)

能力系统实现了 **11 级分层安全模型**，从 L0（本地计算）到 L10（系统管理员）。

#### 能力层级定义

```rust
#[repr(u8)]
pub enum CapabilityLevel {
    L0LocalCompute = 0,    // 本地计算（沙箱）
    L1FileRead = 1,        // 文件读取
    L2FileWrite = 2,       // 文件写入
    L3NetworkOut = 3,      // 出站网络
    L4NetworkIn = 4,       // 入站网络（服务器）
    L5SpawnLimited = 5,    // 有限子 Agent 创建
    L6SpawnUnlimited = 6,  // 无限子 Agent 创建
    L7ChainRead = 7,       // 区块链读取
    L8ChainWriteLow = 8,   // 低价值链上写入
    L9ChainWriteHigh = 9,  // 高价值链上写入（需多签）
    L10SystemAdmin = 10,   // 系统管理员
}
```

#### 能力集

```rust
pub struct CapabilitySet {
    pub max_level: CapabilityLevel,
    pub permissions: HashSet<String>,  // 具体权限列表
    pub expires_at: Option<u64>,       // 过期时间
    pub delegable: bool,               // 是否可委托
}

impl CapabilitySet {
    pub fn empty() -> Self;      // 空能力集
    pub fn standard() -> Self;   // 标准 Agent 能力
    pub fn full() -> Self;       // 完整能力（系统 Agent）
    
    pub fn has(&self, level: CapabilityLevel) -> bool;
    pub fn has_permission(&self, perm: &str) -> bool;
    pub fn verify(&self, required: CapabilityLevel) -> Result<()>;
}
```

#### 能力衰减机制

```rust
pub struct DecayingCapability {
    pub level: CapabilityLevel,
    pub granted_at: u64,
    pub decay_rate: DecayRate,   // Slow(1天/级), Normal(1小时/级), Fast(10分钟/级)
}

impl DecayingCapability {
    pub fn current_level(&self) -> CapabilityLevel;
    pub fn is_expired(&self) -> bool;
    pub fn refresh(&mut self);
}
```

### 3.4 系统调用 (Syscalls)

Kernel 提供 **29 个系统调用** 供上层使用。

#### 系统调用号

```rust
#[repr(u64)]
pub enum SyscallNumber {
    SpawnAgent = 0,           // 创建 Agent
    TerminateAgent = 1,       // 终止 Agent
    SendMessage = 2,          // 发送消息
    AccessResource = 3,       // 访问资源
    ExecutePayment = 4,       // 执行支付
    QueryMemory = 5,          // 查询内存
    UpdateCapability = 6,     // 更新能力
    EnterSandbox = 7,         // 进入沙箱
    ExitSandbox = 8,          // 退出沙箱
    ReadFile = 9,             // 读取文件
    WriteFile = 10,           // 写入文件
    ListFiles = 11,           // 列出文件
    CreateWorkspace = 12,     // 创建工作空间
    DeleteWorkspace = 13,     // 删除工作空间
    QueryState = 14,          // 查询状态
    UpdateState = 15,         // 更新状态
    ScheduleTask = 16,        // 调度任务
    CancelTask = 17,          // 取消任务
    QuerySchedule = 18,       // 查询调度
    BridgeToken = 19,         // 跨链桥接
    SwapToken = 20,           // 代币交换
    StakeToken = 21,          // 质押代币
    UnstakeToken = 22,        // 解除质押
    QueryBalance = 23,        // 查询余额
    RequestAttestation = 24,  // 请求证明
    VerifyAttestation = 25,   // 验证证明
    LogEvent = 26,            // 记录事件
    EmitMetric = 27,          // 发送指标
    QueryMetrics = 28,        // 查询指标
}
```

#### 系统调用所需能力

| 系统调用 | 所需能力 | 说明 |
|---------|---------|------|
| `SpawnAgent` | L5 | 创建子 Agent |
| `TerminateAgent` | L5 | 终止 Agent |
| `SendMessage` | L3 | Agent 间通信 |
| `ReadFile` | L1 | 文件读取 |
| `WriteFile` | L2 | 文件写入 |
| `ExecutePayment` | L8 | 区块链支付 |
| `UpdateCapability` | L7 | 能力升级 |

#### 系统调用分发器

```rust
pub struct SyscallDispatcher {
    handlers: HashMap<SyscallNumber, Box<dyn SyscallHandler>>,
}

#[async_trait]
pub trait SyscallHandler: Send + Sync {
    async fn handle(&self, args: SyscallArgs, ctx: &SyscallContext) -> SyscallResult;
}

pub struct SyscallContext {
    pub caller_id: String,
    pub process_id: u64,
    pub capability_level: u8,
    pub workspace_id: String,
    pub session_id: String,
    pub memory_space: Option<Arc<ProcessMemorySpace>>,
}
```

### 3.5 WASM 运行时

基于 **wasmtime 18.0** 的 WebAssembly 执行环境。

#### 引擎配置

```rust
pub struct EngineConfig {
    pub max_memory_size: usize,       // 最大内存（默认 128MB）
    pub max_fuel: u64,                // 最大燃料单位
    pub fuel_metering: bool,          // 燃料计量
    pub memory_limits: bool,          // 内存限制
    pub wasi_enabled: bool,           // WASI 支持
    pub debug_info: bool,             // 调试信息
    pub parallel_compilation: bool,   // 并行编译
    pub optimize: bool,               // 优化级别
}
```

#### WASM 引擎

```rust
pub struct WasmEngine {
    config: EngineConfig,
    engine: Engine,
    module_cache: Arc<RwLock<HashMap<String, CachedModule>>>, // 模块缓存
}

impl WasmEngine {
    pub fn compile(&self, wasm_bytes: &[u8]) -> KernelResult<Module>;
    pub fn compile_cached(&self, name: &str, wasm_bytes: &[u8]) -> KernelResult<Module>;
    pub fn instantiate(&self, module: &Module) -> KernelResult<WasmInstance>;
    pub fn instantiate_wasi(&self, module: &Module, agent_id: &str, caps: Option<&WasiCapabilities>) -> KernelResult<WasmInstance>;
    pub fn precompile(&self, wasm_bytes: &[u8]) -> KernelResult<Vec<u8>>;
}
```

### 3.6 进程间通信 (IPC)

基于消息传递的 IPC 机制，支持路由和速率限制。

#### 消息信封

```rust
pub struct MessageEnvelope {
    pub source: String,         // 源 Agent ID
    pub destination: String,    // 目标 Agent ID
    pub payload: Vec<u8>,       // 消息负载（JSON 格式）
    pub timestamp: u64,         // 时间戳
    pub priority: u8,           // 优先级（0-255）
    pub timeout_ms: u64,        // 超时时间
}
```

#### 消息路由器

```rust
pub struct MessageRouter {
    mailboxes: Mutex<HashMap<String, AgentMailbox>>,
    global_stats: Mutex<RouterStats>,
    default_rate_limit: (u32, u64),  // (最大消息数, 窗口毫秒)
}

impl MessageRouter {
    pub fn register_agent(&self, agent_id: String) -> mpsc::UnboundedReceiver<MessageEnvelope>;
    pub fn unregister_agent(&self, agent_id: &str);
    pub fn route(&self, message: MessageEnvelope) -> KernelResult<()>;
    pub fn stats(&self) -> RouterStats;
}

/// 全局路由器实例
pub fn global_router() -> Arc<MessageRouter>;
```

#### Agent 邮箱

```rust
pub struct AgentMailbox {
    agent_id: String,
    sender: mpsc::UnboundedSender<MessageEnvelope>,
    rate_limiter: Mutex<RateLimiter>,  // 速率限制器
    stats: Mutex<MailboxStats>,
}
```

### 3.7 存储模块

支持多后端持久化存储，默认提供内存存储，可选 RocksDB。

#### 存储管理器

```rust
pub struct StorageManager {
    config: StorageConfig,
    backends: HashMap<String, Box<dyn StorageBackend>>,
    default_backend: String,
    stats: StorageStats,
}

pub trait StorageBackend: Send + Sync {
    fn put(&self, key: &str, data: &[u8], metadata: EntryMetadata) -> Result<(), StorageError>;
    fn get(&self, key: &str) -> Result<Option<StorageEntry>, StorageError>;
    fn delete(&self, key: &str) -> Result<(), StorageError>;
    fn list(&self, prefix: &str) -> Result<Vec<String>, StorageError>;
    fn exists(&self, key: &str) -> Result<bool, StorageError>;
}
```

#### 工作空间隔离

```rust
// 为每个 Agent 创建独立的工作空间
pub fn workspace_key(agent_id: &str, path: &str) -> String {
    format!("workspace/{}/{}", agent_id, path)
}
```

### 3.8 资源管理

跟踪和管理 Agent 的资源使用情况。

#### 资源使用统计

```rust
pub struct ResourceUsage {
    pub cpu_time: Duration,
    pub memory_bytes: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}
```

#### 资源限制

```rust
pub struct ResourceLimits {
    pub max_cpu_time: Option<Duration>,
    pub max_memory_bytes: Option<u64>,
    pub max_io_bytes: Option<u64>,
    pub max_network_bytes: Option<u64>,
    pub max_file_descriptors: Option<u32>,
    pub max_processes: Option<u32>,
}

impl ResourceLimits {
    pub fn check_usage(&self, usage: &ResourceUsage) -> ResourceStatus;
}

pub enum ResourceStatus {
    WithinLimits,
    Exceeded(ResourceType),
    Warning(ResourceType, f32),  // 接近限制（百分比）
}
```

---

## 与其他模块的关系

### 5.1 与 beebotos-agents 模块的关系

```
┌─────────────────────────────────────────────────────────────┐
│                    beebotos-agents                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ AgentRuntime│  │ QueueManager│  │  Spawning Engine    │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
└─────────┼────────────────┼────────────────────┼────────────┘
          │                │                    │
          │ spawn_task()   │ submit()           │ spawn_agent()
          │                │                    │
          ▼                ▼                    ▼
┌─────────────────────────────────────────────────────────────┐
│                    beebotos-kernel                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  Scheduler  │  │ TaskQueue   │  │  AgentRegistry      │ │
│  │  spawn()    │  │             │  │  register()         │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

#### 业务逻辑关系

| Agents 功能 | Kernel 支持 | 接口 |
|------------|------------|------|
| Agent 生命周期管理 | Task 管理 | `spawn_task()`, `cancel()` |
| 子 Agent 创建 | Agent 注册表 | `SyscallNumber::SpawnAgent` |
| 消息通信 | IPC Router | `global_router().route()` |
| 状态持久化 | Storage | `global_storage().put/get` |
| 资源限制 | ResourceManager | `set_process_limits()` |

#### 关键函数接口

```rust
// Agents -> Kernel
impl Kernel {
    /// 创建任务（Agent 使用）
    pub async fn spawn_task<F>(
        &self,
        name: impl Into<String>,
        priority: scheduler::Priority,
        capabilities: capabilities::CapabilitySet,
        f: F,
    ) -> Result<scheduler::TaskId>
    where F: Future<Output = Result<()>> + Send + 'static;
    
    /// 执行系统调用
    pub async fn syscall(
        &self,
        number: u64,
        args: syscalls::SyscallArgs,
        caller: AgentId,
    ) -> syscalls::SyscallResult;
}
```

### 5.2 与 beebotos-social-brain 模块的关系

```
┌─────────────────────────────────────────────────────────────┐
│                 beebotos-social-brain                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Memory    │  │  Reasoning  │  │   Personality       │ │
│  │ (Episodic/  │  │  (Deductive/│  │   (OCEAN/PAD)       │ │
│  │  Semantic)  │  │  Analogical)│  │                     │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
└─────────┼────────────────┼────────────────────┼────────────┘
          │                │                    │
          │ persist()      │                    │
          ▼                │                    │
┌─────────────────────────────────────────────────────────────┐
│                    beebotos-kernel                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Storage   │  │  Scheduler  │  │   Memory/Resource   │ │
│  │ (KV Store)  │  │             │  │                     │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

#### 业务逻辑关系

| Social Brain 功能 | Kernel 支持 | 说明 |
|------------------|------------|------|
| 记忆持久化 | Storage 模块 | 将记忆数据存储到工作空间 |
| 推理计算 | Scheduler | 调度推理任务到线程池 |
| 情感计算 | Resource 限制 | 限制计算资源使用 |
| NEAT 进化 | WASM 运行时 | 在沙箱中运行神经网络 |

#### 数据流

```
Social Brain Memory Layer:
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Short-term     │────▶│  Consolidation  │────▶│   Long-term     │
│    Memory       │     │    Process      │     │     Storage     │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                         │
                                                         ▼
                                              ┌─────────────────────┐
                                              │  Kernel::Storage    │
                                              │  workspace/{id}/    │
                                              │  memory/            │
                                              └─────────────────────┘
```

### 5.3 与 beebotos-gateway 模块的关系

```
┌─────────────────────────────────────────────────────────────┐
│                 beebotos-gateway                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │  WebSocket  │  │ Rate Limiter│  │    Middleware       │ │
│  │   Server    │  │             │  │   (Auth/CORS)       │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
└─────────┼────────────────┼────────────────────┼────────────┘
          │                │                    │
          │ create_session()                  │ verify_token()
          ▼                │                    ▼
┌─────────────────────────────────────────────────────────────┐
│                    beebotos-kernel                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Security  │  │  Scheduler  │  │   Audit Log         │ │
│  │   Manager   │  │             │  │                     │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

#### 业务逻辑关系

| Gateway 功能 | Kernel 支持 | 说明 |
|-------------|------------|------|
| 身份验证 | Security Manager | ACL 和 RBAC 检查 |
| 速率限制 | RateLimiter | 与 Kernel 资源限制联动 |
| 审计日志 | Audit Log | 记录所有 API 访问 |
| 会话管理 | Security Context | 注册/注销安全上下文 |

---

## 函数接口定义

### 6.1 公共 API

```rust
// ==================== Kernel Builder ====================

pub struct KernelBuilder;

impl KernelBuilder {
    pub fn new() -> Self;
    pub fn with_scheduler(mut self, config: SchedulerConfig) -> Self;
    pub fn with_security_policy<P: SecurityPolicy + 'static>(mut self, policy: P) -> Self;
    pub fn with_memory_config(mut self, config: MemoryConfig) -> Self;
    pub fn with_wasm(mut self, enabled: bool) -> Self;
    pub fn with_tee(mut self, provider: ()) -> Self;
    pub fn with_max_agents(mut self, max: usize) -> Self;
    pub fn build(self) -> Result<Kernel>;
    pub fn boot_and_build(self, boot_info: &BootInfo) -> Result<Kernel>;
}

// ==================== Kernel 核心 ====================

impl Kernel {
    // 生命周期管理
    pub async fn start(&self) -> Result<()>;
    pub async fn stop(&self);
    pub async fn force_stop(&self);
    pub fn is_running(&self) -> bool;
    
    // 任务管理
    pub async fn spawn_task<F>(
        &self,
        name: impl Into<String>,
        priority: Priority,
        capabilities: CapabilitySet,
        f: F,
    ) -> Result<TaskId>
    where F: Future<Output = Result<()>> + Send + 'static;
    
    // 安全管理
    pub fn register_agent(&mut self, context: SecurityContext);
    
    // 系统调用
    pub async fn syscall(
        &self,
        number: u64,
        args: SyscallArgs,
        caller: AgentId,
    ) -> SyscallResult;
    
    // WASM 支持
    pub fn wasm_engine(&self) -> Option<&WasmEngine>;
    pub fn compile_wasm(&self, wasm_bytes: &[u8]) -> Result<wasmtime::Module>;
    pub fn instantiate_wasm(&self, module: &wasmtime::Module) -> Result<WasmInstance>;
    
    // 统计信息
    pub async fn scheduler_stats(&self) -> SchedulerStats;
    pub fn memory_stats(&self) -> MemorySnapshot;
    pub async fn stats(&self) -> KernelStats;
}

// ==================== 调度器 ====================

impl Scheduler {
    pub fn new(config: SchedulerConfig) -> Self;
    pub fn start_with_executor(&mut self) -> Result<(), SchedulerError>;
    pub async fn stop(&self);
    
    pub async fn spawn<F>(
        &self,
        name: impl Into<String>,
        priority: Priority,
        capabilities: CapabilitySet,
        f: F,
    ) -> Result<TaskId, SchedulerError>
    where F: Future<Output = KernelResult<()>> + Send + 'static;
    
    pub async fn cancel(&self, task_id: TaskId) -> bool;
    pub async fn await_task(&self, task_id: TaskId) -> KernelResult<()>;
    pub async fn get_task_info(&self, task_id: TaskId) -> Option<TaskInfo>;
    pub async fn list_tasks(&self) -> Vec<TaskInfo>;
    pub async fn list_tasks_by_state(&self, state: TaskState) -> Vec<TaskInfo>;
    pub async fn stats(&self) -> SchedulerStats;
    pub async fn queue_length(&self) -> usize;
    pub async fn running_count(&self) -> usize;
}

// ==================== 能力系统 ====================

impl CapabilitySet {
    pub fn empty() -> Self;
    pub fn standard() -> Self;
    pub fn full() -> Self;
    
    pub fn with_level(mut self, level: CapabilityLevel) -> Self;
    pub fn with_permission(mut self, perm: impl Into<String>) -> Self;
    pub fn with_expiration(mut self, expires_at: u64) -> Self;
    
    pub fn has(&self, level: CapabilityLevel) -> bool;
    pub fn has_permission(&self, perm: &str) -> bool;
    pub fn is_expired(&self) -> bool;
    pub fn verify(&self, required: CapabilityLevel) -> Result<()>;
    
    pub fn intersect(&self, other: &Self) -> Self;
    pub fn union(&self, other: &Self) -> Self;
}

// ==================== 系统调用分发器 ====================

impl SyscallDispatcher {
    pub fn new() -> Self;
    pub fn register(&mut self, num: SyscallNumber, handler: Box<dyn SyscallHandler>);
    
    pub async fn dispatch(
        &self,
        num: u64,
        args: SyscallArgs,
        caller: AgentId,
    ) -> SyscallResult;
    
    pub async fn dispatch_with_context(
        &self,
        num: u64,
        args: SyscallArgs,
        ctx: SyscallContext,
    ) -> SyscallResult;
}

// ==================== 消息路由器 ====================

impl MessageRouter {
    pub fn new() -> Self;
    pub fn register_agent(&self, agent_id: String) -> mpsc::UnboundedReceiver<MessageEnvelope>;
    pub fn unregister_agent(&self, agent_id: &str);
    pub fn route(&self, message: MessageEnvelope) -> KernelResult<()>;
    pub fn get_mailbox(&self, agent_id: &str) -> Option<AgentMailbox>;
    pub fn stats(&self) -> RouterStats;
    pub fn agent_stats(&self, agent_id: &str) -> Option<MailboxStats>;
    pub fn list_agents(&self) -> Vec<String>;
}

pub fn global_router() -> Arc<MessageRouter>;

// ==================== WASM 引擎 ====================

impl WasmEngine {
    pub fn new(config: EngineConfig) -> KernelResult<Self>;
    pub fn compile(&self, wasm_bytes: &[u8]) -> KernelResult<Module>;
    pub fn compile_cached(&self, name: &str, wasm_bytes: &[u8]) -> KernelResult<Module>;
    pub fn instantiate(&self, module: &Module) -> KernelResult<WasmInstance>;
    pub fn instantiate_wasi(&self, module: &Module, agent_id: &str, caps: Option<&WasiCapabilities>) -> KernelResult<WasmInstance>;
    pub fn precompile(&self, wasm_bytes: &[u8]) -> KernelResult<Vec<u8>>;
    pub fn load_precompiled(&self, name: &str, serialized: &[u8]) -> KernelResult<Module>;
    pub fn cache_stats(&self) -> CacheStats;
    pub fn clear_cache(&self);
}

// ==================== 资源管理 ====================

impl ResourceManager {
    pub fn new(global_limits: ResourceLimits) -> Self;
    pub fn set_process_limits(&mut self, pid: u32, limits: ResourceLimits);
    pub fn get_process_limits(&self, pid: u32) -> Option<&ResourceLimits>;
    pub fn update_usage(&mut self, pid: u32, usage: ResourceUsage);
    pub fn get_usage(&self, pid: u32) -> Option<&ResourceUsage>;
    pub fn check_process_resources(&self, pid: u32) -> ResourceStatus;
    pub fn cleanup_process(&mut self, pid: u32);
}

// ==================== 存储管理 ====================

pub fn global_storage() -> Arc<StorageManager>;
pub fn workspace_key(agent_id: &str, path: &str) -> String;

impl StorageManager {
    pub fn new(config: StorageConfig) -> Self;
    pub fn register_backend(&mut self, name: String, backend: Box<dyn StorageBackend>);
    pub fn put(&mut self, key: &str, data: &[u8]) -> Result<(), StorageError>;
    pub fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
    pub fn delete(&mut self, key: &str) -> Result<(), StorageError>;
    pub fn list(&self, prefix: &str) -> Result<Vec<String>, StorageError>;
    pub fn exists(&self, key: &str) -> Result<bool, StorageError>;
}

// ==================== 安全管理 ====================

impl SecurityManager {
    pub fn new() -> Self;
    pub fn with_audit_config(config: AuditConfig) -> KernelResult<Self>;
    pub fn register_policy(&mut self, policy: Box<dyn SecurityPolicy>);
    pub fn register(&self, context: SecurityContext);
    pub fn unregister(&self, user_id: &str);
    pub fn request_access(&self, subject: &SecurityContext, object: &str, action: AccessAction) -> AccessDecision;
    pub fn check_capability(&self, subject: &SecurityContext, capability: &Capability) -> bool;
    pub fn flush_audit_log(&self) -> KernelResult<()>;
}
```

---

## 数据流分析

### 7.1 Agent 创建流程

```
User Request
     │
     ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Gateway   │────▶│   Agents    │────▶│  SpawnEngine │
│   (Auth)    │     │  (Runtime)  │     │              │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                                │
                     ┌──────────────────────────┘
                     │ syscall(SpawnAgent)
                     ▼
            ┌─────────────────┐
            │     Kernel      │
            │  ┌───────────┐  │
            │  │ Syscall   │  │
            │  │ Dispatcher│  │
            │  └─────┬─────┘  │
            │  ┌─────┴─────┐  │
            │  │  Handler  │  │
            │  │(Capability│  │
            │  │  Check)   │  │
            │  └─────┬─────┘  │
            │  ┌─────┴─────┐  │
            │  │  Agent    │  │
            │  │  Registry │  │
            │  └───────────┘  │
            └─────────────────┘
                     │
                     ▼
            ┌─────────────────┐
            │   Scheduler     │
            │   spawn_task()  │
            └─────────────────┘
```

### 7.2 消息传递流程

```
Agent A                      Kernel                        Agent B
   │                          │                             │
   │     SendMessage syscall  │                             │
   │─────────────────────────▶│                             │
   │                          │                             │
   │                          │  ┌─────────────────────┐    │
   │                          │  │ 1. Capability Check │    │
   │                          │  │ 2. Rate Limit Check │    │
   │                          │  └──────────┬──────────┘    │
   │                          │             │               │
   │                          │  ┌──────────▼──────────┐    │
   │                          │  │   MessageRouter     │    │
   │                          │  │      route()        │    │
   │                          │  └──────────┬──────────┘    │
   │                          │             │               │
   │                          │  ┌──────────▼──────────┐    │
   │                          │  │   Agent B Mailbox   │────┼──▶
   │                          │  │      deliver()      │    │
   │                          │  └─────────────────────┘    │
   │                          │                             │
   │◀─────────────────────────│        Response            │
   │                          │                             │
```

### 7.3 文件访问流程

```
Agent
   │
   │  ReadFile/WriteFile syscall
   │
   ▼
┌────────────────────────────────────────┐
│              Kernel                    │
│  ┌────────────────────────────────┐    │
│  │      Syscall Handler           │    │
│  │  1. Validate Capability (L1/L2)│    │
│  │  2. Validate Path (sandbox)    │    │
│  │  3. Check Resource Limits      │    │
│  └──────────────┬─────────────────┘    │
│                 │                      │
│  ┌──────────────▼─────────────────┐    │
│  │     Storage Manager            │    │
│  │  workspace/{agent_id}/{path}   │    │
│  └──────────────┬─────────────────┘    │
│                 │                      │
│  ┌──────────────▼─────────────────┐    │
│  │    Storage Backend             │    │
│  │  (Memory/RocksDB)              │    │
│  └────────────────────────────────┘    │
└────────────────────────────────────────┘
```

---

## 安全模型

### 8.1 多层安全架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Security Layers                          │
├─────────────────────────────────────────────────────────────┤
│ Layer 5: Application Security (Agent-level policies)        │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: System Call Security (Capability checks)           │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Resource Security (Resource limits & isolation)    │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Memory Security (Sandbox & WASI)                   │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Kernel Security (ACL, Audit, TEE)                  │
└─────────────────────────────────────────────────────────────┘
```

### 8.2 Capability 安全检查流程

```rust
async fn handle_syscall(number: u64, args: SyscallArgs, ctx: &SyscallContext) -> SyscallResult {
    // 1. 获取所需能力等级
    let required_level = required_capability(number);
    
    // 2. 检查调用者能力
    if ctx.capability_level < required_level {
        return SyscallResult::Error(SyscallError::PermissionDenied);
    }
    
    // 3. 检查能力是否过期
    if let Some(registry) = CAPABILITY_REGISTRY.read().as_ref() {
        let caps = registry.read();
        if caps.is_expired() {
            return SyscallResult::Error(SyscallError::PermissionDenied);
        }
    }
    
    // 4. 执行具体处理
    handler.handle(args, ctx).await
}
```

### 8.3 审计日志

```rust
pub struct AuditEntry {
    pub timestamp: u64,
    pub agent_id: String,
    pub action: String,
    pub object: String,
    pub decision: AccessDecision,
    pub metadata: HashMap<String, String>,
}

// 自动记录所有安全相关操作
pub fn log_access_attempt(
    &self,
    subject: &SecurityContext,
    object: &str,
    action: AccessAction,
    decision: AccessDecision,
);
```

---

## 配置示例

### 9.1 生产环境配置

```rust
use beebotos_kernel::{
    KernelBuilder, SchedulerConfig, EngineConfig,
    capabilities::CapabilitySet,
};

let kernel = KernelBuilder::new()
    .with_max_agents(10_000)
    .with_scheduler(SchedulerConfig {
        max_concurrent: 10_000,
        time_slice_ms: 50,
        enable_preemption: true,
        default_priority: Priority::Normal,
        num_workers: num_cpus::get(),
        enable_work_stealing: true,
        enable_cpu_affinity: true,
    })
    .with_wasm(true)
    .with_memory_config(MemoryConfig {
        max_heap_size: 1024 * 1024 * 1024, // 1GB
        enable_safety_checks: true,
    })
    .build()?;
```

### 9.2 开发环境配置

```rust
let kernel = KernelBuilder::new()
    .with_max_agents(100)
    .with_scheduler(SchedulerConfig::development())
    .with_wasm(true)
    .build()?;
```

---

## 总结

BeeBotOS Kernel 作为系统的核心层，提供了：

1. **稳定的运行基础**：抢占式调度器确保任务公平执行
2. **强大的安全保证**：11 级 Capability 模型 + 审计日志
3. **灵活的系统调用**：29 个系统调用支持上层所有功能
4. **高效的资源管理**：资源限制、熔断器、监控统计
5. **完善的隔离机制**：WASM 沙箱、工作空间隔离、内存安全

Kernel 通过清晰的接口与 Agents、Social Brain、Gateway 模块协作，共同构建了一个安全、高效、可扩展的自主操作系统。
