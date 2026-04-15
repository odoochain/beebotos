# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

### Gateway（主服务）
```bash
# 快速检查编译
 cargo check -p beebotos-gateway

# Release 构建
 cargo build -p beebotos-gateway --release

# 从项目根目录启动（配置自动加载）
 ./target/release/beebotos-gateway

# Debug 构建
 cargo build -p beebotos-gateway
```

### Web 前端
```bash
# 构建 WASM（需已安装 wasm-pack）
 wasm-pack build --target web --out-dir pkg

# 启动 Rust 静态文件服务器（开发模式）
 cargo run --bin web-server -- --config config/web-server.toml

# 启动 Release 二进制（必须显式指定静态路径和 Gateway 代理地址）
 ./target/release/web-server --config config/web-server.toml \
   --static-path /absolute/path/to/apps/web/pkg \
   --gateway-url http://localhost:8000
```

### CLI 与 BeeHub
```bash
# CLI 二进制名为 beebot
 cargo run --bin beebot -- --help

# 本地安装 CLI
 cargo install --path apps/cli --force

# BeeHub（Skill 市场）二进制名为 beehub
 cargo run --bin beehub
```

### 测试
```bash
# 全 workspace 测试（较慢，非必要时避免）
 cargo test --workspace --all-features

# 仅运行所有 crate 的单元测试（推荐常规使用）
 cargo test --workspace --lib

# 定向 crate 测试
 cargo test -p beebotos-brain --lib --all-features
 cargo test -p beebotos-gateway --lib
 cargo test -p beebotos-agents --lib

# 运行单个测试
 cargo test -p beebotos-brain --lib test_name --all-features

# 格式化与 lint
 cargo fmt --all
 cargo clippy --workspace --all-targets --all-features -- -D warnings

# 一键完整检查（fmt + clippy + test）
 just check
# 或
 make check
```

### 快捷脚本（Justfile / Makefile）
项目根目录提供了 `justfile` 和 `Makefile`：
```bash
just build      # cargo build --workspace --release
just test       # cargo test --workspace --all-features
just lint       # cargo clippy ...
just fmt        # cargo fmt --all
just dev        # cargo watch -x build -x test（需预先安装 cargo-watch）
```

### 智能合约
```bash
cd contracts && forge build
cd contracts && forge test
```

## High-Level Architecture

### 依赖方向
`core` → `kernel` → `brain` → `agents` → `gateway-lib` / `message-bus` → `gateway` / `web` / `cli`

### 核心 Crate
- **`crates/core`**: 共享类型、错误构造器、事件总线原语。
- **`crates/kernel`**: 抢占式调度器、基于能力的权限安全、WASM 运行时（`wasmtime`）、TEE 抽象。
- **`crates/brain`**: NEAT 神经网络演化、PAD 情绪模型、OCEAN 人格模型、多模态记忆系统。
- **`crates/agents`**: Agent 运行时、LLM 提供商路由（Kimi / OpenAI / Zhipu / DeepSeek / Claude / Ollama）、Channel 注册表（飞书、企业微信、钉钉、Telegram、Discord、Slack、个人微信通过 iLink）、A2A / MCP / Service Mesh、规划引擎（Planning）。
- **`crates/gateway-lib`**: 可复用的 Gateway 基础设施——限流、JWT 中间件、WebSocket 管理、服务发现、熔断器。**Gateway 应用直接消费它。**
- **`crates/message-bus`**: 内部异步消息总线，支持内存、Redis、gRPC 三种传输后端。
- **`crates/chain`**: 链上交互、DAO / 身份 / NFT / 钱包合约相关逻辑。
- **`crates/sdk`**, **`crates/p2p`**, **`crates/telemetry`**: 分别为 SDK 封装、P2P 网络、可观测性埋点，当前处于相对次要地位。

### 顶层应用
- **`apps/gateway`**: 主服务端可执行文件。业务逻辑集中于此：
  - **HTTP handlers**: `apps/gateway/src/handlers/http/`（agents、channels、skills、webhooks、state_machine、task_monitor 等）
  - **Services**: `apps/gateway/src/services/`（llm_service、cache_warmer 等）
  - 数据库持久化、Channel 初始化、AgentService / AgentRuntimeManager 桥接。
  基础设施由 `gateway-lib` 提供，运行时刻由 `agents` 提供。
