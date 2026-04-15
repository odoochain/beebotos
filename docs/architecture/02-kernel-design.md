# 内核设计

> **Layer 1: 系统内核架构详解**

---

## 内核架构

```
┌─────────────────────────────────────────────────────────────┐
│                    System Call Interface                     │
│                    (64 个系统调用)                           │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │Scheduler │ │ Security │ │  WASM    │ │  Memory  │    │
│  │  调度器   │ │  安全    │ │  虚拟机  │ │  内存    │    │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘    │
│       │            │            │            │            │
│       └────────────┴────────────┴────────────┘            │
│                          │                                 │
│                   ┌──────┴──────┐                         │
│                   │   Device    │                         │
│                   │   设备驱动   │                         │
│                   └─────────────┘                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 调度器 (Scheduler)

### 设计目标

- 支持 1000+ Agent 并发
- 公平分配 CPU 时间
- 支持优先级抢占

### CFS + MLFQ 混合算法

```rust
pub struct Scheduler {
    // MLFQ: 多级反馈队列
    mlfq: [VecDeque<Task>; 4],
    
    // CFS: 完全公平调度
    cfs_queue: BTreeMap<u64, Task>,
    
    config: SchedulerConfig,
}
```

### 优先级

| 级别 | 名称 | 时间片 | 抢占 |
|------|------|--------|------|
| 0 | Low | 200ms | 否 |
| 1 | Normal | 100ms | 否 |
| 2 | High | 50ms | 是 |
| 3 | Critical | 10ms | 是 |

---

## 安全模块 (Security)

### 10层 Capability 模型

```rust
pub enum CapabilityLevel {
    L0_LocalCompute = 0,
    L1_FileRead = 1,
    L2_FileWrite = 2,
    L3_NetworkOut = 3,
    L4_NetworkIn = 4,
    L5_SpawnLimited = 5,
    L6_SpawnUnlimited = 6,
    L7_ChainRead = 7,
    L8_ChainWriteLow = 8,
    L9_ChainWriteHigh = 9,
}
```

### 权限检查

```rust
impl SecurityManager {
    pub fn check_access(
        &self,
        subject: &AgentId,
        object: &ObjectId,
        action: Action,
    ) -> AccessDecision {
        // 1. 检查 Capability
        // 2. 检查 ACL
        // 3. 记录审计日志
    }
}
```

---

## WASM 虚拟机

### 沙箱隔离

- 内存隔离
- 执行隔离
- Gas 限制

### 宿主函数

```rust
pub const HOST_FUNCTIONS: &[(&str, HostFunc)] = &[
    ("host_log", host_log),
    ("host_send_message", host_send_message),
    ("host_query_chain", host_query_chain),
    // ... 共 64 个
];
```

---

## 内存管理

### Buddy System

用于大对象 (>4KB) 分配。

### Slab Allocator

用于小对象 (≤4KB) 快速分配。

---

## 系统调用

共 64 个系统调用，分为：

| 类别 | 数量 | 说明 |
|------|------|------|
| Agent 管理 | 10 | 创建、终止、查询 |
| 通信 | 10 | 消息发送接收 |
| 资源 | 10 | 内存、CPU |
| 文件 | 10 | 文件操作 |
| 区块链 | 10 | 链上交互 |
| 记忆 | 10 | 记忆管理 |
| 系统 | 4 | 时间、日志 |

---

**最后更新**: 2026-03-13
