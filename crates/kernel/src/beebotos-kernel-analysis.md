# BeeBotOS Kernel 模块工作原理与代码逻辑分析

## 目录

1. [概述](#概述)
2. [架构设计](#架构设计)
3. [核心模块详解](#核心模块详解)
   - [启动流程 (boot)](#启动流程-boot)
   - [任务调度器 (scheduler)](#任务调度器-scheduler)
   - [能力系统 (capabilities)](#能力系统-capabilities)
   - [安全管理 (security)](#安全管理-security)
   - [系统调用 (syscalls)](#系统调用-syscalls)
   - [内存管理 (memory)](#内存管理-memory)
   - [任务管理 (task)](#任务管理-task)
   - [WASM 运行时 (wasm)](#wasm-运行时-wasm)
   - [资源管理 (resource)](#资源管理-resource)
   - [设备管理 (device)](#设备管理-device)
   - [进程间通信 (ipc)](#进程间通信-ipc)
4. [关键数据结构](#关键数据结构)
5. [工作流程分析](#工作流程分析)
6. [安全模型](#安全模型)
7. [总结](#总结)

---

## 概述

BeeBotOS Kernel 是 BeeBotOS 操作系统的核心内核模块，采用 Rust 语言编写，提供以下核心功能：

- **抢占式任务调度**：基于优先级的多任务调度系统
- **能力安全模型**：11层能力级别访问控制
- **系统调用接口**：29个标准系统调用
- **资源管理**：CPU、内存、IO、网络等资源限制
- **WASM 运行时**：WebAssembly 执行环境
- **内存管理**：虚拟内存和分页管理

### 基础信息

| 属性 | 值 |
|------|-----|
| 模块名称 | beebotos-kernel |
| 版本 | 2.0.0 |
| 语言 | Rust (Edition 2021) |
| 许可证 | MIT |
| 依赖核心 | beebotos-core |

---

## 架构设计

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                      User Applications                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │   CLI   │  │  Web UI │  │ Gateway │  │  Agent Runtime  │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────────┬────────┘ │
└───────┼────────────┼────────────┼────────────────┼──────────┘
        │            │            │                │
        └────────────┴────────────┴────────────────┘
                         │
┌────────────────────────┼─────────────────────────────────────┐
│                   BeeBotOS Kernel                            │
│  ┌─────────────────────┼──────────────────────────────────┐ │
│  │              System Call Interface                      │ │
│  │   (29 syscalls: spawn, message, file, blockchain...)   │ │
│  └─────────────────────┼──────────────────────────────────┘ │
│                        │                                     │
│  ┌─────────────────────┼──────────────────────────────────┐ │
│  │              Capability Security System                 │ │
│  │   (L0-L10: 11-tier capability model)                   │ │
│  └─────────────────────┼──────────────────────────────────┘ │
│                        │                                     │
│  ┌─────────────┐  ┌────┴────┐  ┌──────────┐  ┌───────────┐ │
│  │  Scheduler  │  │  Task   │  │  Memory  │  │   WASM    │ │
│  │  (CFS/RR)   │  │ Manager │  │  Manager │  │  Engine   │ │
│  └─────────────┘  └─────────┘  └──────────┘  └───────────┘ │
│                                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌────────────────┐ │
│  │  Device │  │   IPC   │  │ Resource│  │    Security    │ │
│  │  Driver │  │ Channel │  │  Limit  │  │    (ACL/TEE)   │ │
│  └─────────┘  └─────────┘  └─────────┘  └────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 模块依赖关系

```
lib.rs (Kernel 主结构)
    ├── boot.rs          # 启动流程
    ├── scheduler/       # 任务调度器
    │   ├── mod.rs       # 调度器主逻辑
    │   ├── task.rs      # 任务控制块
    │   ├── fair.rs      # CFS 公平调度
    │   ├── queue.rs     # 任务队列
    │   └── resource.rs  # 资源调度
    ├── capabilities/    # 能力系统
    │   ├── mod.rs       # CapabilityManager
    │   ├── levels.rs    # L0-L10 能力级别
    │   ├── tokens.rs    # 能力令牌
    │   └── registry.rs  # 令牌注册表
    ├── security/        # 安全管理
    │   ├── mod.rs       # SecurityManager
    │   ├── acl.rs       # 访问控制列表
    │   └── sandbox/     # 沙箱
    ├── syscalls/        # 系统调用
    │   └── mod.rs       # SyscallDispatcher
    ├── memory/          # 内存管理
    ├── task/            # 任务管理
    ├── wasm/            # WASM 运行时
    ├── resource/        # 资源管理
    ├── device/          # 设备管理
    └── ipc/             # 进程间通信
```

---

## 核心模块详解

### 启动流程 (boot)

**文件**: `src/boot.rs`

启动流程负责初始化内核的各个子系统。

#### 启动信息结构

```rust
/// Boot information passed from bootloader
pub struct BootInfo {
    pub memory_map: &'static [MemoryRegion],    // 内存映射表
    pub cmd_line: &'static str,                  // 内核命令行参数
    pub bootloader_name: &'static str,           // Bootloader 名称
}

/// Memory region descriptor
pub struct MemoryRegion {
    pub start: u64,           // 起始地址
    pub size: u64,            // 大小
    pub region_type: MemoryRegionType,
}

pub enum MemoryRegionType {
    Usable,               // 可用 RAM
    Reserved,             // 保留
    AcpiReclaimable,      // ACPI 可回收
    AcpiNvs,              // ACPI NVS
    BadMemory,            // 坏内存
    Kernel,               // 内核代码/数据
    BootloaderReserved,   // Bootloader 保留
}
```

#### 启动序列

```rust
pub fn boot(_info: &BootInfo) -> Result<(), BootError> {
    // 1. Set up memory management
    // 2. Initialize interrupt handlers
    // 3. Set up scheduler
    // 4. Start system services
    
    tracing::info!("BeeBotOS Kernel booting...");
    Ok(())
}
```

**启动错误类型**:
- `MemoryInitFailed` - 内存初始化失败
- `InterruptSetupFailed` - 中断设置失败
- `SchedulerInitFailed` - 调度器初始化失败

---

### 任务调度器 (scheduler)

**文件**: `src/scheduler/mod.rs`, `src/scheduler/task.rs`, `src/scheduler/fair.rs`

采用**多级反馈队列 + CFS (Completely Fair Scheduler)** 混合调度算法。

#### 调度器配置

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub max_concurrent: usize,      // 最大并发任务数 (默认: 100)
    pub time_slice_ms: u64,         // 时间片毫秒数 (默认: 100)
    pub enable_preemption: bool,    // 启用抢占 (默认: true)
    pub default_priority: Priority, // 默认优先级 (默认: Normal)
}
```

#### 任务优先级

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    RealTime = 0,  // 实时优先级 (最高)
    High = 1,      // 高优先级
    Normal = 2,    // 正常优先级
    Low = 3,       // 低优先级
    Idle = 4,      // 空闲优先级 (最低)
}
```

#### 任务状态机

```
┌─────────┐     spawn      ┌─────────┐
│  New    │ ──────────────▶│  Ready  │
└─────────┘                └────┬────┘
                                │ schedule
                                ▼
┌─────────┐   complete    ┌─────────┐
│ Zombie  │ ◀──────────── │ Running │
└─────────┘               └────┬────┘
     ▲                         │
     │    unblock              │ block (IO/Sleep/Wait)
     └─────────────────────────┘
               ┌─────────┐
               │ Blocked │
               └─────────┘
```

#### 任务控制块 (TCB)

```rust
pub struct Task {
    pub id: TaskId,                    // 任务ID
    pub name: String,                  // 任务名称
    pub priority: Priority,            // 优先级
    pub state: TaskState,              // 状态
    pub vruntime: u64,                 // 虚拟运行时间 (CFS)
    pub cpu_time: Duration,            // CPU 使用时间
    pub created_at: Instant,           // 创建时间
    pub capabilities: CapabilitySet,   // 能力集
    pub deadline: Option<Instant>,     // EDF 截止时间
}

pub enum TaskState {
    Running,
    Ready,
    Blocked(BlockReason),
    Zombie,
}

pub enum BlockReason {
    Io,               // IO 等待
    Sleep,            // 睡眠
    WaitForEvent,     // 等待事件
    WaitForResource,  // 等待资源
}
```

#### CFS 公平调度算法

```rust
pub struct CFSScheduler {
    tasks: BTreeMap<u64, Task>,       // 按 vruntime 排序的任务
    current: Option<TaskId>,           // 当前任务
    time_slice: Duration,              // 时间片
    min_granularity: Duration,         // 最小粒度
    target_latency: Duration,          // 目标延迟
}
```

**CFS 核心逻辑**:

1. **虚拟运行时间计算**:
   ```rust
   pub fn update_vruntime(&mut self, task: &mut Task, elapsed: Duration) {
       let weight = self.get_weight(task.priority);
       let delta_vruntime = (elapsed.as_nanos() as u64 * 1024) / weight as u64;
       task.vruntime += delta_vruntime;
   }
   ```

2. **优先级权重表** (Nice -20 到 19):
   ```rust
   pub const PRIORITY_WEIGHTS: [u32; 40] = [
       88761, 71755, 56483, 46273, 36291,  // Nice -20 to -16
       29154, 23254, 18705, 14949, 11916,  // Nice -15 to -11
       9548, 7620, 6100, 4904, 3906,       // Nice -10 to -6
       3121, 2501, 1991, 1586, 1277,       // Nice -5 to -1
       1024, 820, 655, 526, 423,           // Nice 0 to 4
       335, 272, 215, 172, 137,            // Nice 5 to 9
       110, 87, 70, 56, 45,                // Nice 10 to 14
       36, 29, 23, 18, 15,                 // Nice 15 to 19
   ];
   ```

3. **时间片计算**:
   ```rust
   pub fn time_slice(&self, num_tasks: usize) -> Duration {
       if num_tasks == 0 {
           return self.time_slice;
       }
       let slice = self.target_latency / num_tasks as u32;
       slice.max(self.min_granularity)
   }
   ```

#### 调度器统计

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub tasks_submitted: u64,              // 提交任务数
    pub tasks_scheduled: u64,              // 调度任务数
    pub tasks_completed: u64,              // 完成任务数
    pub tasks_failed: u64,                 // 失败任务数
    pub average_wait_time_ms: f64,         // 平均等待时间
    pub average_execution_time_ms: f64,    // 平均执行时间
}
```

#### 速率限制器 (Token Bucket)

```rust
pub struct RateLimiter {
    tokens: Arc<Mutex<f64>>,    // 令牌数量
    rate: f64,                   // 每秒令牌产生率
    burst: f64,                  // 最大令牌数
}

impl RateLimiter {
    pub async fn acquire(&self, tokens: f64) -> bool {
        let mut current = self.tokens.lock().await;
        if *current >= tokens {
            *current -= tokens;
            true
        } else {
            false
        }
    }
}
```

---

### 能力系统 (capabilities)

**文件**: `src/capabilities/mod.rs`, `src/capabilities/levels.rs`, `src/capabilities/tokens.rs`

采用 **11 层能力级别模型 (L0-L10)**，实现细粒度访问控制。

#### 能力级别定义

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum CapabilityLevel {
    #[default]
    L0LocalCompute = 0,      // 本地计算 (沙盒)
    L1FileRead = 1,          // 文件系统只读
    L2FileWrite = 2,         // 文件系统读写
    L3NetworkOut = 3,        // 出站网络访问
    L4NetworkIn = 4,         // 入站网络访问
    L5SpawnLimited = 5,      // 有限子代理创建
    L6SpawnUnlimited = 6,    // 无限子代理创建
    L7ChainRead = 7,         // 区块链读取
    L8ChainWriteLow = 8,     // 区块链写入 (低价值)
    L9ChainWriteHigh = 9,    // 区块链写入 (高价值)
    L10SystemAdmin = 10,     // 系统管理
}
```

#### 能力级别详情

| 级别 | 名称 | 权限描述 | 推荐超时 | 最大风险值 |
|------|------|----------|----------|------------|
| L0 | Local Compute | 纯本地计算，无外部访问 | 30s | 0 |
| L1 | File Read | 只读文件系统访问 | 60s | 0 |
| L2 | File Write | 文件系统读写访问 | 60s | 0 |
| L3 | Network Out | 出站网络连接 | 120s | 0 |
| L4 | Network In | 入站网络连接 | 120s | 0 |
| L5 | Spawn Limited | 最多创建 10 个子代理 | 300s | 0 |
| L6 | Spawn Unlimited | 无限子代理创建 | 600s | 0 |
| L7 | Chain Read | 读取区块链状态和事件 | 60s | 0 |
| L8 | Chain Write Low | 执行低价值交易 (< 1 ETH) | 180s | 1 ETH |
| L9 | Chain Write High | 执行高价值交易 | 300s | ∞ |
| L10 | System Admin | 完整系统控制 | 600s | ∞ |

#### 特殊要求

- **L9/L10 需要 TEE** (可信执行环境)
- **L9 需要多签批准**

#### 能力集

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub max_level: CapabilityLevel,           // 最大能力级别
    pub permissions: HashSet<String>,         // 具体权限列表
    pub expires_at: Option<u64>,              // 过期时间
    pub delegable: bool,                      // 是否可委托
}
```

#### 能力集操作

```rust
impl CapabilitySet {
    // 预定义能力集
    pub fn empty() -> Self;        // 空能力集
    pub fn full() -> Self;         // 完整能力集 (系统代理)
    pub fn standard() -> Self;     // 标准代理能力集
    
    // 检查
    pub fn has(&self, level: CapabilityLevel) -> bool;
    pub fn has_permission(&self, perm: &str) -> bool;
    pub fn is_expired(&self) -> bool;
    pub fn verify(&self, required: CapabilityLevel) -> Result<()>;
    
    // 集合操作
    pub fn intersect(&self, other: &Self) -> Self;  // 交集
    pub fn union(&self, other: &Self) -> Self;      // 并集
}
```

#### 能力令牌

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub id: String,                    // 令牌ID
    pub agent_id: AgentId,             // 代理ID
    pub level: CapabilityLevel,        // 授予的级别
    pub created_at: u64,               // 创建时间
    pub expires_at: Option<u64>,       // 过期时间
    pub status: TokenStatus,           // 状态
    pub justification: String,         // 申请理由
}

pub enum TokenStatus {
    Pending,   // 待审批
    Active,    // 已激活
    Expired,   // 已过期
    Revoked,   // 已撤销
}
```

#### 能力管理器

```rust
pub struct CapabilityManager {
    agent_caps: HashMap<AgentId, CapabilitySet>,  // 代理能力映射
    registry: CapabilityRegistry,                  // 令牌注册表
}

impl CapabilityManager {
    pub fn assign(&mut self, agent_id: AgentId, caps: CapabilitySet);
    pub fn get(&self, agent_id: &AgentId) -> Option<&CapabilitySet>;
    pub fn revoke(&mut self, agent_id: &AgentId) -> Option<CapabilitySet>;
    pub fn check(&self, agent_id: &AgentId, level: CapabilityLevel) -> Result<()>;
    
    // 能力升级
    pub fn request_elevation(&mut self, agent_id: AgentId, request: CapabilityRequest) 
        -> Result<CapabilityToken>;
    pub fn approve_elevation(&mut self, token_id: &str) -> Result<CapabilityToken>;
}
```

#### 衰减能力 (时间衰减)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayingCapability {
    pub level: CapabilityLevel,
    pub granted_at: u64,
    pub decay_rate: DecayRate,
}

pub enum DecayRate {
    Slow,    // 1 级/天
    Normal,  // 1 级/小时
    Fast,    // 1 级/10分钟
}
```

---

### 安全管理 (security)

**文件**: `src/security/mod.rs`, `src/security/acl.rs`

#### 安全上下文

```rust
pub struct SecurityContext {
    pub user_id: String,                    // 用户ID
    pub group_id: String,                   // 组ID
    pub capabilities: Vec<Capability>,      // 能力列表
    pub clearance_level: ClearanceLevel,    // 安全许可级别
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearanceLevel {
    Public = 0,
    Internal = 1,
    Confidential = 2,
    Secret = 3,
    TopSecret = 4,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    FileRead,
    FileWrite,
    FileExecute,
    NetworkAccess,
    ProcessSpawn,
    ProcessKill,
    MemoryAllocate,
    DeviceAccess(String),
    SystemCall(String),
}
```

#### 安全策略接口

```rust
pub trait SecurityPolicy: Send + Sync {
    fn check_access(
        &self,
        subject: &SecurityContext,
        object: &str,
        action: AccessAction,
    ) -> AccessDecision;
    
    fn check_capability(&self, subject: &SecurityContext, capability: &Capability) -> bool;
}
```

#### 访问决策

```rust
pub enum AccessDecision {
    Allow,   // 允许
    Deny,    // 拒绝
    Ask,     // 询问用户
}

pub enum AccessAction {
    Read,
    Write,
    Execute,
    Delete,
    Create,
}
```

#### 安全管理器

```rust
pub struct SecurityManager {
    policies: Vec<Box<dyn SecurityPolicy>>,
    audit_log: AuditLog,
}

impl SecurityManager {
    pub fn register_policy(&mut self, policy: Box<dyn SecurityPolicy>);
    
    pub fn request_access(
        &mut self,
        subject: &SecurityContext,
        object: &str,
        action: AccessAction,
    ) -> AccessDecision {
        for policy in &self.policies {
            let decision = policy.check_access(subject, object, action);
            self.audit_log.log_access_attempt(subject, object, action, decision);
            
            match decision {
                AccessDecision::Deny => return AccessDecision::Deny,
                AccessDecision::Allow => continue,
                AccessDecision::Ask => return AccessDecision::Ask,
            }
        }
        AccessDecision::Allow
    }
}
```

#### 审计日志

```rust
pub struct AuditLog {
    entries: Vec<AuditEntry>,
    max_entries: usize,  // 默认 10000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub user_id: String,
    pub object: String,
    pub action: String,
    pub decision: String,
    pub context: serde_json::Value,
}
```

---

### 系统调用 (syscalls)

**文件**: `src/syscalls/mod.rs`

提供 **29 个系统调用**，按能力级别分组。

#### 系统调用号

```rust
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyscallNumber {
    // Agent 管理 (L5)
    SpawnAgent = 0,           // 创建代理
    TerminateAgent = 1,       // 终止代理
    
    // 消息通信 (L3)
    SendMessage = 2,          // 发送消息
    
    // 资源访问 (L1-L4)
    AccessResource = 3,       // 访问资源
    
    // 区块链/金融 (L8)
    ExecutePayment = 4,       // 执行支付
    BridgeToken = 19,         // 跨链桥接
    SwapToken = 20,           // 代币交换
    StakeToken = 21,          // 质押代币
    UnstakeToken = 22,        // 解除质押
    QueryBalance = 23,        // 查询余额
    
    // 内存/状态 (L0)
    QueryMemory = 5,          // 查询内存
    QueryState = 14,          // 查询状态
    UpdateState = 15,         // 更新状态
    
    // 能力管理 (L7)
    UpdateCapability = 6,     // 更新能力
    
    // 沙箱 (L6)
    EnterSandbox = 7,         // 进入沙箱
    ExitSandbox = 8,          // 退出沙箱
    
    // 文件系统 (L1-L2)
    ReadFile = 9,             // 读取文件
    WriteFile = 10,           // 写入文件
    ListFiles = 11,           // 列出文件
    
    // 工作区 (L4)
    CreateWorkspace = 12,     // 创建工作区
    DeleteWorkspace = 13,     // 删除工作区
    
    // 任务调度 (L4)
    ScheduleTask = 16,        // 调度任务
    CancelTask = 17,          // 取消任务
    QuerySchedule = 18,       // 查询调度
    
    // 证明 (L3)
    RequestAttestation = 24,  // 请求证明
    VerifyAttestation = 25,   // 验证证明
    
    // 日志/指标 (L0-L2)
    LogEvent = 26,            // 记录事件
    EmitMetric = 27,          // 发送指标
    QueryMetrics = 28,        // 查询指标
}
```

#### 系统调用参数与结果

```rust
#[derive(Debug, Clone, Default)]
pub struct SyscallArgs {
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
}

#[derive(Debug, Clone)]
pub enum SyscallResult {
    Success(u64),
    Error(SyscallError),
    Async(u64),  // 异步操作句柄
}

#[repr(i64)]
pub enum SyscallError {
    Success = 0,
    InvalidSyscall = -1,
    InvalidArgs = -2,
    PermissionDenied = -3,
    ResourceNotFound = -4,
    ResourceBusy = -5,
    OutOfMemory = -6,
    Timeout = -7,
    Cancelled = -8,
    InternalError = -9,
    NotImplemented = -10,
    QuotaExceeded = -11,
    InvalidCapability = -12,
}
```

#### 系统调用分发器

```rust
pub struct SyscallDispatcher {
    handlers: HashMap<SyscallNumber, Box<dyn SyscallHandler>>,
}

#[async_trait::async_trait]
pub trait SyscallHandler: Send + Sync {
    async fn handle(&self, args: SyscallArgs, ctx: &SyscallContext) -> SyscallResult;
}

pub struct SyscallContext {
    pub caller_id: String,
    pub capability_level: u8,
    pub workspace_id: String,
    pub session_id: String,
}
```

#### 系统调用能力要求

```rust
fn required_capability(&self, num: SyscallNumber) -> u8 {
    match num {
        // L0: 基本操作
        QueryMemory | LogEvent | QueryMetrics => 0,
        
        // L1: 文件操作
        ReadFile | ListFiles | QueryState => 1,
        
        // L2: 写操作
        WriteFile | UpdateState | EmitMetric => 2,
        
        // L3: 外部操作
        SendMessage | AccessResource | RequestAttestation | VerifyAttestation => 3,
        
        // L4: 管理
        CreateWorkspace | DeleteWorkspace | QuerySchedule | ScheduleTask | CancelTask => 4,
        
        // L5: 创建/终止
        SpawnAgent | TerminateAgent => 5,
        
        // L6: 沙箱控制
        EnterSandbox | ExitSandbox => 6,
        
        // L7: 能力更新
        UpdateCapability => 7,
        
        // L8: 金融操作
        ExecutePayment | BridgeToken | SwapToken | StakeToken | UnstakeToken | QueryBalance => 8,
    }
}
```

---

### 内存管理 (memory)

**文件**: `src/memory/mod.rs`

```rust
pub mod allocator;    // 内核分配器
pub mod paging;       // 分页管理
pub mod heap;         // 堆管理
pub mod vm;           // 虚拟内存
pub mod slab;         // Slab 分配器
pub mod mmap;         // 内存映射

pub use allocator::KernelAllocator;
pub use paging::PageTable;
pub use vm::VirtualMemory;

/// Initialize memory management
pub fn init() -> KernelResult<()> {
    tracing::info!("Initializing memory management");
    Ok(())
}

/// Memory allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Memory allocation failed: {:?}", layout);
}
```

---

### 任务管理 (task)

**文件**: `src/task/mod.rs`

```rust
pub mod process;   // 进程管理
pub mod thread;    // 线程管理
pub mod syscall;   // 任务系统调用
pub mod fork;      // 进程分叉
pub mod signal;    // 信号处理
pub mod wait;      // 等待机制

pub use process::Process;
pub use thread::Thread;

/// Task ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new(id: u64) -> Self { Self(id) }
    pub fn as_u64(&self) -> u64 { self.0 }
}
```

---

### WASM 运行时 (wasm)

**文件**: `src/wasm/mod.rs`, `src/wasm/engine.rs`

```rust
pub mod engine;       // WASM 引擎
pub mod instance;     // 实例管理
pub mod memory;       // WASM 内存
pub mod host_funcs;   // 宿主函数
pub mod metering;     // 计量/计费
pub mod trap;         // 陷阱处理
pub mod precompile;   // 预编译

pub use engine::WasmEngine;
pub use instance::WasmInstance;

/// Initialize WASM runtime
pub fn init() -> KernelResult<()> {
    tracing::info!("Initializing WASM runtime");
    Ok(())
}
```

#### WASM 引擎

```rust
pub struct WasmEngine {
    config: EngineConfig,
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub max_memory: usize,       // 最大内存 (默认: 128MB)
    pub max_execution_time: u64, // 最大执行时间
}

impl WasmEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }
    
    pub fn instantiate(&self, module: &[u8]) -> KernelResult<()> {
        tracing::info!("Instantiating WASM module");
        Ok(())
    }
}
```

---

### 资源管理 (resource)

**文件**: `src/resource/mod.rs`

提供资源限制和监控功能。

#### 资源使用统计

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_cpu_time: Option<Duration>,
    pub max_memory_bytes: Option<u64>,
    pub max_io_bytes: Option<u64>,
    pub max_network_bytes: Option<u64>,
    pub max_file_descriptors: Option<u32>,
    pub max_processes: Option<u32>,
}

impl ResourceLimits {
    pub fn default() -> Self {
        Self {
            max_cpu_time: Some(Duration::from_secs(3600)),
            max_memory_bytes: Some(1024 * 1024 * 1024),      // 1GB
            max_io_bytes: Some(1024 * 1024 * 1024 * 10),     // 10GB
            max_network_bytes: Some(1024 * 1024 * 1024 * 10), // 10GB
            max_file_descriptors: Some(1024),
            max_processes: Some(100),
        }
    }
}
```

#### 资源管理器

```rust
pub struct ResourceManager {
    process_usage: HashMap<u32, ResourceUsage>,
    process_limits: HashMap<u32, ResourceLimits>,
    global_limits: ResourceLimits,
}

impl ResourceManager {
    pub fn set_process_limits(&mut self, pid: u32, limits: ResourceLimits);
    pub fn update_usage(&mut self, pid: u32, usage: ResourceUsage);
    pub fn check_process_resources(&self, pid: u32) -> ResourceStatus;
}
```

---

### 设备管理 (device)

**文件**: `src/device/mod.rs`

```rust
pub mod timer;      // 定时器
pub mod network;    // 网络设备
pub mod storage;    // 存储设备
pub mod console;    // 控制台
pub mod pci;        // PCI 总线

pub use timer::Timer;
pub use network::NetworkDevice;

/// Device trait
pub trait Device {
    fn init(&mut self) -> KernelResult<()>;
    fn shutdown(&mut self) -> KernelResult<()>;
}
```

---

### 进程间通信 (ipc)

**文件**: `src/ipc/mod.rs`

```rust
pub mod channel;       // IPC 通道
pub mod message;       // 消息队列
pub mod shared_memory; // 共享内存
pub mod pipe;          // 管道

pub use channel::IpcChannel;
pub use message::MessageQueue;

/// Initialize IPC subsystem
pub fn init() -> KernelResult<()> {
    tracing::info!("Initializing IPC");
    Ok(())
}
```

---

## 关键数据结构

### Kernel 主结构

```rust
/// BeeBotOS Kernel
pub struct Kernel {
    scheduler: scheduler::Scheduler,              // 任务调度器
    security: security::SecurityManager,          // 安全管理器
    syscall_dispatcher: syscalls::SyscallDispatcher, // 系统调用分发器
    config: KernelConfig,                         // 内核配置
    running: std::sync::atomic::AtomicBool,       // 运行状态
}

/// Kernel configuration
pub struct KernelConfig {
    pub scheduler: scheduler::SchedulerConfig,
    pub security_policy: Box<dyn security::SecurityPolicy>,
    pub tee_provider: Option<()>,
    pub max_agents: usize,        // 默认: 1000
    pub audit_enabled: bool,      // 默认: true
}
```

### 错误类型层次

```rust
/// Kernel errors
#[derive(Error, Debug, Clone)]
pub enum KernelError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Insufficient capabilities: required {required:?}, have {current:?}")]
    InsufficientCapability { required: CapabilityLevel, current: CapabilityLevel },
    
    #[error("Security error: {0}")]
    Security(String),
    
    #[error("Capability expired")]
    CapabilityExpired,
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    #[error("Invalid syscall: {0}")]
    InvalidSyscall(u64),
    
    #[error("Scheduler error: {0}")]
    Scheduler(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    // ... more variants
}
```

---

## 工作流程分析

### 1. 内核启动流程

```
┌────────────────────────────────────────────────────────────┐
│ 1. Bootloader 加载内核                                      │
│    └── 传递 BootInfo (内存映射、命令行参数)                  │
├────────────────────────────────────────────────────────────┤
│ 2. 执行 boot() 函数                                         │
│    ├── 初始化内存管理 (memory::init)                        │
│    ├── 设置中断处理程序                                     │
│    ├── 初始化调度器 (scheduler::Scheduler::new)             │
│    ├── 初始化安全管理器 (security::SecurityManager::new)    │
│    ├── 初始化系统调用分发器                                 │
│    └── 启动系统服务                                         │
├────────────────────────────────────────────────────────────┤
│ 3. 内核进入运行状态                                         │
│    └── scheduler.start() 启动调度循环                       │
└────────────────────────────────────────────────────────────┘
```

### 2. 任务创建与调度流程

```
User Request
     │
     ▼
┌─────────────────┐
│ kernel.spawn_task│
│ (name, priority, │
│  capabilities, f)│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Task::new()     │◀── 创建任务控制块 (TCB)
│  - 分配 TaskId  │
│  - 设置优先级   │
│  - 初始化状态   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ scheduler.submit│◀── 提交到就绪队列
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 就绪队列         │◀── BinaryHeap 按优先级排序
│ (Ready Queue)   │
└────────┬────────┘
         │
         │ 调度循环 (每 time_slice_ms)
         ▼
┌─────────────────┐
│ 选择任务         │◀── pick_next(): 选择 vruntime 最小的
│ (CFS Algorithm) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 执行任务         │◀── 异步执行 Future
│ (Running)       │
└────────┬────────┘
         │
         ├──────────┬──────────┐
         │          │          │
         ▼          ▼          ▼
    ┌────────┐  ┌────────┐  ┌────────┐
    │Complete│  │ Block  │  │  Fail  │
    │ (完成) │  │ (阻塞) │  │ (失败) │
    └────┬───┘  └────┬───┘  └────┬───┘
         │           │           │
         ▼           ▼           ▼
    更新统计      移入阻塞队列   记录错误
    释放资源      等待 unblock
```

### 3. 系统调用处理流程

```
Agent
  │
  │ syscall(number, args, caller)
  ▼
┌────────────────────────────────┐
│ SyscallDispatcher::dispatch()  │
│                                │
│ 1. 创建 SyscallContext         │
│    - caller_id                 │
│    - capability_level          │
│    - workspace_id              │
│    - session_id                │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 2. 验证系统调用号              │
│    SyscallNumber::from_u64()   │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 3. 检查能力级别                │
│    required_capability()       │
│    ─────────────────────       │
│    if ctx.capability_level <   │
│       required_level {         │
│        return PermissionDenied │
│    }                           │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 4. 分发到处理器                │
│    handlers.get(&syscall_num)  │
│                                │
│    handler.handle(args, ctx)   │
│         │                      │
│         ▼                      │
│    ┌─────────────┐             │
│    │ 具体处理逻辑 │             │
│    └──────┬──────┘             │
│           │                    │
│           ▼                    │
│    SyscallResult::Success()    │
│    or                          │
│    SyscallResult::Error()      │
└────────────────────────────────┘
```

### 4. 能力检查流程

```
Agent Request
     │
     ▼
┌──────────────────────────────┐
│ capability_manager.check()   │
│    (agent_id, required_level)│
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ 1. 获取代理能力集            │
│    agent_caps.get(agent_id)  │
│                              │
│    if None:                  │
│       return NoCapabilities  │
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ 2. 检查是否过期              │
│    caps.is_expired()         │
│                              │
│    if expired:               │
│       return CapabilityExpired│
└──────────┬───────────────────┘
           │
           ▼
┌──────────────────────────────┐
│ 3. 验证能力级别              │
│    caps.verify(required)     │
│                              │
│    if max_level < required:  │
│       return InsufficientCapability
│                              │
│    else:                     │
│       return Ok(())          │
└──────────────────────────────┘
```

### 5. 能力升级流程

```
Agent
  │
  │ request_elevation(CapabilityRequest)
  │   - level: L8
  │   - justification: "Need to execute payment"
  │   - duration_seconds: 3600
  ▼
┌─────────────────────────────────────────┐
│ CapabilityManager::request_elevation()  │
└──────────┬──────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│ 1. 检查当前能力                         │
│    if current.has(request.level):       │
│       return active token               │
└──────────┬──────────────────────────────┘
           │ No, need elevation
           ▼
┌─────────────────────────────────────────┐
│ 2. 创建待审批令牌                       │
│    CapabilityToken::new_pending()       │
│    - status: Pending                    │
│    - justification: 保存                │
└──────────┬──────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│ 3. 注册到注册表                         │
│    registry.register(token)             │
└──────────┬──────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│ 4. 等待审批                             │
│    (返回 Pending Token 给用户)          │
└──────────┬──────────────────────────────┘
           │
           │ Admin Approval
           ▼
┌─────────────────────────────────────────┐
│ 5. 审批                                 │
│    approve_elevation(token_id)          │
│    registry.approve()                   │
│    - status: Active                     │
└─────────────────────────────────────────┘
```

---

## 安全模型

### 多层安全架构

```
┌─────────────────────────────────────────────┐
│ Layer 5: Application Security               │
│  - Agent code review                        │
│  - WASM sandbox                             │
├─────────────────────────────────────────────┤
│ Layer 4: System Call Security               │
│  - Capability-based access control          │
│  - 29 syscalls with level requirements      │
├─────────────────────────────────────────────┤
│ Layer 3: Resource Security                  │
│  - Resource quotas (CPU, memory, IO)        │
│  - Cgroup-based isolation                   │
├─────────────────────────────────────────────┤
│ Layer 2: Capability Security                │
│  - L0-L10 capability levels                 │
│  - Token-based elevation                    │
│  - Time-decaying capabilities               │
├─────────────────────────────────────────────┤
│ Layer 1: Access Control                     │
│  - SecurityPolicy trait                     │
│  - ACL (Access Control List)                │
│  - Audit logging                            │
├─────────────────────────────────────────────┤
│ Layer 0: Hardware Security                  │
│  - TEE (Trusted Execution Environment)      │
│  - Memory protection                        │
└─────────────────────────────────────────────┘
```

### 安全边界

| 层级 | 边界 | 控制机制 |
|------|------|----------|
| L0 | 计算边界 | WASM 沙箱、内存隔离 |
| L1-L2 | 存储边界 | 文件权限、只读/读写区分 |
| L3-L4 | 网络边界 | 出站/入站访问控制 |
| L5-L6 | 代理边界 | 子代理创建限制 |
| L7-L9 | 链上边界 | 区块链读写、价值限制 |
| L10 | 系统边界 | 完全系统控制 |

---

## 总结

### 核心特性

1. **抢占式多任务调度**
   - CFS 公平调度算法
   - 5 级优先级 (RealTime, High, Normal, Low, Idle)
   - 时间片轮转 + 虚拟运行时间

2. **11层能力安全模型**
   - L0-L10 细粒度访问控制
   - 能力令牌 + 升级机制
   - 时间衰减能力

3. **29个系统调用**
   - 按能力级别分组
   - 异步系统调用支持
   - 完整的错误处理

4. **资源管理**
   - CPU、内存、IO、网络限制
   - Cgroup 集成
   - 资源使用统计

5. **WASM 运行时**
   - 安全的沙箱执行环境
   - 内存和执行时间限制
   - 宿主函数支持

### 模块关系图

```
┌────────────────────────────────────────────────────────────┐
│                        Kernel                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Scheduler   │◀─│    Task      │◀─│   Process    │     │
│  │  (CFS/RR)    │  │  Control     │  │   Thread     │     │
│  └──────┬───────┘  └──────────────┘  └──────────────┘     │
│         │                                                   │
│  ┌──────┴───────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Capability  │◀─│   Security   │◀─│    ACL       │     │
│  │   Manager    │  │   Manager    │  │   Audit      │     │
│  └──────┬───────┘  └──────────────┘  └──────────────┘     │
│         │                                                   │
│  ┌──────┴───────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Syscall    │◀─│    WASM      │◀─│   Engine     │     │
│  │  Dispatcher  │  │   Runtime    │  │  Instance    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │    Memory    │  │   Resource   │  │    Device    │     │
│  │   (VM/Heap)  │  │   (Limits)   │  │   (Driver)   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└────────────────────────────────────────────────────────────┘
```

### 技术栈

| 组件 | 用途 |
|------|------|
| Tokio | 异步运行时 |
| async-trait | 异步 trait |
| serde | 序列化/反序列化 |
| ethers | 区块链交互 |
| crossbeam | 并发原语 |
| sysinfo | 系统信息 |
| num_cpus | CPU 信息 |

---

*文档版本: 1.0*  
*最后更新: 2026-03-20*  
*对应代码版本: beebotos-kernel v1.0.0*