- **`apps/web`**: Leptos WASM 前端（`csr` 模式）+ Rust HTTP 服务器（`web-server` binary），用于服务静态文件并将 API 请求代理到 Gateway。`apps/web/pkg/` 是 wasm-pack 构建输出目录。
- **`apps/cli`**: 命令行界面，用于管理 Agent 和 Channel。二进制名为 `beebot`。
- **`apps/beehub`**: 轻量级 Skill 市场服务，二进制名为 `beehub`，依赖 `beebotos-core`，使用 `axum` + SQLite。

### 架构守卫（CI 强制）
`crates/agents` **严禁依赖任何 Web 框架**。禁止的依赖/导入包括：`axum`、`actix-web`、`rocket`、`warp`、`tide`、`salvo`。由 `.github/workflows/architecture-guard.yml` 校验，会在 push/PR 时自动扫描 `crates/agents/Cargo.toml` 和 `crates/agents/src/`。

## 关键模式与约定

### 固定端口
- **Gateway**：固定监听 `0.0.0.0:8000`。
- **Web 管理后台**：固定监听 `0.0.0.0:8090`（`web-server` 默认端口，代理 API 请求到 `http://localhost:8000`）。

### 目录约定
- **配置文件统一放在 `config/` 目录下**：所有运行时配置文件（如 `config/beebotos.toml`、`config/web-server.toml`）必须集中在项目根目录的 `config/` 下，不再分散在 `apps/*/config/` 等子目录中。
- **数据文件统一放在 `data/` 目录下**：所有持久化数据（SQLite 数据库、日志、缓存、WASM 缓存、transcripts、workspaces 等）必须集中在项目根目录的 `data/` 下。严禁将数据写入 `config/` 或写死绝对路径（如 `/var/lib/beebotos`、`/tmp/...`）。
- **Gateway 配置**：敏感密钥（API key、JWT secret）和非敏感参数全部收敛到 `config/beebotos.toml` 单一文件中，项目不再使用 `.env` 文件。
- **数据库路径**：SQLite 文件使用相对路径 `data/beebotos.db`，由 `apps/gateway/src/config.rs` 中的 `load()` 归一化为绝对路径，确保从任意工作目录启动都能正确定位。

### 个人微信（iLink 协议）
- 实现位于 `crates/agents/src/communication/channel/personal_wechat_channel.rs`。
- 通过 iLink API（`https://ilinkai.weixin.qq.com`）进行二维码登录。
- `connect()` 是非阻塞的：生成 QR 码后立即返回。
- 用户扫码并在前端状态变为 `confirmed` 后，`apps/gateway/src/handlers/http/channels.rs` 中的 `check_wechat_qr` handler 会调用 downcast 后的 `PersonalWeChatChannel.complete_login(bot_token, base_url, event_bus)`，从而启动长轮询消息监听。

### 测试组织
- Workspace 级集成测试：`tests/integration/`、`tests/e2e/`。
- Crate 级测试：各 crate 的 `src/` 下通过 `#[cfg(test)]`  or `tests/` 目录编写。
- Gateway 基准测试：`apps/gateway/benches/`。

### Clippy 配置
项目使用 `clippy.toml` 设定了自定义阈值：`cognitive-complexity-threshold = 30`、`too-many-lines-threshold = 100` 等。

## 开发经验

### 工具选择：Read vs Bash sed
- **Read 工具的 `limit` 不能为负数**。当需要读取文件末尾附近的行号范围时，不要依赖 Read 的负 `limit`，而是直接使用 `Bash sed -n 'START,ENDp' FILE` 来获取精确行号范围的内容。

### 项目特定的辅助脚本
- 根目录下的 `fix_compilation.sh` 包含针对特定编译问题的批量修复逻辑，遇到大范围编译报错时可参考。
- `setup_kimi_env.sh` 用于设置 Kimi API 环境变量（仅供开发环境使用，生产环境应使用配置文件）。

### Cargo.lock 策略
本项目是二进制项目集合（gateway、cli、beehub、web-server），**`Cargo.lock` 已纳入版本控制**，以保证所有开发者构建结果一致。不要将其加入 `.gitignore`。
