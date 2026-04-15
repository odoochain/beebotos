# BeeBotOS Gateway 模块业务逻辑关系分析

## 一、模块概述

### 1. beebotos-gateway-lib (crates/gateway)
**定位**: 网关核心库 - 提供可复用的网关基础设施

**核心职责**:
- 速率限制 (Token Bucket, Sliding Window, Fixed Window)
- JWT 认证与授权中间件
- WebSocket 连接管理
- 服务发现与负载均衡
- 熔断器模式
- 健康检查

**特点**:
- 纯库形式，无业务逻辑
- 可独立使用
- 提供 Builder 模式构建

---

### 2. beebotos-gateway (apps/gateway)
**定位**: API Gateway 应用程序 - 面向外部的统一入口

**核心职责**:
- HTTP/HTTPS 服务器
- 数据库持久化 (PostgreSQL)
- Agent 生命周期管理
- 外部 API 路由
- 身份验证 (JWT + API Key)
- 指标收集与监控

**特点**:
- 可执行程序 (binary)
- 依赖 gateway-lib 和 kernel
- 处理具体业务场景

---

### 3. beebotos-kernel (crates/kernel)
**定位**: 操作系统内核 - Agent 运行时环境

**核心职责**:
- Agent 进程/任务调度
- 基于能力的权限系统 (Capability-based Security)
- WASM 运行时 (WASI 支持)
- 系统调用接口 (29个 syscall)
- 内存隔离与管理
- 持久化存储 (RocksDB)

**特点**:
- 底层基础设施
- 安全沙箱环境
- 资源限制与监控

---

## 二、依赖关系图

```
┌─────────────────────────────────────────────────────────────┐
│                    beebotos-gateway                         │
│                   (API Gateway 应用)                         │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │ HTTP Server │  │   Agent     │  │   Auth      │        │
│  │  (axum)     │  │  Management │  │ (JWT/Key)   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        │               │               │
        ▼               ▼               ▼
┌───────────────┐ ┌──────────┐ ┌──────────────┐
│beebotos-      │ │beebotos- │ │  PostgreSQL  │
│gateway-lib    │ │kernel    │ │   Database   │
│(网关基础设施)   │ │(Agent OS)│ │              │
└───────────────┘ └──────────┘ └──────────────┘
```

---

## 三、业务逻辑交互关系

### 1. 请求流 (Request Flow)

```
Client Request
     │
     ▼
┌─────────────────────────────────────┐
│     beebotos-gateway (HTTP Server)  │
│  ┌───────────────────────────────┐  │
│  │  1. 中间件链 (来自 gateway-lib) │  │
│  │     - request_id_middleware    │  │
│  │     - rate_limit_middleware    │  │
│  │     - auth_middleware (JWT)    │  │
│  └───────────────────────────────┘  │
│  ┌───────────────────────────────┐  │
│  │  2. 路由处理                    │  │
│  │     - /api/v1/agents/*         │  │
│  │     - /api/v1/skills/*         │  │
│  └───────────────────────────────┘  │
└─────────────────────┬───────────────┘
                      │
                      ▼
         ┌─────────────────────┐
         │  3. 业务逻辑判断      │
         │  - 需要内核介入?      │
         └─────────────────────┘
               │         │
               │ No      │ Yes
               ▼         ▼
      ┌──────────┐  ┌──────────┐
      │ 直接处理  │  │ 调用     │
      │ (DB操作) │  │ kernel   │
      └──────────┘  └────┬─────┘
                         │
                         ▼
              ┌─────────────────────┐
              │  beebotos-kernel    │
              │  ┌───────────────┐  │
              │  │ - 创建 Agent  │  │
              │  │ - 调度任务    │  │
              │  │ - 执行 WASM   │  │
              │  │ - 资源限制    │  │
              │  └───────────────┘  │
              └─────────────────────┘
```

### 2. Agent 生命周期管理

