
## beebotos-chain 编译和使用指南

**beebotos-chain** 是 BeeBotOS 的 **区块链集成层**，提供与 Monad 区块链的交互、DAO 治理、钱包管理、跨链桥接等功能。

---

### 📦 编译命令

#### 1. 编译整个项目（包含 chain）
```bash
# 项目根目录
cargo build --release

# 编译后的库
# target/release/libbeebotos_chain.rlib
```

#### 2. 只编译 Chain crate
```bash
# 编译 beebotos-chain
cargo build --release -p beebotos-chain

# 调试模式
cargo build -p beebotos-chain
```

#### 3. 运行测试
```bash
# 运行单元测试
cargo test -p beebotos-chain

# 带日志输出
RUST_LOG=debug cargo test -p beebotos-chain -- --nocapture
```

---

### 🚀 使用方法

#### 作为库依赖

在 `Cargo.toml` 中添加：
```toml
[dependencies]
beebotos-chain = { path = "crates/chain" }
```

---

### 💻 编程示例

#### 1. 连接到 Monad 网络

```rust
use beebotos_chain::monad::{MonadClient, MonadConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 使用预设配置
    let config = MonadConfig::testnet();
    
    // 或使用自定义配置
    let config = MonadConfig {
        rpc_url: "https://rpc.testnet.monad.xyz".to_string(),
        ws_url: Some("wss://ws.testnet.monad.xyz".to_string()),
        chain_id: 10143,
        confirmation_blocks: 1,
        gas_limit: 30_000_000,
    };
    
    // 创建客户端
    let client = MonadClient::new(config).await?;
    
    // 获取当前区块号
    let block_number = client.get_block_number().await?;
    println!("Current block: {}", block_number);
    
    // 查询余额
    let address = "0x1234...".parse()?;
    let balance = client.get_balance(address).await?;
    println!("Balance: {} ETH", balance);
    
    Ok(())
}
```

---

#### 2. 钱包管理

```rust
use beebotos_chain::wallet::{Wallet, HDWallet, KeyStore};

// 方法1: 从私钥创建
let wallet = Wallet::from_key(&[/* 32字节私钥 */])?;
println!("Address: {:?}", wallet.address());

// 方法2: 创建随机钱包
let wallet = Wallet::random();
println!("New address: {:?}", wallet.address());

// 签名消息
let signature = wallet.sign_message(b"Hello World").await?;
println!("Signature: {:?}", signature);

// 方法3: 使用助记词创建 HD 钱包
let mnemonic = HDWallet::generate_mnemonic();
let hd_wallet = HDWallet::from_mnemonic(&mnemonic)?;

// 派生账户
let account = hd_wallet.derive_account(0, Some("Main Account".to_string()))?;
println!("Derived address: {:?}", account.address);

// 密钥存储
let keystore = KeyStore::open("./keystore")?;
let encrypted_key = EncryptedKey { /* ... */ };
keystore.store(&wallet.address(), &encrypted_key)?;

// 加载密钥
if let Some(key) = keystore.load(&wallet.address())? {
    println!("Loaded key for {}", wallet.address());
}
```

---

#### 3. DAO 治理交互

```rust
use beebotos_chain::dao::{DAOClient, DAOInterface, VoteType, ProposalId};
use ethers::types::{Address, U256};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建 DAO 客户端
    let dao = DAOClient::new(
        "https://rpc.testnet.monad.xyz",
        "0xDAOContractAddress".parse()?,
    ).await?;
    
    // 获取成员信息
    let member = dao.get_member("0xMemberAddress".parse()?).await?;
    println!("Reputation: {}", member.reputation);
    println!("Voting Power: {}", member.voting_power);
    
    // 创建提案
    let proposal_id = dao.create_proposal(
        vec!["0xTargetContract".parse()?],
        vec![U256::zero()],
        vec![calldata],
        "Allocate budget for Q2 development",
    ).await?;
    println!("Created proposal: {}", proposal_id);
    
    // 投票
    let tx_hash = dao.cast_vote(proposal_id, VoteType::For).await?;
    println!("Vote submitted: {:?}", tx_hash);
    
    // 查询投票权
    let voting_power = dao.get_voting_power("0xYourAddress".parse()?).await?;
    println!("Your voting power: {}", voting_power);
    
    // 设置委托
    dao.set_delegation("0xDelegateAddress".parse()?).await?;
    
    // 创建预算
    let budget_id = dao.create_budget(
        "0xBeneficiary".parse()?,
        U256::from(1000000000000000000u64), // 1 ETH
        86400 * 30, // 30天
        BudgetType::Monthly,
    ).await?;
    
    Ok(())
}
```

---

#### 4. 跨链桥接

```rust
use beebotos_chain::bridge::{Bridge, ChainId, BridgeTx};

// 创建跨链桥
let bridge = Bridge::new(
    ChainId::Ethereum,
    ChainId::Monad,
    "0xBridgeContract".parse()?,
);

// 创建桥接交易
let tx = BridgeTx {
    from: "0xSender".parse()?,
    to: "0xReceiver".parse()?,
    amount: 1000000000000000000u64.into(), // 1 ETH
    token: None, // Native token
};

// 执行桥接
let tx_hash = bridge.initiate_transfer(tx).await?;
println!("Bridge initiated: {:?}", tx_hash);

// 监控状态
let status = bridge.poll_status(tx_hash).await?;
match status {
    BridgeStatus::Pending => println!("Waiting for confirmation..."),
    BridgeStatus::Confirmed => println!("Bridge completed!"),
    BridgeStatus::Failed(e) => println!("Bridge failed: {}", e),
}
```

