# BeeBotOS Agents 模块调试和测试详细指导文档

## 目录

1. [测试环境搭建](#1-测试环境搭建)
2. [快速开始](#2-快速开始)
3. [单元测试指南](#3-单元测试指南)
4. [集成测试指南](#4-集成测试指南)
5. [模块专项测试](#5-模块专项测试)
6. [调试技巧](#6-调试技巧)
7. [性能测试](#7-性能测试)
8. [Mock 和 Stub](#8-mock-和-stub)
9. [CI/CD 集成](#9-cicd-集成)
10. [常见问题排查](#10-常见问题排查)

---

## 1. 测试环境搭建

### 1.1 基础环境要求

```bash
# Rust 工具链 (1.75+)
rustup update stable

# 安装必要组件
rustup component add rustfmt clippy llvm-tools-preview

# 安装 cargo 工具
cargo install cargo-tarpaulin cargo-audit cargo-outdated

# 安装测试工具
cargo install cargo-nextest --locked
```

### 1.2 项目配置

```toml
# crates/agents/Cargo.toml (开发依赖)
[dev-dependencies]
tokio-test = { version = "0.4", features = ["full"] }
mockall = "0.12"
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3.10"
wiremock = "0.6"
```

### 1.3 环境变量配置

```bash
# .env 文件
RUST_LOG=beebotos_agents=debug,info
RUST_BACKTRACE=1
TEST_TIMEOUT=30000
MOCK_MCP_SERVER=true

# 会话存储路径
BEE_SESSION_PATH=data/sessions
BEE_TRANSCRIPT_PATH=data/transcripts
```

---

## 2. 快速开始

### 2.1 运行所有测试

```bash
# 基础测试
cargo test -p beebotos-agents

# 带日志输出
cargo test -p beebotos-agents -- --nocapture

# 使用 nextest (推荐)
cargo nextest run -p beebotos-agents

# 并行测试
cargo test -p beebotos-agents --jobs 4
```

### 2.2 运行特定测试

```bash
# 运行特定模块测试
cargo test -p beebotos-agents a2a
cargo test -p beebotos-agents session
cargo test -p beebotos-agents queue

# 运行特定测试函数
cargo test -p beebotos-agents test_a2a_client_creation

# 忽略某些测试
cargo test -p beebotos-agents -- --skip integration
```

### 2.3 代码覆盖率

```bash
# 生成覆盖率报告
cargo tarpaulin -p beebotos-agents --out Html --out Lcov

# 查看覆盖率
open tarpaulin-report.html
```

---

## 3. 单元测试指南

### 3.1 测试结构

```
crates/agents/
├── src/
│   ├── lib.rs          # 模块级单元测试
│   ├── a2a/mod.rs      # A2A 模块内联测试
│   └── ...
└── tests/              # 集成测试目录
    ├── a2a_test.rs
    ├── agent_test.rs
    └── protocol_tests.rs
```

### 3.2 内联单元测试示例

```rust
// src/session/key.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_creation() {
        let key = SessionKey::new("agent123", SessionType::Session);
        assert_eq!(key.agent_id, "agent123");
        assert_eq!(key.session_type, SessionType::Session);
        assert_eq!(key.depth, 0);
    }

    #[test]
    fn test_session_key_parse_v1() {
        // 旧格式兼容: agent:<id>:<type>:<uuid>
        let key_str = "agent:abc123:session:550e8400-e29b-41d4-a716-446655440000";
        let key = SessionKey::parse(key_str).unwrap();
        
        assert_eq!(key.agent_id, "abc123");
        assert_eq!(key.session_type, SessionType::Session);
        assert_eq!(key.depth, 0); // V1 默认 depth 为 0
    }

    #[test]
    fn test_session_key_parse_v1() {
        // 新格式: agent:<id>:<type>:<depth>:<uuid>
        let key_str = "agent:abc123:subagent:2:6ba7b810-9dad-11d1-80b4-00c04fd430c8";
        let key = SessionKey::parse(key_str).unwrap();
        
        assert_eq!(key.agent_id, "abc123");
        assert_eq!(key.session_type, SessionType::Subagent);
        assert_eq!(key.depth, 2);
    }

    #[test]
    fn test_spawn_child_depth_limit() {
        let parent = SessionKey::new("agent123", SessionType::Session);
        
        // 递归创建子会话
        let child1 = parent.spawn_child().unwrap();
        assert_eq!(child1.depth, 1);
        
        let child2 = child1.spawn_child().unwrap();
        assert_eq!(child2.depth, 2);
        
        // 达到最大深度
        let mut current = child2;
        for _ in 2..SessionKey::MAX_DEPTH {
            current = current.spawn_child().unwrap();
        }
        
        // 超过最大深度应该失败
        assert!(current.spawn_child().is_err());
    }

    #[test]
    fn test_session_key_display() {
        let key = SessionKey::new("agent123", SessionType::Session);
        let display = key.to_string();
        
        // 验证新格式包含 depth
        assert!(display.starts_with("agent:agent123:session:0:"));
    }
}
```

### 3.3 异步测试

```rust
// src/queue/manager.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_submit_task() {
        let manager = QueueManager::new();
        
        let task = QueueTask {
            id: TaskId::new(),
            session_key: SessionKey::new("test", SessionType::Session),
            task_type: TaskType::Main,
            priority: Priority::Normal,
        };
        
        let result = manager.submit_main(task).await;
        assert!(result.is_ok());
        
        // 验证队列长度
        let stats = manager.stats().await;
        assert_eq!(stats.main_queue_pending, 1);
    }

    #[tokio::test]
    async fn test_queue_priority_ordering() {
        let manager = QueueManager::new();
        
        // 提交不同优先级的任务
        let low_priority = create_task(Priority::Low);
        let high_priority = create_task(Priority::High);
        let critical_priority = create_task(Priority::Critical);
        
        manager.submit_main(low_priority).await.unwrap();
        manager.submit_main(high_priority).await.unwrap();
        manager.submit_main(critical_priority).await.unwrap();
        
        // 验证按优先级取出
        let next = manager.get_next_task().await.unwrap();
        assert_eq!(next.priority, Priority::Critical);
        
        let next = manager.get_next_task().await.unwrap();
        assert_eq!(next.priority, Priority::High);
        
        let next = manager.get_next_task().await.unwrap();
        assert_eq!(next.priority, Priority::Low);
    }

    #[tokio::test]
    async fn test_concurrent_queue_access() {
        use tokio::task::JoinSet;
        
        let manager = Arc::new(QueueManager::new());
        let mut set = JoinSet::new();
        
        // 并发提交任务
        for i in 0..100 {
            let mgr = manager.clone();
            set.spawn(async move {
                let task = create_task_with_id(i);
                mgr.submit_main(task).await
            });
        }
        
        // 等待所有提交完成
        while let Some(result) = set.join_next().await {
            assert!(result.unwrap().is_ok());
        }
        
        // 验证所有任务都在队列中
        let stats = manager.stats().await;
        assert_eq!(stats.main_queue_pending, 100);
    }
}
```

### 3.4 使用 Mock 测试

```rust
// src/skills/executor.rs

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    // 创建 Mock 类型
    mock! {
        SkillLoader {
            async fn load_skill(&self, skill_id: &str) -> Result<LoadedSkill, SkillLoadError>;
            fn get_skill(&self, skill_id: &str) -> Option<&LoadedSkill>;
        }
    }

    #[tokio::test]
    async fn test_skill_execution_success() {
        let mut mock_loader = MockSkillLoader::new();
        
        // 配置 mock 行为
        mock_loader
            .expect_load_skill()
            .with(mockall::predicate::eq("test_skill"))
            .times(1)
            .returning(|_| Ok(create_test_skill()));
        
        let executor = SkillExecutor::new();
        let skill = mock_loader.load_skill("test_skill").await.unwrap();
        
        let context = SkillContext {
            input: "test input".to_string(),
            parameters: HashMap::new(),
        };
        
        let result = executor.execute(&skill, context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_skill_execution_with_params() {
        let executor = SkillExecutor::new();
        let skill = create_test_skill_with_params();
        
        let context = SkillContext {
            input: "process".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("timeout".to_string(), "30".to_string());
                params.insert("retries".to_string(), "3".to_string());
                params
            },
        };
        
        let result = executor.execute(&skill, context).await.unwrap();
        assert!(result.success);
        
        // 验证资源使用被记录
        assert!(result.resources_used.cpu_time_ms > 0);
        assert!(result.resources_used.memory_peak_mb > 0);
    }
}
```

---

## 4. 集成测试指南

### 4.1 A2A 协议集成测试

```rust
// tests/a2a_integration.rs

use beebotos_agents::a2a::{A2AClient, A2AMessage, MessageType, MessagePriority};
use beebotos_agents::types::AgentId;

#[tokio::test]
async fn test_a2a_end_to_end_communication() {
    // 创建两个 A2A 客户端
    let client_a = A2AClient::new().expect("Failed to create client A");
    let client_b = A2AClient::new().expect("Failed to create client B");
    
    // 注册 Agent B 到发现服务
    let agent_b_card = create_test_agent_card("agent_b");
    client_b.discovery().register_agent(agent_b_card);
    
    // Agent A 发送消息给 Agent B
    let message = A2AMessage::new(
        MessageType::Request,
        AgentId::from_string("agent_a"),
        Some(AgentId::from_string("agent_b")),
        create_test_payload(),
    ).with_priority(MessagePriority::High);
    
    let result = client_a.send_message(message, "agent_b").await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.msg_type, MessageType::Request);
}

#[tokio::test]
async fn test_a2a_discovery_flow() {
    let client = A2AClient::new().expect("Failed to create client");
    let discovery = client.discovery();
    
    // 注册多个具有不同能力的 agent
    let compute_agent = create_agent_with_capabilities("compute_1", vec!["compute", "storage"]);
    let storage_agent = create_agent_with_capabilities("storage_1", vec!["storage"]);
    
    discovery.register_agent(compute_agent);
    discovery.register_agent(storage_agent);
    
    // 按能力发现 agent
    let compute_agents = discovery.find_agents_by_capability("compute");
    assert_eq!(compute_agents.len(), 1);
    assert_eq!(compute_agents[0].id, "compute_1");
    
    let storage_agents = discovery.find_agents_by_capability("storage");
    assert_eq!(storage_agents.len(), 2);
}

#[tokio::test]
async fn test_a2a_message_signing() {
    use chrono::Utc;
    
    let client = A2AClient::new().expect("Failed to create client");
    
    let message = A2AMessage::new(
        MessageType::Request,
        AgentId::from_string("sender"),
        Some(AgentId::from_string("receiver")),
        create_test_payload(),
    );
    
    // 签名消息
    let signed = message.sign(vec![1, 2, 3, 4]);
    assert!(signed.signature.is_some());
    
    // 验证签名格式 (现在签名是 Option<Vec<u8>>)
    let sig = signed.signature.unwrap();
    assert!(!sig.is_empty());
}
```

### 4.2 会话管理集成测试

```rust
// tests/session_integration.rs

use beebotos_agents::session::{SessionKey, SessionType, SessionContext, IsolationLevel};

#[tokio::test]
async fn test_session_lifecycle() {
    // 创建会话
    let session_key = SessionKey::new("agent_001", SessionType::Session);
    let context = SessionContext::new(session_key.clone());
    
    // 验证会话属性
    assert_eq!(context.session_key.agent_id, "agent_001");
    assert!(context.validate().is_ok());
    
    // 创建子会话
    let child_key = session_key.spawn_child().expect("Failed to spawn child");
    assert_eq!(child_key.depth, 1);
    assert_eq!(child_key.session_type, SessionType::Subagent);
    
    // 验证路径生成
    let path = session_key.to_path();
    assert!(path.to_string().contains("agent_001"));
    assert!(path.to_string().contains("session"));
}

#[tokio::test]
async fn test_session_isolation() {
    let session_key = SessionKey::new("agent_001", SessionType::Session);
    
    // 创建隔离配置
    let isolation = IsolationConfig {
        level: IsolationLevel::Process,
        namespace: "test-ns".to_string(),
        resource_limits: ResourceLimits {
            cpu_quota: 100000,
            memory_limit: 1024 * 1024 * 100, // 100MB
        },
    };
    
    let context = SessionContext::with_isolation(session_key, isolation);
    
    // 验证隔离配置
    assert_eq!(context.isolation.level, IsolationLevel::Process);
    assert_eq!(context.isolation.resource_limits.memory_limit, 104857600);
}

#[tokio::test]
async fn test_session_persistence() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let persistence = SessionPersistence::new(temp_dir.path());
    
    let session_key = SessionKey::new("agent_001", SessionType::Session);
    let context = SessionContext::new(session_key.clone());
    
    // 保存会话
    persistence.save(&session_key, &context).await.expect("Failed to save session");
    
    // 加载会话
    let loaded = persistence.load(&session_key).await.expect("Failed to load session");
    assert_eq!(loaded.session_key, session_key);
}
```

### 4.3 任务队列集成测试

```rust
// tests/queue_integration.rs

use beebotos_agents::queue::{QueueManager, QueueTask, TaskType, Priority};

#[tokio::test]
async fn test_multi_queue_system() {
    let manager = QueueManager::new();
    
    // 提交到主队列
    let main_task = create_queue_task(TaskType::Main, Priority::Normal);
    manager.submit_main(main_task).await.unwrap();
    
    // 提交到 Cron 队列
    let cron_task = create_queue_task(TaskType::Cron, Priority::Low);
    manager.submit_cron(cron_task, "0 */5 * * * *").await.unwrap();
    
    // 提交到子代理队列 (并行执行)
    for i in 0..5 {
        let subagent_task = create_queue_task(TaskType::Subagent, Priority::High);
        manager.submit_subagent(subagent_task).await.unwrap();
    }
    
    // 验证各队列状态
    let stats = manager.stats().await;
    assert_eq!(stats.main_queue_pending, 1);
    assert_eq!(stats.cron_queue_pending, 1);
    assert_eq!(stats.subagent_queue_pending, 5);
}

#[tokio::test]
async fn test_queue_concurrency_limits() {
    let manager = QueueManager::new();
    
    // 子代理队列最大并发数为 5
    let mut handles = vec![];
    
    for i in 0..10 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let task = create_queue_task(TaskType::Subagent, Priority::Normal);
            mgr.submit_subagent(task).await
        });
        handles.push(handle);
    }
    
    // 所有提交都应该成功 (队列会缓冲)
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
    
    // 但并发执行数应该受限
    let permits = manager.subagent_permits_available().await;
    assert!(permits <= 5);
}
```

---

## 5. 模块专项测试

### 5.1 A2A 模块测试矩阵

| 测试项 | 测试类型 | 命令 | 说明 |
|--------|----------|------|------|
| 消息序列化 | 单元测试 | `cargo test a2a::message` | 验证 A2AMessage 序列化 |
| 发现服务 | 单元测试 | `cargo test a2a::discovery` | 测试 Agent 发现逻辑 |
| 安全签名 | 单元测试 | `cargo test a2a::security` | 验证 Ed25519 签名 |
| 传输层 | 集成测试 | `cargo test a2a_transport` | HTTP/WebSocket 传输 |
| 协议协商 | 集成测试 | `cargo test a2a::negotiation` | 能力协商流程 |

### 5.2 会话模块测试矩阵

```bash
# Session Key 测试
cargo test session::key --features test-utils

# Session Context 测试
cargo test session::context

# 隔离级别测试
cargo test session::isolation -- --ignored

# 持久化测试 (需要文件系统)
cargo test session::persistence -- --ignored
```

### 5.3 技能系统测试

```rust
// tests/skills_test.rs

#[tokio::test]
async fn test_skill_loading() {
    let loader = SkillLoader::new();
    
    // 添加技能搜索路径
    loader.add_path("./test_skills");
    
    // 加载技能
    let skill = loader.load_skill("test_skill").await;
    assert!(skill.is_ok());
    
    let loaded = skill.unwrap();
    assert_eq!(loaded.name, "Test Skill");
    assert_eq!(loaded.version.to_string(), "1.0.0");
}

#[tokio::test]
async fn test_skill_version_parsing() {
    use beebotos_agents::skills::registry::Version;
    
    let v1 = Version::parse("1.2.3").unwrap();
    assert_eq!(v1.major, 1);
    assert_eq!(v1.minor, 2);
    assert_eq!(v1.patch, 3);
    
    // 版本比较
    let v2 = Version::parse("1.2.4").unwrap();
    assert!(v1 < v2);
    
    // 无效版本
    assert!(Version::parse("1.2").is_err());
    assert!(Version::parse("1.2.3.4").is_err());
}
```

---

## 6. 调试技巧

### 6.1 日志调试

```rust
// 在代码中添加结构化日志
use tracing::{info, debug, warn, error, span, Level};

#[tokio::test]
async fn test_with_logging() {
    // 初始化测试日志
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
    
    let span = span!(Level::INFO, "a2a_test", agent_id = "test_agent");
    let _enter = span.enter();
    
    debug!("Creating A2A client...");
    let client = A2AClient::new().expect("Failed to create client");
    
    info!("Client created successfully");
    
    // 使用 instrument 宏自动跟踪异步函数
    let result = send_test_message(&client).await;
    debug!(result = ?result, "Message sent");
}

#[tracing::instrument(skip(client))]
async fn send_test_message(client: &A2AClient) -> Result<A2AMessage, A2AError> {
    // 自动记录函数入口和出口
    client.send_message(create_test_message(), "target").await
}
```

### 6.2 使用 GDB/LLDB 调试

```bash
# 编译带调试信息的测试
cargo test -p beebotos-agents --no-run

# 使用 rust-gdb
cargo gdb --bin test_a2a_client_creation

# 常用 GDB 命令
(gdb) break beebotos_agents::a2a::A2AClient::new
(gdb) run
(gdb) print client
(gdb) step
(gdb) continue
(gdb) bt  # 查看调用栈
```

### 6.3 Tokio Console 调试

```toml
# Cargo.toml
[dev-dependencies]
console-subscriber = "0.2"
```

```rust
// 在测试主函数中启用
#[tokio::main]
async fn main() {
    console_subscriber::init();
    
    // 运行测试...
}
```

```bash
# 启动 tokio-console
cargo install tokio-console
tokio-console
```

### 6.4 内存调试

```rust
// 检测内存泄漏
#[test]
fn test_memory_leak() {
    use std::sync::Arc;
    use std::mem::drop;
    
    let initial_memory = get_memory_usage();
    
    for _ in 0..1000 {
        let client = Arc::new(A2AClient::new().unwrap());
        let discovery = client.discovery();
        // 使用 discovery...
        drop(client);
        drop(discovery);
    }
    
    // 强制垃圾回收
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    let final_memory = get_memory_usage();
    assert!(final_memory - initial_memory < 1024 * 1024); // < 1MB 增长
}

fn get_memory_usage() -> usize {
    // 使用 sysinfo 或其他方式获取内存使用
    0
}
```

---

## 7. 性能测试

### 7.1 Criterion Benchmark

```rust
// benches/agent_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use beebotos_agents::a2a::{A2AClient, A2AMessage, MessageType};

fn bench_a2a_message_creation(c: &mut Criterion) {
    c.bench_function("a2a_message_new", |b| {
        b.iter(|| {
            let msg = A2AMessage::new(
                MessageType::Request,
                AgentId::from_string("sender"),
                Some(AgentId::from_string("receiver")),
                create_test_payload(),
            );
            black_box(msg);
        });
    });
}

fn bench_session_key_parse(c: &mut Criterion) {
    let key_str = "agent:abc123:session:0:550e8400-e29b-41d4-a716-446655440000";
    
    c.bench_function("session_key_parse", |b| {
        b.iter(|| {
            let key = SessionKey::parse(black_box(key_str));
            black_box(key);
        });
    });
}

fn bench_queue_throughput(c: &mut Criterion) {
    use tokio::runtime::Runtime;
    
    let rt = Runtime::new().unwrap();
    
    c.bench_function("queue_submit_1000", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = QueueManager::new();
            for i in 0..1000 {
                let task = create_queue_task(i);
                manager.submit_main(task).await.unwrap();
            }
        });
    });
}

criterion_group!(benches, bench_a2a_message_creation, bench_session_key_parse, bench_queue_throughput);
criterion_main!(benches);
```

### 7.2 压力测试

```rust
// tests/stress_test.rs

#[tokio::test]
async fn test_concurrent_a2a_messaging() {
    use std::sync::Arc;
    
    let client = Arc::new(A2AClient::new().unwrap());
    let mut handles = vec![];
    
    let start = std::time::Instant::now();
    
    // 并发发送 10,000 条消息
    for i in 0..10000 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let msg = create_message(i);
            client.send_message(msg, "target").await
        });
        handles.push(handle);
    }
    
    let mut success = 0;
    let mut failed = 0;
    
    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => success += 1,
            Err(_) => failed += 1,
        }
    }
    
    let duration = start.elapsed();
    
    println!("Success: {}, Failed: {}", success, failed);
    println!("Throughput: {} msg/s", 10000.0 / duration.as_secs_f64());
    
    assert!(success > 9000); // 至少 90% 成功率
}
```

---

## 8. Mock 和 Stub

### 8.1 HTTP Mock (使用 wiremock)

```rust
// tests/mcp_mock_test.rs

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_mcp_client_with_mock() {
    // 启动 mock 服务器
    let mock_server = MockServer::start().await;
    
    // 配置 mock 响应
    Mock::given(method("POST"))
        .and(path("/mcp/v1/tools/call"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "result": "success",
                "output": "Mock tool output"
            })))
        .mount(&mock_server)
        .await;
    
    // 创建 MCP 客户端指向 mock
    let mut mcp_manager = MCPManager::new();
    mcp_manager.add_server("mock", MCPClient::new(&mock_server.uri()));
    
    // 测试调用
    let result = mcp_manager
        .call_tool("mock", "test_tool", serde_json::json!({}))
        .await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Mock tool output");
}
```

### 8.2 时间 Mock

```rust
// 使用 tokio::time::pause 控制时间
#[tokio::test]
async fn test_cron_with_mock_time() {
    tokio::time::pause();
    
    let scheduler = CronScheduler::new();
    let mut triggered = false;
    
    scheduler.add_job("0 * * * * *", || async {
        triggered = true;
    }).await;
    
    // 快进 1 分钟
    tokio::time::advance(std::time::Duration::from_secs(60)).await;
    tokio::time::resume();
    
    assert!(triggered);
}
```

---

## 9. CI/CD 集成

### 9.1 GitHub Actions 配置

```yaml
# .github/workflows/agents-test.yml
name: Agents Tests

on:
  push:
    paths:
      - 'crates/agents/**'
  pull_request:
    paths:
      - 'crates/agents/**'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: crates/agents
      
      - name: Run clippy
        run: cargo clippy -p beebotos-agents -- -D warnings
      
      - name: Run fmt check
        run: cargo fmt -p beebotos-agents -- --check
      
      - name: Run unit tests
        run: cargo test -p beebotos-agents --lib
      
      - name: Run integration tests
        run: cargo test -p beebotos-agents --test '*'
        env:
          RUST_LOG: debug
      
      - name: Generate coverage
        run: cargo tarpaulin -p beebotos-agents --out Lcov
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./lcov.info
```

### 9.2 测试报告生成

```bash
# HTML 报告
cargo test -p beebotos-agents -- --format junit > test-results.xml

# 覆盖率报告
cargo tarpaulin -p beebotos-agents --out Html --output-dir coverage

# 性能基准对比
cargo bench -p beebotos-agents -- --baseline main
```

---

## 10. 常见问题排查

### 10.1 编译错误

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `link.exe not found` | 缺少 MSVC | 安装 Visual Studio Build Tools |
| `wasmtime` 编译失败 | 缺少依赖 | `cargo build --features wasm-runtime` |
| `openssl` 错误 | 系统库缺失 | 安装 `libssl-dev` (Linux) |

### 10.2 运行时错误

```rust
// 错误: A2AClient::new() 返回 Err
// 原因: 安全模块初始化失败

// 调试步骤:
let client = match A2AClient::new() {
    Ok(c) => c,
    Err(e) => {
        eprintln!("A2A client creation failed: {:?}", e);
        // 检查密钥生成是否可用
        panic!("Cannot proceed without A2A client");
    }
};
```

### 10.3 测试超时

```bash
# 增加测试超时时间
cargo test -p beebotos-agents -- --timeout 300

# 只运行快速测试
cargo test -p beebotos-agents -- --skip slow
```

### 10.4 资源泄漏检测

```bash
# 使用 valgrind (Linux)
valgrind --leak-check=full --show-leak-kinds=all cargo test -p beebotos-agents

# 使用 AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo test -p beebotos-agents -Z build-std
```

### 10.5 调试检查清单

- [ ] `RUST_LOG` 环境变量设置正确
- [ ] `RUST_BACKTRACE=1` 启用堆栈跟踪
- [ ] 测试用例使用 `#[tokio::test]` 而非 `#[test]` (异步测试)
- [ ] Mock 对象的 `expect()` 配置正确
- [ ] 临时文件使用 `tempfile` crate 管理
- [ ] 并发测试使用 `Arc` 共享状态

---

## 附录: 测试工具速查表

| 命令 | 用途 |
|------|------|
| `cargo test -p beebotos-agents` | 运行所有测试 |
| `cargo test -p beebotos-agents a2a` | 运行 A2A 模块测试 |
| `cargo test -p beebotos-agents -- --nocapture` | 显示 println 输出 |
| `cargo test -p beebotos-agents -- --test-threads=1` | 单线程运行 |
| `cargo nextest run -p beebotos-agents` | 使用 nextest 运行 |
| `cargo tarpaulin -p beebotos-agents` | 生成覆盖率报告 |
| `cargo bench -p beebotos-agents` | 运行性能基准测试 |
| `cargo clippy -p beebotos-agents` | 静态分析检查 |

---

*文档版本: 1.0*
*最后更新: 2026-03-22*

&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&
已生成 `beebotos-agents` 模块的详细调试和测试指导文档，保存至 `beebotos-agents-debugging-testing-guide.md`。

## 文档内容概览

| 章节 | 内容 |
|------|------|
| **测试环境搭建** | Rust 工具链、依赖安装、环境变量配置 |
| **快速开始** | 运行测试、代码覆盖率、常见命令 |
| **单元测试指南** | 内联测试、异步测试、Mock 测试示例 |
| **集成测试指南** | A2A 协议、会话管理、任务队列集成测试 |
| **模块专项测试** | A2A/会话/技能等模块的测试矩阵 |
| **调试技巧** | 日志调试、GDB/LLDB、Tokio Console、内存调试 |
| **性能测试** | Criterion Benchmark、压力测试代码示例 |
| **Mock 和 Stub** | wiremock HTTP Mock、时间 Mock |
| **CI/CD 集成** | GitHub Actions 配置、测试报告生成 |
| **常见问题排查** | 编译/运行时错误、资源泄漏检测检查清单 |

## 核心测试命令速查

```bash
# 基础测试
cargo test -p beebotos-agents
cargo nextest run -p beebotos-agents

# 带日志和覆盖率
RUST_LOG=debug cargo test -p beebotos-agents -- --nocapture
cargo tarpaulin -p beebotos-agents --out Html

# 性能测试
cargo bench -p beebotos-agents

# 调试构建
RUST_BACKTRACE=1 cargo test -p beebotos-agents test_a2a_client_creation
```