```
beebotos-gateway                    beebotos-kernel
     │                                    │
     │  1. POST /api/v1/agents            │
     │ ────────────────────────────────>  │
     │  (创建 Agent 请求)                  │
     │                                    │
     │  2. 验证权限、配额                   │
     │     写入数据库 (agents 表)          │
     │                                    │
     │  3. 调用 kernel API                │
     │ ────────────────────────────────>  │
     │     create_agent_capability()      │
     │                                    │
     │  4. 创建沙箱环境                     │
     │     分配 capability tokens         │
     │     初始化 WASM runtime            │
     │                                    │
     │  5. 返回 Agent ID                  │
     │ <────────────────────────────────  │
     │                                    │
     │  6. POST /api/v1/agents/:id/start  │
     │ ────────────────────────────────>  │
     │                                    │
     │  7. 调度执行 (scheduler)            │
     │     spawn_wasm_task()              │
     │                                    │
     │  8. 返回运行状态                     │
     │ <────────────────────────────────  │
     │                                    │
```

### 3. 安全模型交互

```
┌─────────────────────────────────────────────────────────────┐
│                      Security Flow                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Layer 1: beebotos-gateway (API Security)                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ - JWT/API Key 认证                                   │   │
│  │ - Rate Limiting (gateway-lib)                        │   │
│  │ - CORS/HTTPS                                         │   │
│  │ - Request Validation                                 │   │
│  └─────────────────────┬───────────────────────────────┘   │
│                        │                                     │
│                        ▼                                     │
│  Layer 2: beebotos-gateway-lib (Infrastructure)             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ - Token validation                                   │   │
│  │ - Circuit breaker                                    │   │
│  │ - Service discovery                                  │   │
│  └─────────────────────┬───────────────────────────────┘   │
│                        │                                     │
│                        ▼                                     │
│  Layer 3: beebotos-kernel (OS-level Security)               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ - Capability-based access control                    │   │
│  │ - Process isolation                                  │   │
│  │ - Resource limits (cgroup)                           │   │
│  │ - Syscall filtering                                  │   │
│  │ - Memory sandbox                                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、数据流关系

### 1. 配置流

```
Environment / Config File
           │
           ▼
┌──────────────────────┐
│ beebotos-gateway     │
│ (AppConfig)          │
│ - server.port        │
│ - jwt.secret         │
│ - database.url       │
└──────────┬───────────┘
           │
           ▼ (部分传递)
┌──────────────────────┐
│ beebotos-gateway-lib │
│ (GatewayConfig)      │
│ - rate_limit         │
│ - cors               │
│ - websocket          │
└──────────┬───────────┘
           │
           ▼ (独立配置)
┌──────────────────────┐
│ beebotos-kernel      │
│ (KernelConfig)       │
│ - max_agents         │
│ - wasm_limits        │
│ - storage_path       │
└──────────────────────┘
```

### 2. 状态同步

| 模块 | 管理的状态 | 同步方式 |
|------|-----------|---------|
| beebotos-gateway | Agent 元数据 (DB) | PostgreSQL 持久化 |
| beebotos-gateway | Agent 运行时状态 (内存) | RwLock<HashMap> |
| beebotos-gateway-lib | 限流计数器 | DashMap (内存) |
| beebotos-gateway-lib | WebSocket 连接 | Arc<RwLock> |
| beebotos-kernel | Agent 进程 | Kernel scheduler |
| beebotos-kernel | Capability tokens | 内存 + RocksDB |

---

## 五、代码层面的调用关系

### 1. gateway → gateway-lib 调用

```rust
// apps/gateway/src/main.rs
use beebotos_gateway_lib::rate_limit::{RateLimitManager, token_bucket::TokenBucketRateLimiter};
use beebotos_gateway_lib::middleware::auth_middleware;

// 使用 gateway-lib 的限流器
let rate_limiter = Arc::new(RateLimiter::new(100, 200));

// 使用 gateway-lib 的中间件
let app = Router::new()
    .layer(from_fn_with_state(state.clone(), middleware::auth_middleware));
