
## beebotos-p2p 编译和使用指南

**beebotos-p2p** 是 BeeBotOS 的 **点对点网络层**，基于 libp2p 实现，提供 Agent 之间的去中心化通信、服务发现、消息传递等功能。

---

### 📦 编译命令

#### 1. 编译整个项目（包含 p2p）
```bash
# 项目根目录
cargo build --release

# 编译后的库
# target/release/libbeebotos_p2p.rlib
```

#### 2. 只编译 P2P crate
```bash
# 编译 beebotos-p2p
cargo build --release -p beebotos-p2p

# 调试模式
cargo build -p beebotos-p2p
```

#### 3. 运行测试
```bash
# 运行单元测试
cargo test -p beebotos-p2p

# 带日志输出
RUST_LOG=debug cargo test -p beebotos-p2p -- --nocapture
```

---

### 🚀 使用方法

#### 作为库依赖

在 `Cargo.toml` 中添加：
```toml
[dependencies]
beebotos-p2p = { path = "crates/p2p" }
```

---

### 💻 编程示例

#### 1. 创建和启动 P2P 节点

```rust
use beebotos_p2p::P2PNode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 P2P 节点
    let node = P2PNode::new()?;
    
    println!("Local Peer ID: {}", node.local_peer_id());
    
    // 启动网络
    node.start().await?;
    println!("P2P network started!");
    
    // 运行一段时间
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    
    // 停止网络
    node.stop().await?;
    println!("P2P network stopped!");
    
    Ok(())
}
```

---

#### 2. 配置 P2P 网络

```rust
use beebotos_p2p::behaviour::P2PConfig;

fn main() {
    // 配置 P2P 网络参数
    let config = P2PConfig {
        listen_addrs: vec![
            "/ip4/0.0.0.0/tcp/4001".to_string(),
            "/ip4/0.0.0.0/udp/4001/quic".to_string(),
        ],
        bootstrap_peers: vec![
            "/dns4/bootstrap.beebotos.io/tcp/4001/p2p/12D3...".to_string(),
        ],
        enable_mdns: true,      // 本地网络发现
        enable_kademlia: true,  // DHT 路由
        capabilities: vec![
            "agent".to_string(),
            "storage".to_string(),
            "compute".to_string(),
        ],
    };
    
    println!("P2P Config: {:?}", config);
}
```

---

#### 3. 服务发现

```rust
use beebotos_p2p::discovery::AgentDiscovery;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let discovery = AgentDiscovery::new();
    
    // 查找具有特定能力的 Agent
    let storage_peers = discovery
        .find_peers_with_capability("storage")
        .await?;
    
    println!("Found {} storage peers", storage_peers.len());
    for peer_id in storage_peers {
        println!("  - {}", peer_id);
    }
    
    // 查找计算节点
    let compute_peers = discovery
        .find_peers_with_capability("compute")
        .await?;
    
    Ok(())
}
```

---

#### 4. 消息处理

```rust
use beebotos_p2p::{P2PMessage, Result};
use beebotos_p2p::messaging::{MessageHandler, DefaultMessageHandler};
use libp2p::PeerId;

// 自定义消息处理器
struct MyMessageHandler;

#[async_trait::async_trait]
impl MessageHandler for MyMessageHandler {
    async fn handle_message(&self, msg: P2PMessage) -> Result<()> {
        let from = msg.from;
        let payload = String::from_utf8_lossy(&msg.payload);
        
        println!("Received message from {}: {}", from, payload);
        
        // 根据消息类型处理
        match payload.as_ref() {
            "ping" => println!("Received ping from {}", from),
            "task_request" => println!("Task request from {}", from),
            _ => println!("Unknown message type"),
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 使用默认处理器
    let default_handler = DefaultMessageHandler;
    
    // 或使用自定义处理器
    let my_handler = MyMessageHandler;
    
    // 创建示例消息
    let msg = P2PMessage {
        from: PeerId::random(),
        to: Some(PeerId::random()),
        payload: b"Hello, P2P!".to_vec(),
    };
    
    // 处理消息
    my_handler.handle_message(msg).await?;
    
    Ok(())
}
```

