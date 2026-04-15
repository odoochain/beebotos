
## beebotos-beehub 编译和使用指南

**BeeHub** 是 BeeBotOS 的 **Skill 市场（技能市场）** 应用，提供技能包的发布、发现、下载等功能。

---

### 📦 编译命令

#### 1. 编译整个项目（包含 beehub）
```bash
# 在项目根目录
cargo build --release

# 或者只编译 beehub
cargo build --release -p beebotos-beehub
```

#### 2. 单独运行
```bash
# 运行
cargo run -p beebotos-beehub
# 或
cargo run --bin beehub
```

#### 3. 检查编译
```bash
cargo check -p beebotos-beehub
```

---

### 🚀 使用方法

#### 启动服务
```bash
# 默认监听 0.0.0.0:3000
cargo run -p beebotos-beehub

# 输出
INFO BeeHub listening on 0.0.0.0:3000
```

#### API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | 首页 - 返回服务信息 |
| GET | `/api/skills` | 列出所有技能 |
| POST | `/api/skills` | 发布新技能 |
| GET | `/api/skills/:id` | 获取技能详情 |
| GET | `/api/skills/:id/download` | 下载技能包 |

---

### 📡 测试示例

```bash
# 1. 启动服务
cargo run -p beebotos-beehub &

# 2. 测试 API
curl http://localhost:3000/

# 3. 列出技能
curl http://localhost:3000/api/skills | jq

# 4. 发布技能
curl -X POST http://localhost:3000/api/skills \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-skill",
    "version": "1.0.0",
    "description": "A cool skill",
    "license": "MIT",
    "repository": "https://github.com/user/skill"
  }'

# 5. 获取技能详情
curl http://localhost:3000/api/skills/http-client

# 6. 下载技能
curl http://localhost:3000/api/skills/http-client/download
```

---

### 🛠 技术栈

| 组件 | 用途 |
|------|------|
| **axum** | Web 框架 |
| **sqlx** | SQLite 数据库（当前版本 0.8.3）|
| **tokio** | 异步运行时 |
| **tower-http** | CORS、限流等中间件 |
| **serde** | JSON 序列化 |

---

### ⚙️ 配置说明

当前是 **Stub 实现**（占位符），数据是硬编码的：

- 数据存储：目前返回硬编码数据，未连接真实数据库
- 端口：固定 `3000`
- 数据库：已配置 SQLite，但未启用实际存储逻辑

如需完整功能，需要：
1. 实现 `storage.rs` 中的数据库存储逻辑
2. 添加配置文件支持（如 `config.toml`）
3. 实现文件上传/下载功能（技能包文件）

---

### 📁 项目结构

```
apps/beehub/
├── Cargo.toml          # 依赖配置
└── src/
    ├── main.rs         # 入口 - 路由配置
    ├── handlers.rs     # API 处理函数
    ├── models.rs       # 数据模型（Skill 等）
    └── storage.rs      # 存储层（Stub）
```

需要我帮你实现完整的数据库存储功能吗？