```

### 2. gateway → kernel 调用

```rust
// 创建 Agent 时调用 kernel
use beebotos_kernel::capabilities::{Capability, CapabilityToken};
use beebotos_kernel::wasm::{WasmEngine, WasmInstance};

// 创建 capability
let cap = kernel.create_agent_capability(agent_id, permissions)?;

// 运行 WASM
let instance = kernel.spawn_wasm(code, capabilities, resource_limits)?;
```

### 3. kernel → gateway 反向通知

```rust
// kernel 通过事件通知 gateway
// apps/gateway/src/handlers/websocket/mod.rs

// WebSocket 推送状态更新
ws_manager.broadcast_to_channel("agent_events", json!({
    "agent_id": agent_id,
    "status": "completed",
    "result": result
})).await?;
```

---

## 六、职责边界划分

### ✅ beebotos-gateway-lib 应该做的
- [x] 通用的限流算法实现
- [x] JWT 的验证与生成
- [x] WebSocket 连接管理
- [x] 服务发现接口定义
- [x] 熔断器、负载均衡

### ❌ beebotos-gateway-lib 不应该做的
- [ ] 特定业务逻辑 (Agent 管理)
- [ ] 数据库操作
- [ ] 外部 HTTP 客户端调用
- [ ] 业务相关的错误消息

### ✅ beebotos-gateway 应该做的
- [x] HTTP 路由和 handler
- [x] Agent CRUD 操作
- [x] 数据库持久化
- [x] 集成 gateway-lib 的能力
- [x] 调用 kernel 创建 Agent

### ✅ beebotos-kernel 应该做的
- [x] Agent 沙箱执行
- [x] 资源限制和监控
- [x] 系统调用处理
- [x] 权限验证 (capability)
- [x] WASM 运行时

---

## 七、改进建议

### 1. 当前问题
1. **gateway 和 gateway-lib 功能重叠**
   - 两者都有 middleware
   - 两者都有 rate_limit
   - 两者都有 auth

2. **gateway 和 kernel 边界不清晰**
   - Agent 状态分散在两个模块
   - 缺乏统一的事件通知机制

3. **gateway-lib 未被充分利用**
   - 当前 apps/gateway 使用自己的实现
   - 应该更多复用 gateway-lib

### 2. 重构建议

```
建议的模块关系:

┌─────────────────────────────────────────────┐
│          beebotos-gateway (应用层)          │
│                                             │
│  只做:                                      │
│  - HTTP 路由 handlers                       │
│  - 业务逻辑编排 (调用 lib 和 kernel)         │
│  - 数据库操作                                │
└─────────────────────┬───────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        │             │             │
        ▼             ▼             ▼
┌──────────┐ ┌──────────────┐ ┌──────────┐
│gateway-  │ │  beebotos-   │ │PostgreSQL│
│lib       │ │  kernel      │ │          │
│(基础设施) │ │  (Agent OS)  │ │          │
└──────────┘ └──────────────┘ └──────────┘

重构动作:
1. apps/gateway 的 middleware → 使用 gateway-lib
2. apps/gateway 的 rate_limit → 使用 gateway-lib  
3. apps/gateway 的 websocket → 使用 gateway-lib
4. 统一通过 kernel 管理 Agent 生命周期
```

---

## 八、总结

| 维度 | beebotos-gateway-lib | beebotos-gateway | beebotos-kernel |
|------|---------------------|------------------|-----------------|
| **定位** | 基础设施库 | API 网关应用 | Agent 操作系统 |
| **复用性** | 高 | 低 | 中 |
| **业务耦合** | 无 | 高 | 中 |
| **主要职责** | 限流、认证、WS | 路由、CRUD、DB | 执行、安全、调度 |
| **依赖方向** | 被依赖 | 依赖 lib + kernel | 被依赖 |
| **数据持久化** | 无 | PostgreSQL | RocksDB |
| **部署形态** | 库 | 服务 | 库/服务 |

**核心关系**: 
- `gateway` 是入口，协调 `gateway-lib` 和 `kernel`
- `gateway-lib` 提供通用能力
- `kernel` 提供安全的 Agent 运行环境
