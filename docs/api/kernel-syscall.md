# BeeBotOS 内核系统调用

> **64 个系统调用接口详细说明**

---

## 概述

BeeBotOS 内核提供 64 个系统调用，Agent 通过 WASM 宿主函数接口调用这些功能。

---

## 系统调用列表

### Agent 管理 (0-9)

| 编号 | 名称 | 描述 | 权限 |
|------|------|------|------|
| 0 | `spawn_agent` | 创建子 Agent | L5 |
| 1 | `terminate_agent` | 终止 Agent | L5 |
| 2 | `suspend_agent` | 暂停 Agent | L5 |
| 3 | `resume_agent` | 恢复 Agent | L5 |
| 4 | `query_agent_status` | 查询状态 | L0 |
| 5 | `list_child_agents` | 列出子 Agent | L0 |
| 6 | `update_agent_config` | 更新配置 | L6 |
| 7 | `transfer_capability` | 转移权限 | L9 |
| 8 | `delegate_capability` | 委托权限 | L5 |
| 9 | `revoke_capability` | 撤销权限 | L5 |

### 通信 (10-19)

| 编号 | 名称 | 描述 | 权限 |
|------|------|------|------|
| 10 | `send_message` | 发送消息 | L3 |
| 11 | `receive_message` | 接收消息 | L3 |
| 12 | `broadcast` | 广播消息 | L4 |
| 13 | `discover_services` | 发现服务 | L3 |
| 14 | `register_service` | 注册服务 | L4 |
| 15 | `unregister_service` | 注销服务 | L4 |
| 16 | `negotiate` | 协商 | L3 |
| 17 | `accept_proposal` | 接受提议 | L3 |
| 18 | `reject_proposal` | 拒绝提议 | L3 |
| 19 | `query_conversation` | 查询会话 | L0 |

### 资源访问 (20-29)

| 编号 | 名称 | 描述 | 权限 |
|------|------|------|------|
| 20 | `access_resource` | 访问资源 | L3 |
| 21 | `allocate_memory` | 分配内存 | L0 |
| 22 | `free_memory` | 释放内存 | L0 |
| 23 | `query_memory` | 查询内存 | L0 |
| 24 | `get_resource_limit` | 获取限制 | L0 |
| 25 | `set_resource_limit` | 设置限制 | L6 |
| 26 | `request_cpu_time` | 请求 CPU | L0 |
| 27 | `yield_cpu` | 让出 CPU | L0 |
| 28 | `query_cpu_usage` | 查询 CPU 使用 | L0 |
| 29 | `reserve_resource` | 预留资源 | L5 |

### 文件操作 (30-39)

| 编号 | 名称 | 描述 | 权限 |
|------|------|------|------|
| 30 | `open_file` | 打开文件 | L1 |
| 31 | `read_file` | 读取文件 | L1 |
| 32 | `write_file` | 写入文件 | L2 |
| 33 | `close_file` | 关闭文件 | L0 |
| 34 | `delete_file` | 删除文件 | L2 |
| 35 | `list_directory` | 列出目录 | L1 |
| 36 | `create_directory` | 创建目录 | L2 |
| 37 | `remove_directory` | 删除目录 | L2 |
| 38 | `get_file_info` | 获取文件信息 | L1 |
| 39 | `seek_file` | 文件定位 | L1 |

### 区块链 (40-49)

| 编号 | 名称 | 描述 | 权限 |
|------|------|------|------|
| 40 | `query_balance` | 查询余额 | L7 |
| 41 | `query_contract` | 查询合约 | L7 |
| 42 | `read_event` | 读取事件 | L7 |
| 43 | `execute_payment` | 执行支付 | L8 |
| 44 | `call_contract` | 调用合约 | L8 |
| 45 | `deploy_contract` | 部署合约 | L9 |
| 46 | `sign_message` | 签名消息 | L7 |
| 47 | `verify_signature` | 验证签名 | L0 |
| 48 | `bridge_token` | 跨链转账 | L8 |
| 49 | `stake_token` | 质押代币 | L8 |

### 记忆与知识 (50-59)

