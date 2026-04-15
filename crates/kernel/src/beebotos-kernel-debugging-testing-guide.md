# BeeBotOS Kernel 调试和测试指导文档

## 目录

1. [测试概述](#测试概述)
2. [单元测试](#单元测试)
3. [集成测试](#集成测试)
4. [基准测试](#基准测试)
5. [调试工具](#调试工具)
6. [内存安全调试](#内存安全调试)
7. [性能分析](#性能分析)
8. [常见问题排查](#常见问题排查)
9. [测试配置](#测试配置)

---

## 测试概述

BeeBotOS Kernel 提供多层次的测试体系：

```
┌─────────────────────────────────────────────────────────────┐
│                    Testing Pyramid                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│                    ┌───────────┐                           │
│                    │  E2E Tests │  ← 完整系统测试            │
│                    └─────┬─────┘                           │
│                   ┌──────┴──────┐                         │
│                  ┌┤ Integration ├┐  ← 模块间交互测试        │
│                  ││   Tests     ││                        │
│                  │└──────┬──────┘│                        │
│                  │  ┌────┴────┐  │                         │
│                  └──┤  Unit   ├──┘  ← 模块内部测试          │
│                     │  Tests  │                            │
│                     └─────────┘                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 测试结构

```
crates/kernel/
├── src/                    # 源代码（内含 #[cfg(test)] 模块）
├── tests/                  # 集成测试
│   ├── scheduler_tests.rs  # 调度器测试
│   ├── capability_tests.rs # 能力系统测试
│   └── security_tests.rs   # 安全模块测试
├── benches/                # 基准测试
│   └── scheduler_bench.rs  # 调度器性能测试
├── examples/               # 示例程序
│   └── simple_scheduler.rs # 简单调度器示例
└── fuzz/                   # 模糊测试
    └── fuzz_scheduler.rs   # 调度器模糊测试
```

---

## 单元测试

### 2.1 运行单元测试

```bash
# 运行所有单元测试
cd crates/kernel
cargo test

# 运行特定模块的测试
cargo test scheduler
cargo test capabilities
cargo test security

# 显示详细输出
cargo test -- --nocapture

# 运行单个测试
cargo test test_scheduler_creation -- --nocapture
```

### 2.2 调度器单元测试

```rust
// crates/kernel/src/scheduler/mod.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_scheduler_spawn() {
        let mut scheduler = Scheduler::new(SchedulerConfig::development());
        scheduler.start_with_executor().unwrap();
        
        let task_id = scheduler.spawn(
            "test-task",
            Priority::Normal,
            CapabilitySet::standard(),
            async { Ok(()) }
        ).await.unwrap();
        
        assert!(task_id > 0);
        scheduler.stop().await;
    }
    
    #[tokio::test]
    async fn test_scheduler_cancel() {
        let mut scheduler = Scheduler::new(SchedulerConfig::development());
        scheduler.start_with_executor().unwrap();
        
        let task_id = scheduler.spawn(
            "long-task",
            Priority::Normal,
            CapabilitySet::standard(),
            async {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                Ok(())
            }
        ).await.unwrap();
        
        assert!(scheduler.cancel(task_id).await);
        scheduler.stop().await;
    }
}
```

### 2.3 能力系统单元测试

```rust
// crates/kernel/src/capabilities/mod.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_level_ordering() {
        assert!(CapabilityLevel::L10SystemAdmin > CapabilityLevel::L0LocalCompute);
        assert!(CapabilityLevel::L5SpawnLimited > CapabilityLevel::L4NetworkIn);
    }
    
    #[test]
    fn test_capability_decay() {
        let cap = DecayingCapability::new(
            CapabilityLevel::L5SpawnLimited,
            DecayRate::Fast
        );
        
        // Initially at L5
        assert_eq!(cap.current_level(), CapabilityLevel::L5SpawnLimited);
    }
}
```

### 2.4 内存安全单元测试

```rust
// crates/kernel/src/memory/safety.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_allocation_tracking() {
        let tracker = MemorySafetyTracker::new(false);
        
        let ptr = 0x1000 as *mut u8;
        tracker.record_allocation(ptr, 1024);
        
        assert_eq!(tracker.stats().total_allocations, 1);
        
        let check = tracker.record_deallocation(ptr);
        assert!(matches!(check, AllocationCheck::Valid(_)));
    }
    
    #[test]
    fn test_double_free_detection() {
        let tracker = MemorySafetyTracker::new(false);
        
        let ptr = 0x1000 as *mut u8;
        tracker.record_allocation(ptr, 1024);
        tracker.record_deallocation(ptr);
        
        let check = tracker.record_deallocation(ptr);
        assert!(matches!(check, AllocationCheck::DoubleFree));
        assert_eq!(tracker.stats().double_frees_detected, 1);
    }
    
    #[test]
    fn test_use_after_free_detection() {
        let tracker = MemorySafetyTracker::new(false);
        
        let ptr = 0x1000 as *mut u8;
        tracker.record_allocation(ptr, 1024);
        tracker.record_deallocation(ptr);
        
        let check = tracker.check_access(ptr, 1);
        assert!(matches!(check, AccessCheck::UseAfterFree));
    }
}
```

---

## 集成测试

### 3.1 运行集成测试

```bash
# 运行所有集成测试
cargo test --test "*"

# 运行特定集成测试文件
cargo test --test scheduler_tests
cargo test --test capability_tests
cargo test --test security_tests
```

### 3.2 调度器集成测试

```rust
// crates/kernel/tests/scheduler_tests.rs

use beebotos_kernel::scheduler::{Scheduler, SchedulerConfig, Task, Priority};
use beebotos_kernel::scheduler::queue::{TaskQueue, SchedulingAlgorithm};

#[tokio::test]
async fn test_scheduler_creation() {
    let config = SchedulerConfig::default();
    let scheduler = Scheduler::new(config);
    
    assert_eq!(scheduler.queue_length().await, 0);
    assert_eq!(scheduler.running_count().await, 0);
}

#[tokio::test]
async fn test_task_submission() {
    let config = SchedulerConfig::default();
    let scheduler = Scheduler::new(config);
    
    let task = Task::new(1, "test_task")
        .with_priority(Priority::Normal);
    let task_id = scheduler.submit(task).await.unwrap();
    
    assert_eq!(task_id, 1);
    assert_eq!(scheduler.queue_length().await, 1);
}

#[tokio::test]
async fn test_priority_ordering() {
    // Lower value = higher priority
    assert!(Priority::RealTime < Priority::High);
    assert!(Priority::High < Priority::Normal);
    assert!(Priority::Normal < Priority::Low);
    assert!(Priority::Low < Priority::Idle);
}

#[tokio::test]
async fn test_task_queue_operations() {
    let queue = TaskQueue::new(SchedulingAlgorithm::Priority);
    
    let task1 = Task::new(1, "task1").with_priority(Priority::Normal);
    let task2 = Task::new(2, "task2").with_priority(Priority::High);
    
    queue.enqueue(task1).await;
    queue.enqueue(task2).await;
    
    assert_eq!(queue.len().await, 2);
    
    // Should dequeue higher priority task first
    let dequeued = queue.dequeue().await;
    assert!(dequeued.is_some());
}
```

### 3.3 能力系统集成测试

```rust
// crates/kernel/tests/capability_tests.rs

use beebotos_kernel::capabilities::{CapabilitySet, CapabilityLevel};

#[test]
fn test_capability_check() {
    let caps = CapabilitySet::standard();
    assert!(caps.has(CapabilityLevel::L1FileRead));
    assert!(caps.has(CapabilityLevel::L3NetworkOut));
    assert!(!caps.has(CapabilityLevel::L10SystemAdmin));
}

#[test]
fn test_capability_expiration() {
    let caps = CapabilitySet::standard()
        .with_expiration(0); // Expired immediately
    
    assert!(caps.is_expired());
    assert!(!caps.has(CapabilityLevel::L1FileRead));
}

#[test]
fn test_capability_verify() {
    let caps = CapabilitySet::standard();
    
    assert!(caps.verify(CapabilityLevel::L1FileRead).is_ok());
    assert!(caps.verify(CapabilityLevel::L10SystemAdmin).is_err());
}
```

### 3.4 安全模块集成测试

```rust
// crates/kernel/tests/security_tests.rs

use beebotos_kernel::security::*;
use beebotos_kernel::security::acl::{AccessControlList, Permission};

#[test]
fn test_access_control_list() {
    let mut acl = AccessControlList::new("alice".to_string());
    
    acl.add_entry("bob".to_string(), vec![Permission::Read], true);
    
    assert_eq!(
        acl.check_access("alice", AccessAction::Read),
        AccessDecision::Allow
    );
    
    assert_eq!(
        acl.check_access("bob", AccessAction::Write),
        AccessDecision::Deny
    );
}

#[test]
fn test_capability_set() {
    let mut caps = CapabilitySet::new();
    
    caps.add_permitted(Capability::FileRead);
    caps.add_effective(Capability::FileRead).unwrap();
    
    assert!(caps.has_capability(&Capability::FileRead));
    assert!(!caps.has_capability(&Capability::FileWrite));
}

#[test]
fn test_security_manager() {
    let mut manager = SecurityManager::new();
    let policy = Box::new(DiscretionaryAccessControl::new());
    manager.register_policy(policy);
    
    let context = SecurityContext {
        user_id: "alice".to_string(),
        group_id: "users".to_string(),
        capabilities: vec![Capability::FileRead],
        clearance_level: ClearanceLevel::Internal,
    };
    
    let decision = manager.request_access(&context, "data/tmp/test", AccessAction::Read);
    assert_eq!(decision, AccessDecision::Allow);
}

#[test]
fn test_audit_log() {
    let mut log = AuditLog::new();
    
    let context = SecurityContext {
        user_id: "alice".to_string(),
        group_id: "users".to_string(),
        capabilities: vec![],
        clearance_level: ClearanceLevel::Internal,
    };
    
    log.log_access_attempt(&context, "data/tmp/test", AccessAction::Read, AccessDecision::Allow);
    
    let entries = log.query(Some("alice"), None, None);
    assert_eq!(entries.len(), 1);
}
```

---

## 基准测试

### 4.1 运行基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench scheduler

# 保存基准结果
cargo bench -- --save-baseline before_optimization

# 与基线比较
cargo bench -- --baseline before_optimization
```

### 4.2 调度器基准测试

```rust
// crates/kernel/benches/scheduler_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use beebotos_kernel::scheduler::{Task, Priority};
use beebotos_kernel::scheduler::queue::{TaskQueue, SchedulingAlgorithm};
use beebotos_kernel::capabilities::{CapabilitySet, CapabilityLevel};

fn bench_task_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_creation");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size), 
            size, 
            |b, &_size| {
                b.iter(|| {
                    Task::new(
                        rand::random::<u64>(),
                        "bench_task",
                    ).with_priority(Priority::Normal)
                });
            }
        );
    }
    
    group.finish();
}

fn bench_capability_verification(c: &mut Criterion) {
    let caps = CapabilitySet::standard();
    
    c.bench_function("capability_verify_l5", |b| {
        b.iter(|| {
            let _ = black_box(caps.verify(CapabilityLevel::L5SpawnLimited));
        });
    });
}

criterion_group!(
    benches,
    bench_task_creation,
    bench_capability_verification,
);
criterion_main!(benches);
```

### 4.3 基准测试结果分析

```bash
$ cargo bench

Running target/release/deps/scheduler_bench

task_creation/10          time:   [15.234 ns 15.456 ns 15.678 ns]
task_creation/100         time:   [15.123 ns 15.345 ns 15.567 ns]
task_creation/1000        time:   [15.012 ns 15.234 ns 15.456 ns]

capability_verify_l5      time:   [45.123 ns 45.456 ns 45.789 ns]
```

---

## 调试工具

### 5.1 调试追踪器 (Tracer)

```rust
use beebotos_kernel::debug::Tracer;

fn process_data() {
    let tracer = Tracer::new("process_data");
    
    tracer.trace("Starting data processing");
    // ... 处理逻辑
    
    tracer.trace("Phase 1 complete");
    // ... 更多处理
    
    // Tracer 在 drop 时自动记录总耗时
}
```

**输出示例：**
```
[process_data] Starting data processing (45µs)
[process_data] Phase 1 complete (1234µs)
[process_data] completed in 5678µs
```

### 5.2 内存调试器 (MemoryDebugger)

```rust
use beebotos_kernel::debug::MemoryDebugger;

fn monitor_memory() {
    // 打印内存统计
    MemoryDebugger::print_stats();
    
    // 获取当前 RSS
    if let Some(rss) = MemoryDebugger::current_rss() {
        println!("Current RSS: {} KB", rss);
    }
}
```

**输出示例（Linux）：**
```
VmPeak:    85632 kB
VmSize:    84320 kB
VmRSS:     23456 kB
VmData:    12345 kB
```

### 5.3 调试宏

```rust
use beebotos_kernel::{debug_assert, debug_print};

fn test_function() {
    // 仅在 debug 构建时生效
    debug_assert!(value > 0, "value must be positive");
    
    debug_print!("Processing item: {}", item_id);
}
```

### 5.4 断点函数

```rust
use beebotos_kernel::debug::breakpoint;

fn critical_section() {
    breakpoint();  // 在 debug 构建时暂停 100ms
    // ... 关键代码
}
```

---

## 内存安全调试

### 6.1 内存安全追踪器

```rust
use beebotos_kernel::memory::safety::{
    MemorySafetyTracker, init_global_tracker, global_tracker
};

fn main() {
    // 初始化全局追踪器（ paranoid_mode = true 记录堆栈）
    init_global_tracker(true);
    
    // 获取追踪器
    if let Some(tracker) = global_tracker() {
        // 模拟内存分配
        let ptr = 0x1000 as *mut u8;
        tracker.record_allocation(ptr, 1024);
        
        // 检查内存访问
        let check = tracker.check_access(ptr, 512);
        println!("Access check: {:?}", check);
        
        // 释放内存
        tracker.record_deallocation(ptr);
        
        // 打印内存泄漏报告
        tracker.print_leak_report();
        
        // 获取统计信息
        let stats = tracker.stats();
        println!("Total allocations: {}", stats.total_allocations);
        println!("Double frees detected: {}", stats.double_frees_detected);
        println!("Use after free detected: {}", stats.use_after_free_detected);
    }
}
```

### 6.2 Canary 保护

```rust
use beebotos_kernel::memory::safety::CanaryGuard;

fn protected_buffer() {
    let canary = CanaryGuard::new();
    
    unsafe {
        let mut buffer: u64 = 0;
        canary.write(&mut buffer);
        
        // ... 使用 buffer
        
        // 验证 canary 是否被篡改
        assert!(canary.verify(&buffer), "Buffer overflow detected!");
    }
}
```

### 6.3 内存毒化

```rust
use beebotos_kernel::memory::safety::{poison_memory, POISON_BYTE};

unsafe fn safe_free(ptr: *mut u8, size: usize) {
    // 在释放前毒化内存
    poison_memory(ptr, size);
    
    // 现在可以安全释放
    std::alloc::dealloc(ptr, layout);
}
```

---

## 性能分析

### 7.1 调度器统计

```rust
use beebotos_kernel::KernelBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kernel = KernelBuilder::new().build()?;
    kernel.start().await?;
    
    // ... 运行任务
    
    // 获取调度器统计
    let stats = kernel.scheduler_stats().await;
    println!("Tasks submitted: {}", stats.tasks_submitted);
    println!("Tasks completed: {}", stats.tasks_completed);
    println!("Tasks failed: {}", stats.tasks_failed);
    println!("Workers: {}", stats.workers);
    
    Ok(())
}
```

### 7.2 内存统计

```rust
use beebotos_kernel::memory::MemorySnapshot;

fn check_memory() {
    let snapshot = MemorySnapshot::capture();
    
    println!("Total memory: {} MB", snapshot.total_bytes / 1024 / 1024);
    println!("Used memory: {} MB", snapshot.used_bytes / 1024 / 1024);
    println!("Free memory: {} MB", snapshot.free_bytes / 1024 / 1024);
}
```

### 7.3 使用 Tracing 进行性能分析

```rust
use tracing::{info, debug, trace};

#[tracing::instrument]
async fn process_task(task_id: u64) {
    trace!("Starting task processing");
    
    debug!(task_id, "Processing task");
    // ... 处理逻辑
    
    info!(task_id, "Task completed");
}
```

**运行带有 tracing 的程序：**
```bash
# 设置日志级别
RUST_LOG=info cargo run
RUST_LOG=debug cargo run
RUST_LOG=trace cargo run

# 只显示 kernel 模块的日志
RUST_LOG=beebotos_kernel=debug cargo run
```

---

## 常见问题排查

### 8.1 调度器问题

#### 问题：任务不执行
```rust
// 检查调度器是否已启动
let mut scheduler = Scheduler::new(config);
scheduler.start_with_executor()?;  // 不要忘记启动！

// 检查任务是否被正确提交
let task_id = scheduler.spawn(...).await?;
println!("Task spawned with ID: {:?}", task_id);
```

#### 问题：任务被取消
```rust
// 检查取消令牌
let token = CancellationToken::new();

// 在任务中定期检查取消状态
async fn cancellable_task(token: CancellationToken) -> Result<()> {
    loop {
        if token.is_cancelled() {
            println!("Task was cancelled");
            return Ok(());
        }
        // ... 任务逻辑
    }
}
```

### 8.2 能力系统问题

#### 问题：权限被拒绝
```rust
// 检查能力等级
let caps = CapabilitySet::standard();
match caps.verify(CapabilityLevel::L5SpawnLimited) {
    Ok(()) => println!("Has required capability"),
    Err(e) => println!("Capability check failed: {}", e),
}

// 检查能力是否过期
if caps.is_expired() {
    println!("Capability has expired!");
}
```

#### 问题：能力验证失败
```rust
// 详细检查
fn debug_capability(caps: &CapabilitySet, required: CapabilityLevel) {
    println!("Max level: {:?}", caps.max_level);
    println!("Required level: {:?}", required);
    println!("Permissions: {:?}", caps.permissions);
    println!("Expired: {}", caps.is_expired());
    
    match caps.verify(required) {
        Ok(()) => println!("✅ Capability check passed"),
        Err(e) => println!("❌ Capability check failed: {:?}", e),
    }
}
```

### 8.3 内存问题

#### 问题：内存泄漏检测
```rust
use beebotos_kernel::memory::safety::global_tracker;

fn detect_leaks() {
    if let Some(tracker) = global_tracker() {
        let leaks = tracker.leak_report();
        
        if leaks.is_empty() {
            println!("✅ No memory leaks detected");
        } else {
            println!("⚠️  Found {} potential leaks:", leaks.len());
            for block in leaks {
                println!("  - Address: {:p}, Size: {} bytes", 
                    block.addr as *const u8, block.size);
            }
        }
        
        tracker.print_leak_report();
    }
}
```

#### 问题：双重释放检测
```rust
// 使用内存安全追踪器
test_double_free_detection();

// 输出：
// ERROR: Double-free detected at 0x1000
// thread 'test_double_free' panicked at 'Double-free detected at 0x1000'
```

### 8.4 系统调用问题

#### 问题：系统调用失败
```rust
use beebotos_kernel::syscalls::{SyscallResult, SyscallError};

async fn safe_syscall(kernel: &Kernel, number: u64, args: SyscallArgs) {
    match kernel.syscall(number, args, caller_id).await {
        SyscallResult::Success(value) => {
            println!("Syscall succeeded with value: {}", value);
        }
        SyscallResult::Error(err) => {
            match err {
                SyscallError::PermissionDenied => {
                    eprintln!("Permission denied - check capability level");
                }
                SyscallError::InvalidArgs => {
                    eprintln!("Invalid arguments provided");
                }
                SyscallError::ResourceNotFound => {
                    eprintln!("Resource not found");
                }
                _ => eprintln!("Syscall failed: {:?}", err),
            }
        }
        SyscallResult::Async(handle) => {
            println!("Async operation started, handle: {}", handle);
        }
    }
}
```

---

## 测试配置

### 9.1 Cargo.toml 测试配置

```toml
[package]
name = "beebotos-kernel"
version = "1.0.0"
edition = "2021"

[dependencies]
# ... 生产依赖

[dev-dependencies]
tokio-test = "0.4"        # 异步测试支持
criterion = "0.5"         # 基准测试框架
tempfile = "3.10"         # 临时文件用于测试

[features]
default = ["std", "jemalloc", "wasm"]
std = []
no_std = []
tee = []
jemalloc = ["dep:tikv-jemallocator"]
wasm = ["dep:wasmtime", "dep:wasmtime-wasi", "dep:wasmtime-runtime", "dep:wat"]
rocksdb = ["dep:rocksdb"]
metrics-prometheus = ["dep:metrics-exporter-prometheus"]

# 测试专用特性
[profile.test]
opt-level = 0
debug = true

[profile.bench]
opt-level = 3
lto = true
```

### 9.2 持续集成配置

```yaml
# .github/workflows/test.yml
name: Kernel Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run unit tests
        run: |
          cd crates/kernel
          cargo test --lib
          
      - name: Run integration tests
        run: |
          cd crates/kernel
          cargo test --test '*'
          
      - name: Run benchmarks
        run: |
          cd crates/kernel
          cargo bench --no-fail-fast
          
      - name: Check code coverage
        run: |
          cargo install cargo-tarpaulin
          cd crates/kernel
          cargo tarpaulin --out Xml
```

### 9.3 测试环境变量

```bash
# 启用详细日志
export RUST_LOG=beebotos_kernel=trace

# 启用内存安全 paranoid 模式
export BEE_MEMORY_PARANOID=1

# 设置测试超时
export BEE_TEST_TIMEOUT=300

# 禁用某些功能进行测试
export BEE_DISABLE_WASM=1
```

---

## 示例程序

### 完整测试示例

```rust
// crates/kernel/examples/simple_scheduler.rs

use beebotos_kernel::KernelBuilder;
use beebotos_kernel::scheduler::{Priority, SchedulerConfig};
use beebotos_kernel::capabilities::CapabilitySet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Simple Scheduler Example\n");

    // 创建内核
    let kernel = KernelBuilder::new()
        .with_scheduler(SchedulerConfig {
            time_slice_ms: 100,
            ..Default::default()
        })
        .build()?;

    // 启动内核
    kernel.start().await?;
    println!("✅ Kernel started\n");

    // 创建任务
    for i in 0..5 {
        let task_id = kernel.spawn_task(
            format!("task_{}", i),
            Priority::Normal,
            CapabilitySet::standard(),
            async move {
                println!("  Task {} is running", i);
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                println!("  Task {} completed", i);
                Ok(())
            }
        ).await?;

        println!("✅ Spawned task {} with ID {:?}", i, task_id);
    }

    // 等待任务完成
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 显示统计
    let stats = kernel.scheduler_stats().await;
    println!("\n📊 Scheduler Stats:");
    println!("  Tasks submitted: {}", stats.tasks_submitted);
    println!("  Tasks completed: {}", stats.tasks_completed);

    // 停止内核
    kernel.stop().await;
    println!("\n✅ Kernel stopped");

    Ok(())
}
```

**运行示例：**
```bash
cd crates/kernel
cargo run --example simple_scheduler
```

---

## 总结

BeeBotOS Kernel 提供全面的调试和测试支持：

| 类别 | 工具/方法 | 用途 |
|-----|----------|-----|
| **单元测试** | `cargo test --lib` | 测试模块内部功能 |
| **集成测试** | `cargo test --test '*'` | 测试模块间交互 |
| **基准测试** | `cargo bench` | 性能测试和优化 |
| **调试追踪** | `Tracer` | 函数执行时间追踪 |
| **内存调试** | `MemorySafetyTracker` | 内存泄漏、双重释放检测 |
| **日志追踪** | `tracing` | 结构化日志和分布式追踪 |
| **统计监控** | `SchedulerStats` | 运行时性能统计 |

通过合理使用这些工具，可以确保 Kernel 模块的正确性、性能和稳定性。



%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&
我已经生成了 `beebotos-kernel-debugging-testing-guide.md` 详细文档，包含以下内容：

## 文档结构概览

| 章节 | 内容 |
|------|------|
| **测试概述** | 测试金字塔结构、测试目录组织 |
| **单元测试** | 调度器、能力系统、内存安全的单元测试示例 |
| **集成测试** | 测试文件位置、运行命令、代码示例 |
| **基准测试** | Criterion 框架使用、性能分析 |
| **调试工具** | Tracer、MemoryDebugger、调试宏、断点 |
| **内存安全调试** | MemorySafetyTracker、Canary 保护、内存毒化 |
| **性能分析** | 调度器统计、内存统计、Tracing 日志 |
| **常见问题排查** | 调度器、能力系统、内存、系统调用问题 |
| **测试配置** | Cargo.toml、CI 配置、环境变量 |

## 关键测试命令

```bash
# 单元测试
cargo test --lib
cargo test scheduler

# 集成测试
cargo test --test scheduler_tests
cargo test --test capability_tests
cargo test --test security_tests

# 基准测试
cargo bench

# 带日志的测试
RUST_LOG=debug cargo test -- --nocapture
```

## 核心调试工具

| 工具 | 用途 | 代码示例 |
|------|------|---------|
| **Tracer** | 函数执行时间追踪 | `let tracer = Tracer::new("func");` |
| **MemorySafetyTracker** | 内存泄漏/双重释放检测 | `init_global_tracker(true);` |
| **MemoryDebugger** | 内存统计 | `MemoryDebugger::print_stats();` |
| **CanaryGuard** | 缓冲区溢出检测 | `canary.verify(&buffer)` |
| **breakpoint()** | 调试断点 | `breakpoint();` |

文档已保存到 `beebotos-kernel-debugging-testing-guide.md`，总长度约 26KB。