---

#### 5. DeFi 交互

```rust
use beebotos_chain::defi::{DEX, SwapParams};

// 连接到 DEX
let dex = DEX::new(
    "0xRouterAddress".parse()?,
    client.provider(),
);

// 交换代币
let swap = SwapParams {
    token_in: "0xTokenA".parse()?,
    token_out: "0xTokenB".parse()?,
    amount_in: 1000000000000000000u64.into(),
    min_amount_out: 900000000000000000u64.into(),
    deadline: chrono::Utc::now().timestamp() as u64 + 300,
};

let result = dex.swap_exact_tokens_for_tokens(swap).await?;
println!("Swapped {} for {}", result.amount_in, result.amount_out);

// 借贷协议
let lending = LendingProtocol::new("0xLendingPool".parse()?);
let deposit_tx = lending.deposit(
    "0xToken".parse()?,
    1000000000000000000u64.into(),
).await?;
```

---

#### 6. 身份和 DID

```rust
use beebotos_chain::identity::{DIDDocument, DIDResolver, VerificationMethod};

// 创建 DID 文档
let mut doc = DIDDocument::new("did:beebotos:agent-123");

// 添加验证方法
let vm = VerificationMethod::new(
    "did:beebotos:agent-123#keys-1",
    "Ed25519VerificationKey2020",
    "0xPublicKey".to_string(),
);
doc.add_verification_method(vm);

// 解析 DID
let resolver = DIDResolver::new();
let resolved = resolver.resolve("did:beebotos:agent-123").await?;
println!("DID Document: {:?}", resolved);
```

---

### 📋 核心功能模块

| 模块 | 路径 | 功能 |
|------|------|------|
| **monad** | `src/monad/` | Monad 区块链客户端 |
| **dao** | `src/dao/` | DAO 治理合约交互 |
| **wallet** | `src/wallet/` | HD 钱包和密钥管理 |
| **bridge** | `src/bridge/` | 跨链桥接 |
| **identity** | `src/identity/` | DID 身份系统 |
| **defi** | `src/defi/` | DeFi 协议交互 |
| **oracle** | `src/oracle/` | 价格预言机 |
| **bindings** | `src/bindings/` | 智能合约绑定 |

---

### ⚙️ Chain 配置

```rust
use beebotos_chain::ChainConfig;

// 预设配置
let config = ChainConfig::monad_testnet();

// 自定义配置
let config = ChainConfig {
    rpc_url: "https://rpc.testnet.monad.xyz".to_string(),
    chain_id: 10143,
    dao_address: Some("0xDAO...".parse()?),
    treasury_address: Some("0xTreasury...".parse()?),
    token_address: Some("0xToken...".parse()?),
};
```

---

### 📁 项目结构

```
crates/chain/
├── Cargo.toml
└── src/
    ├── lib.rs              # 库入口
    ├── error.rs            # 错误定义
    ├── types.rs            # 公共类型
    ├── config.rs           # 配置管理
    ├── monad/              # Monad 客户端
    │   ├── client.rs       # 区块链客户端
    │   ├── contract.rs     # 合约交互
    │   └── types.rs        # Monad 类型
    ├── dao/                # DAO 治理
    │   ├── client.rs       # DAO 客户端
    │   ├── governance.rs   # 治理逻辑
    │   ├── proposal.rs     # 提案管理
    │   ├── voting.rs       # 投票系统
    │   ├── treasury.rs     # 金库管理
    │   └── delegation.rs   # 委托机制
    ├── wallet/             # 钱包管理
    │   ├── mod.rs          # 钱包实现
    │   └── hd_wallet.rs    # 分层确定性钱包
    ├── bridge/             # 跨链桥
    │   ├── mod.rs          # 桥接核心
    │   ├── router.rs       # 路由管理
    │   └── adapters/       # 各链适配器
    ├── identity/           # 身份系统
    │   ├── did.rs          # DID 实现
    │   ├── registry.rs     # 身份注册表
    │   └── credentials.rs  # 可验证凭证
    ├── defi/               # DeFi 集成
    │   ├── dex.rs          # DEX 交互
    │   └── lending.rs      # 借贷协议
    ├── oracle/             # 预言机
    │   └── mod.rs          # 价格 feed
    └── bindings/           # 合约绑定
        ├── agent_dao.rs    # AgentDAO 合约
        ├── agent_identity.rs # 身份合约
        └── a2a_commerce.rs # A2A 商务合约
```

---

### 🛠 技术栈

| 组件 | 用途 |
|------|------|
| **ethers** | 以太坊/EVM 区块链交互 |
| **tokio** | 异步运行时 |
| **serde** | 序列化 |
| **url** | URL 解析 |

---

### ⚠️ 注意事项

1. **私钥安全** - 生产环境使用硬件钱包或密钥管理系统
2. **Gas 费用** - 注意 Monad 网络的 gas 价格和限额
3. **合约地址** - 确保使用正确的合约地址（测试网/主网）
4. **确认区块** - 根据安全需求设置适当的确认区块数

需要我帮你实现具体的区块链交互功能或提供其他使用示例吗？


