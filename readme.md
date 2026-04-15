# BeeBotOS v1.0 🐝

> **Web4.0 自主智能体操作系统** —— 首个专为 AI Agent 设计的去中心化操作系统

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Solidity](https://img.shields.io/badge/solidity-%5E0.8.0-blue.svg)](https://soliditylang.org)

---

## 🌟 项目简介

BeeBotOS 是一个**5层架构的 Web4.0 自主智能体操作系统**，它将操作系统的资源管理、进程调度、安全隔离等核心概念引入 AI Agent 世界，解决了传统 Agent 平台的三大痛点：

| 痛点 | BeeBotOS 解决方案 |
|------|-------------------|
| ❌ Agent 之间缺乏安全隔离 | ✅ WASM 沙箱 + Capability 权限模型 |
| ❌ 无法精确控制资源使用 | ✅ 内核级调度器 + Gas 计量系统 |
| ❌ 单点故障影响全局 | ✅ 分布式 P2P 网络 + 区块链锚定 |

### 核心数据

- 📦 **431+** Rust 源文件
- 🔗 **14** 个核心 Crates
- 📜 **49** 个 Solidity 智能合约
- 🧠 **3** 种神经网络进化算法
- 💱 **4** 条链支持 (ETH/BSC/Polygon/Solana)

---

## 🚀 快速开始

### 环境要求

- **Rust** >= 1.75.0
- **Node.js** >= 18.0 (可选，用于前端)
- **Foundry** (用于 Solidity 合约开发)
- **Docker** >= 24.0 (可选，用于容器化部署)

### 安装

```bash
# 克隆仓库
git clone https://github.com/your-org/beebotos.git
cd beebotos

# 安装依赖
cargo fetch

# 编译项目
cargo build --release

# 运行测试
cargo test --all
```

### 运行示例

```bash
# 启动本地开发网络
make localnet-up

# 运行示例 Agent
cargo run --example simple_agent

# 运行 A2A 通信示例
cargo run --example a2a_chat
```

---

## 🏗️ 系统架构

BeeBotOS 采用**5层模块化架构**，每一层都可独立扩展和升级：

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 4: Applications                                           │
│  DeFAI · Social AI · DAO Governance · Game AI                   │
├─────────────────────────────────────────────────────────────────┤
│ Layer 3: Agent Layer                                            │
│  A2A Protocol · MCP · Browser Automation · Workflow Engine      │
├─────────────────────────────────────────────────────────────────┤
│ Layer 2: Social Brain                                           │
│  NEAT · PAD · OCEAN · Memory System · Reasoning Engine          │
├─────────────────────────────────────────────────────────────────┤
│ Layer 1: Kernel                                                 │
│  Scheduler · Security · WASM Runtime · Syscalls · IPC           │
├─────────────────────────────────────────────────────────────────┤
│ Layer 0: Blockchain                                             │
│  Ethereum · BSC · Polygon · Solana · Cross-Chain Bridge         │
└─────────────────────────────────────────────────────────────────┘
```

### 架构亮点

#### 🔒 Layer 1: 系统内核
- **CFS + MLFQ 混合调度器**: 公平且高效的 CPU 时间分配
- **10层 Capability 模型**: 细粒度权限控制，支持权限委托
- **WASM 沙箱**: 安全的 Agent 执行环境，支持 Gas 计量
- **64个系统调用**: 标准化的内核接口

#### 🧠 Layer 2: 社交大脑
- **NEAT**: 神经网络进化算法，自动优化网络结构
- **PAD**: 情绪动态模型（愉悦-唤醒-支配）
- **OCEAN**: 大五人格模型（开放-尽责-外向-亲和-神经质）
- **记忆系统**: STM(短期)/LTM(长期)/EM(情景)三层记忆

#### 🤝 Layer 3: Agent 层
- **A2A 协议**: Agent-to-Agent 标准化通信
- **MCP**: Model Context Protocol 工具调用
- **Browser Automation**: CDP 协议的浏览器自动化

---

## 📁 项目结构

```
beebotos/
├── 📦 crates/                    # Rust 核心库 (14 crates)
│   ├── core/                     # 核心类型和工具
│   ├── kernel/                   # 系统内核 (调度/安全/WASM)
│   ├── brain/                    # 神经网络和认知模型
│   ├── social-brain/             # 人格和情绪模型
│   ├── agents/                   # Agent 运行时和 A2A
│   ├── chain/                    # 区块链集成
│   ├── dao/                      # DAO 治理
│   ├── p2p/                      # 点对点网络
│   ├── storage/                  # 存储系统
│   ├── crypto/                   # 加密工具
│   ├── wasm/                     # WASM 工具
│   ├── gateway/                  # API 网关
│   ├── metrics/                  # 监控指标
│   └── telemetry/                # 遥测和日志
│
├── 📜 contracts/                 # Solidity 智能合约
│   └── solidity/
│       ├── dao/                  # DAO 核心合约
│       ├── tokens/               # 代币合约
│       ├── bridge/               # 跨链桥
│       └── skills/               # Skill 注册表
│
├── 🎮 apps/                      # 示例应用
│   ├── defai/                    # DeFAI 应用
│   ├── social/                   # 社交 Agent
│   └── game/                     # 游戏 AI
│
├── 📖 docs/                      # 文档
├── 🐳 docker/                    # Docker 配置
├── ☸️ helm/                      # Kubernetes 配置
├── 🔧 config/                    # 配置文件
└── 🧪 tests/                     # 集成测试
```

---

## 💡 核心特性

### 1. 自主智能体运行时
```rust
use beebotos::agent::{Agent, AgentConfig};

let config = AgentConfig::new()
    .with_personality(Personality::analytical())
    .with_memory_limit(512 * MB)
    .with_capability(CapabilityLevel::L3_NetworkOut);

let agent = Agent::spawn(config).await?;
agent.run().await?;
```

### 2. A2A 协议通信
```rust
// Agent A 向 Agent B 发送消息
let message = A2AMessage::new()
    .to(agent_b_id)
    .intent(Intent::Negotiate)
    .payload(negotiation_data);

agent_a.send(message).await?;

// Agent B 接收并响应
let response = agent_b.receive().await?;
agent_b.reply(response.id(), accept_deal).await?;
```

### 3. 区块链集成
```rust
use beebotos::chain::{Chain, Wallet};

// 多链钱包管理
let wallet = Wallet::create()
    .add_chain(Chain::Ethereum)
    .add_chain(Chain::Solana)
    .build()?;

// 执行跨链支付
wallet.bridge(Chain::Ethereum, Chain::Solana, amount).await?;
```

### 4. WASM 技能系统
```rust
// 加载 WASM 技能模块
let skill = Skill::from_wasm("trading_strategy.wasm")
    .with_gas_limit(1_000_000)
    .with_capability(CapabilityLevel::L8_ChainWrite);

// 在沙箱中执行
let result = agent.execute(skill, market_data).await?;
```

---

## 🛠️ 技术栈

### 后端 (Rust)
| 组件 | 技术选型 | 用途 |
|------|---------|------|
| 异步运行时 | Tokio | 高并发任务调度 |
| Web 框架 | Axum | HTTP API 服务 |
| WASM 引擎 | Wasmtime | Agent 沙箱执行 |
| P2P 网络 | libp2p | 去中心化通信 |
| 区块链 | ethers-rs, solana-client | 链上交互 |
| 数据库 | RocksDB, PostgreSQL | 数据持久化 |

### 智能合约 (Solidity)
| 组件 | 技术选型 |
|------|---------|
| 开发框架 | Foundry |
| 合约标准 | OpenZeppelin |
| 跨链协议 | LayerZero / Axelar |

### 前端 (可选)
| 组件 | 技术选型 |
|------|---------|
| 框架 | React / Vue |
| Web3 | wagmi, viem |
| UI | Tailwind CSS |

---

## 📚 文档导航

### 入门指南
- [📖 完整项目规范](BeeBotOS-Complete-Project-Spec-v2.0.md) - 详细需求与结构设计
- [🏗️ 架构设计文档](BeeBotOS-Design-v2-with-DAO.md) - 5层架构详细说明
- [💻 技术规范](BeeBotOS-V1-Technical-Specification.md) - API、配置、部署指南
- [⚙️ 内核层详解](BeeBotOS-Kernel-Layer-Detailed-Summary.md) - Layer 1 设计原理

### 开发文档
- [📝 贡献指南](CONTRIBUTING.md) - 如何参与项目
- [🤝 行为准则](CODE_OF_CONDUCT.md) - 社区规范
- [🔒 安全政策](SECURITY.md) - 安全报告流程
- [📋 目录结构](BeeBotOS-Directory-Structure-v2.0.md) - 项目组织说明

### 参考文档
- [📊 API 文档](https://docs.beebotos.io/api) - 自动生成 API 参考
- [🔧 配置示例](config/) - 各种环境的配置文件
- [🧪 测试示例](tests/) - 集成测试用例
- [🎮 应用示例](apps/) - 示例应用程序

---

## 🧪 测试

### 运行测试

```bash
# 单元测试
cargo test --lib

# 集成测试
cargo test --test integration

# 端到端测试
cargo test --test e2e

# 性能基准测试
cargo bench

# Solidity 合约测试
cd contracts/solidity && forge test
```

### 代码质量

```bash
# 格式化代码
cargo fmt --all

# 静态分析
cargo clippy --all-targets --all-features -- -D warnings

# 安全检查
cargo audit
```

---

## 🚀 部署

### Docker 部署

```bash
# 构建镜像
docker build -t beebotos:v1.0.0 .

# 运行容器
docker-compose up -d
```

### Kubernetes 部署

```bash
# 安装 Helm Chart
helm install beebotos ./helm/beebotos \
  --namespace beebotos \
  --create-namespace \
  --set replicaCount=3
```

详细部署文档请参考 [DEPLOYMENT.md](docs/deployment/DEPLOYMENT.md)

---

## 🤝 如何贡献

我们欢迎各种形式的贡献！

1. **Fork** 本仓库
2. **创建分支** (`git checkout -b feature/amazing-feature`)
3. **提交更改** (`git commit -m 'Add amazing feature'`)
4. **推送分支** (`git push origin feature/amazing-feature`)
5. **创建 Pull Request**

请参考 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详细的贡献流程和代码规范。

### 贡献者

感谢所有为 BeeBotOS 做出贡献的开发者！

<a href="https://github.com/your-org/beebotos/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=your-org/beebotos" />
</a>

---

## 📄 许可证

本项目采用 **MIT 许可证** - 详见 [LICENSE](LICENSE) 文件

```
MIT License

Copyright (c) 2026 BeeBotOS Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
```

---

## 🌐 社区

- 💬 [Discord](https://discord.gg/beebotos)
- 🐦 [Twitter/X](https://twitter.com/beebotos)
- 📝 [Blog](https://blog.beebotos.io)
- 📧 [Email](mailto:contact@beebotos.io)

---

## 🗺️ 路线图

### 2026 Q1 - v1.0.0
- ✅ 核心内核架构
- ✅ A2A 协议 v1.0
- ✅ 多链钱包支持
- ✅ WASM 技能系统

### 2026 Q2 - v1.1.0
- 🔄 TEE 硬件安全 (Intel SGX/AMD SEV)
- 🔄 跨链桥主网上线
- 🔄 DAO 治理 V1

### 2026 Q3 - v1.2.0
- 📅 移动客户端 (iOS/Android)
- 📅 浏览器插件
- 📅 AI 驱动的代码生成

---

## 🙏 致谢

- [Rust 社区](https://www.rust-lang.org/community) - 提供优秀的编程语言和生态
- [OpenAI](https://openai.com) - AI 模型的先驱
- [Ethereum Foundation](https://ethereum.foundation) - 区块链基础设施
- [WebAssembly](https://webassembly.org) - 安全的沙箱执行环境

---

<div align="center">

**⭐ 如果这个项目对您有帮助，请给我们一个 Star！**

🐝 **构建智能的未来，一起进入 Web4.0 时代！**

</div>
