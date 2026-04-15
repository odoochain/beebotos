

## beebotos-cli 编译和使用指南

**beebotos-cli** 是 BeeBotOS 的 **命令行工具**，提供完整的 Agent 管理、DAO 治理、技能操作等功能。可执行文件名为 `beebot`。

---

### 📦 编译命令

#### 1. 编译整个项目（包含 CLI）
```bash
# 项目根目录
cargo build --release

# 编译后的可执行文件
./target/release/beebot
```

#### 2. 只编译 CLI
```bash
# 编译 beebotos-cli 包
cargo build --release -p beebotos-cli

# 或指定二进制文件
cargo build --release --bin beebot
```

#### 3. 安装到系统 PATH
```bash
# 安装到 ~/.cargo/bin
cargo install --path apps/cli

# 现在可以直接使用
beebot --help
```

#### 4. 开发调试模式
```bash
# 快速编译（无优化，用于开发）
cargo build -p beebotos-cli

# 运行
cargo run -p beebotos-cli -- --help
```

---

### 🚀 使用方法

#### 基本语法
```bash
beebot [OPTIONS] <COMMAND>

# 全局选项
-e, --endpoint <URL>    # API 端点（默认: http://localhost:8080）
-h, --help              # 帮助
-V, --version           # 版本
```

---

### 📋 命令清单

#### 🤖 **Agent 管理** (`beebot agent`)

| 命令 | 说明 | 示例 |
|------|------|------|
| `list` | 列出所有 Agent | `beebot agent list --limit 10` |
| `create` | 创建新 Agent | `beebot agent create MyAgent -d "描述" -c skill1,skill2` |
| `get` | 查看 Agent 详情 | `beebot agent get agent-123` |
| `delete` | 删除 Agent | `beebot agent delete agent-123` |
| `task` | 执行任务 | `beebot agent task agent-123 -t analyze -i "data"` |
| `spawn` | 创建子 Agent | `beebot agent spawn agent-123 SubAgent "完成子任务"` |
| `state` | 查看状态 | `beebot agent state agent-123` |
| `subagents` | 列出子 Agent | `beebot agent subagents agent-123` |

**示例：**
```bash
# 创建新 Agent
beebot agent create WebCrawler \
  --description "自动爬取网页内容的Agent" \
  --capability http,parser,storage

# 执行任务
beebot agent task WebCrawler \
  --task-type crawl \
  --input "https://example.com"

# 列出所有 Agent
beebot agent list --limit 20
```

---

#### 🧠 **认知功能** (`beebot brain`)

| 命令 | 说明 | 示例 |
|------|------|------|
| `emotion` | 查看情绪状态 | `beebot brain emotion agent-123` |
| `personality` | 查看人格特质 | `beebot brain personality agent-123` |
| `memory` | 查询记忆 | `beebot brain memory agent-123 "关键词"` |
| `neural` | 神经网络状态 | `beebot brain neural agent-123` |
| `evolve` | 触发进化 | `beebot brain evolve agent-123 --generations 10` |

**示例：**
```bash
# 查看 Agent 情绪状态
beebot brain emotion MyAgent
# 输出:
# Emotion state of: MyAgent
#   Pleasure:  0.5
#   Arousal:   0.3
#   Dominance: 0.4
#   Current:   content 😊

# 查看人格特质（OCEAN模型）
beebot brain personality MyAgent

# 触发进化优化
beebot brain evolve MyAgent --generations 50
```

---

#### 🏛️ **DAO 治理** (`beebot dao`)

| 命令 | 说明 | 示例 |
|------|------|------|
| `proposals` | 列出提案 | `beebot dao proposals --status active` |
| `propose` | 创建提案 | `beebot dao propose "标题" "描述"` |
| `proposal` | 查看提案详情 | `beebot dao proposal 123` |
| `vote` | 投票 | `beebot dao vote 123 for --reason "支持"` |
| `execute` | 执行提案 | `beebot dao execute 123` |
| `treasury` | 查看金库 | `beebot dao treasury` |
| `delegate` | 委托投票权 | `beebot dao delegate 0x1234...` |
| `member` | 查看成员信息 | `beebot dao member 0x1234...` |

**示例：**
```bash
# 创建提案
beebot dao propose "增加新功能" "建议添加自动备份功能"

# 投票（for/against/abstain）
beebot dao vote 42 for --reason "这个功能很有用"

# 查看金库余额
beebot dao treasury
# 输出:
# Treasury Balance:
#   Native: 1000.0
#   BEE:    1000000.0
```

---

#### 🛠️ **技能管理** (`beebot skill`)

| 命令 | 说明 | 示例 |
|------|------|------|
| `list` | 列出技能 | `beebot skill list --category dev` |
| `get` | 查看技能详情 | `beebot skill get skill-123` |
| `upload` | 上传技能 | `beebot skill upload ./my-skill.wasm` |
| `execute` | 执行技能 | `beebot skill execute skill-123 -p key=value` |
| `delete` | 删除技能 | `beebot skill delete skill-123` |
| `categories` | 列出分类 | `beebot skill categories` |

**示例：**
```bash
# 列出开发类技能
beebot skill list --category development

# 上传新技能
beebot skill upload ./target/wasm32-unknown-unknown/release/my_skill.wasm

# 执行技能并传参
beebot skill execute http-client \
  --param url=https://api.example.com \
  --param method=GET
```

---

#### ⚙️ **系统操作** (`beebot system`)

| 命令 | 说明 | 示例 |
|------|------|------|
| `health` | 健康检查 | `beebot system health` |
| `status` | 系统状态 | `beebot system status` |
| `version` | 版本信息 | `beebot system version` |

**示例：**
```bash
# 健康检查
beebot system health
# 输出: ✓ Healthy

# 查看系统状态
beebot system status
# 输出:
# System Status:
#   Agents: 10 total (5 active, 5 idle)
#   Tasks: 3 pending, 2 running, 100 completed
#   CPU: 25.5%
#   Memory: 512 MB
```

---

### 🔧 高级用法

#### 指定 API 端点
```bash
# 连接到远程节点
beebot -e https://api.beebotos.io agent list

# 或设置环境变量
export BEEBOT_ENDPOINT=http://localhost:9090
beebot agent list
```

#### 管道和脚本
```bash
# 批量创建 Agent
for name in Agent1 Agent2 Agent3; do
  beebot agent create "$name" --capability basic
done

# 获取所有 Agent 并处理
beebot agent list | grep "active" | wc -l
```

---

### 🛠 技术栈

| 组件 | 用途 |
|------|------|
| **clap** | 命令行参数解析 |
| **tokio** | 异步运行时 |
| **colored** | 终端彩色输出 |
| **indicatif** | 进度条 |
| **dialoguer** | 交互式提示 |
| **table** | 表格输出 |
| **config** | 配置文件管理 |

---

### 📁 项目结构

```
apps/cli/
├── Cargo.toml
└── src/
    ├── main.rs           # CLI 入口 & 命令定义
    ├── commands/
    │   └── mod.rs        # 额外命令模块（备用）
    ├── client.rs         # API 客户端
    ├── output.rs         # 输出格式化
    ├── completion.rs     # Shell 补全
    └── ...               # 其他模块
```

---

### ⚠️ 注意事项

1. **当前为 Stub 实现** - 命令已定义但大部分功能返回模拟数据
2. **API 连接** - 默认连接 `http://localhost:8080`，需确保后端服务运行
3. **配置文件** - 支持从 `~/.beebot/config.toml` 读取配置

需要我帮你实现完整的 API 客户端功能或添加新的命令吗？