| 编号 | 名称 | 描述 | 权限 |
|------|------|------|------|
| 50 | `query_memory` | 查询记忆 | L0 |
| 51 | `store_memory` | 存储记忆 | L0 |
| 52 | `update_memory` | 更新记忆 | L0 |
| 53 | `delete_memory` | 删除记忆 | L0 |
| 54 | `search_knowledge` | 搜索知识 | L0 |
| 55 | `add_knowledge` | 添加知识 | L1 |
| 56 | `query_knowledge_graph` | 查询知识图谱 | L0 |
| 57 | `update_emotion` | 更新情绪 | L0 |
| 58 | `query_emotion` | 查询情绪 | L0 |
| 59 | `learn_from_interaction` | 交互学习 | L0 |

### 系统 (60-63)

| 编号 | 名称 | 描述 | 权限 |
|------|------|------|------|
| 60 | `get_time` | 获取时间 | L0 |
| 61 | `get_random` | 获取随机数 | L0 |
| 62 | `log_event` | 记录日志 | L0 |
| 63 | `emit_metric` | 发送指标 | L0 |

---

## 系统调用接口

### Rust 接口

```rust
#[repr(u64)]
pub enum SyscallNumber {
    SpawnAgent = 0,
    TerminateAgent = 1,
    SendMessage = 10,
    ReceiveMessage = 11,
    QueryBalance = 40,
    ExecutePayment = 43,
    QueryMemory = 50,
    StoreMemory = 51,
    // ...
}

pub struct SyscallArgs {
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
    pub arg4: u64,
    pub arg5: u64,
}

pub enum SyscallResult {
    Success(u64),
    Error(SyscallError),
}

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

### WASM 宿主函数

```rust
// 在 WASM 中调用
#[link(wasm_import_module = "beebotos")]
extern "C" {
    fn syscall(number: u64, args: *const u64) -> i64;
    
    // 快捷函数
    fn host_send_message(recipient: i32, message: i32) -> i32;
    fn host_query_memory(key: i32, value: i32, max_len: i32) -> i32;
    fn host_store_memory(key: i32, value: i32) -> i32;
    fn host_log(level: i32, message: i32);
}
```

---

## 使用示例

### 发送消息

```rust
fn send_message_example() {
    let args = SyscallArgs {
        arg0: recipient_id.as_u64(),
        arg1: message_ptr as u64,
        arg2: message_len as u64,
        ..Default::default()
    };
    
    match syscall(SyscallNumber::SendMessage as u64, &args) {
        SyscallResult::Success(msg_id) => {
            println!("Message sent: {}", msg_id);
        }
        SyscallResult::Error(e) => {
            eprintln!("Failed to send: {:?}", e);
        }
    }
}
```

### 查询余额

```rust
fn query_balance_example() {
    let args = SyscallArgs {
        arg0: chain_id as u64,
        arg1: token_address_ptr as u64,
        arg2: owner_address_ptr as u64,
        arg3: result_ptr as u64,
        ..Default::default()
    };
    
    let result = syscall(SyscallNumber::QueryBalance as u64, &args);
    
    match result {
        SyscallResult::Success(_) => {
            let balance = unsafe { *(result_ptr as *const U256) };
            println!("Balance: {}", balance);
        }
        SyscallResult::Error(SyscallError::PermissionDenied) => {
            eprintln!("No L7 permission");
        }
        _ => {}
    }
}
```

### 存储记忆

```rust
fn store_memory_example() {
    let key = "user_preference";
    let value = r#"{"theme": "dark", "language": "zh"}"#;
    
    let args = SyscallArgs {
        arg0: key.as_ptr() as u64,
        arg1: key.len() as u64,
        arg2: value.as_ptr() as u64,
        arg3: value.len() as u64,
        arg4: MemoryType::LTM as u64,
        arg5: 0, // importance
    };
    
    syscall(SyscallNumber::StoreMemory as u64, &args);
}
```

---

## 性能考虑

| 调用类型 | 延迟 | 说明 |
|---------|------|------|
| L0-L2 | < 1μs | 本地操作 |
| L3-L6 | ~10μs | 网络/进程 |
| L7-L9 | ~100ms | 区块链操作 |

---

**文档版本**: v1.0.0