---

#### 5. 完整 P2P Agent 示例

```rust
use beebotos_p2p::{
    P2PNode, P2PMessage, Result,
    behaviour::{AgentBehaviour, P2PConfig, AgentBehaviourEvent},
    messaging::MessageHandler,
};
use libp2p::PeerId;
use std::sync::Arc;
use tokio::sync::mpsc;

// 自定义行为
struct MyAgent {
    node: P2PNode,
    event_rx: mpsc::Receiver<AgentBehaviourEvent>,
}

impl MyAgent {
    async fn new(config: P2PConfig) -> Result<Self> {
        let node = P2PNode::new()?;
        let (event_tx, event_rx) = mpsc::channel(100);
        
        // 创建 Agent 行为
        let _behaviour = AgentBehaviour::new(&config, node.local_peer_id())?;
        
        Ok(Self {
            node,
            event_rx,
        })
    }
    
    async fn run(&mut self) -> Result<()> {
        self.node.start().await?;
        
        println!("Agent running with ID: {}", self.node.local_peer_id());
        
        // 处理网络事件
        while let Some(event) = self.event_rx.recv().await {
            match event {
                AgentBehaviourEvent::MessageReceived { from, data } => {
                    println!("From {}: {:?}", from, String::from_utf8_lossy(&data));
                }
            }
        }
        
        Ok(())
    }
    
    async fn send_message(&self, to: PeerId, data: Vec<u8>) -> Result<()> {
        let msg = P2PMessage {
            from: self.node.local_peer_id(),
            to: Some(to),
            payload: data,
        };
        
        // 发送消息逻辑
        println!("Sending message to {}", to);
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = P2PConfig {
        listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".to_string()],
        bootstrap_peers: vec![],
        enable_mdns: true,
        enable_kademlia: true,
        capabilities: vec!["chat".to_string()],
    };
    
    let mut agent = MyAgent::new(config).await?;
    agent.run().await?;
    
    Ok(())
}
```

---

### 📋 核心功能模块

| 模块 | 文件 | 功能 |
|------|------|------|
| **P2PNode** | `lib.rs` | P2P 网络节点管理 |
| **AgentBehaviour** | `behaviour.rs` | Agent 网络行为定义 |
| **AgentDiscovery** | `discovery.rs` | 服务发现和 peer 查找 |
| **MessageHandler** | `messaging.rs` | 消息处理 trait |
| **Transport** | `transport.rs` | 网络传输层 |

---

### 🔧 libp2p 协议栈

| 协议 | 用途 |
|------|------|
| **TCP** | 基础传输 |
| **QUIC** | 快速 UDP 连接 |
| **WebSocket** | 浏览器兼容 |
| **Noise** | 加密握手 |
| **Yamux** | 多路复用 |
| **GossipSub** | 发布订阅消息 |
| **mDNS** | 本地网络发现 |
| **Kademlia** | DHT 路由 |
| **Identify** | 节点识别 |
| **Ping** | 连通性检测 |

---

### 📁 项目结构

```
crates/p2p/
├── Cargo.toml
└── src/
    ├── lib.rs          # 库入口 - P2PNode
    ├── behaviour.rs    # Agent 网络行为
    ├── discovery.rs    # 服务发现
    ├── messaging.rs    # 消息处理
    ├── message.rs      # 消息类型定义
    └── transport.rs    # 传输层
```

---

### 🛠 技术栈

| 组件 | 用途 |
|------|------|
| **libp2p** | P2P 网络协议栈 |
| **tokio** | 异步运行时 |
| **serde** | 序列化 |
| **futures** | 异步编程 |

---

### ⚠️ 注意事项

1. **当前为 Stub 实现** - 基础结构已定义，核心功能待实现
2. **需要 bootstrap 节点** - 生产环境需要配置引导节点
3. **防火墙配置** - 确保 TCP/UDP 端口开放
4. **NAT 穿透** - 复杂网络环境可能需要中继节点

需要我帮你实现完整的 P2P 网络功能吗？


