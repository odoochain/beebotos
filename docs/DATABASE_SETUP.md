# BeeBotOS 数据库设置指南

## 概述

BeeBotOS 使用 **SQLite** 作为数据库，它是一个轻量级、无需服务器配置的嵌入式数据库。

## 快速开始

### 1. SQLite 已内置

SQLite 不需要单独安装，它已经通过 `rusqlite` 和 `sqlx` crate 集成到项目中。

### 2. 数据库文件位置

默认数据库文件位置：
- 开发环境：`data/beebotos.db`
- 测试环境：`data/beebotos_test.db`

### 3. 自动初始化

数据库会在应用启动时自动创建并运行迁移：

```bash
# 启动网关
./target/release/beebotos-gateway --config config/beebotos.toml
```

或者使用 cargo 运行：

```bash
cargo run -p beebotos-gateway -- --config config/beebotos.toml
```

## 数据库结构

### 核心表

| 表名 | 说明 |
|------|------|
| `agents` | AI Agent 信息 |
| `sessions` | 会话管理 |
| `memories` | 记忆存储 |
| `skills` | 技能市场 |
| `tasks` | 任务队列 |
| `transactions` | 区块链交易 |
| `events` | 系统事件日志 |

### A2A 相关表

| 表名 | 说明 |
|------|------|
| `a2a_deals` | Agent 间交易 |
| `a2a_negotiations` | 协商记录 |
| `a2a_capabilities` | 能力注册 |
| `a2a_messages` | A2A 消息 |
| `agent_reputation` | Agent 信誉评分 |

### 系统表

| 表名 | 说明 |
|------|------|
| `system_settings` | 系统配置 |
| `dao_proposals` | DAO 提案 |
| `dao_votes` | 投票记录 |

## 配置连接

编辑 `config/beebotos.toml` 文件：

```toml
[database]
url = "sqlite:data/beebotos.db"
max_connections = 20
min_connections = 5
connect_timeout_seconds = 10
idle_timeout_seconds = 600
run_migrations = true
```

或者使用环境变量：

```env
DATABASE_URL="sqlite:data/beebotos.db"
```

## 验证安装

### 使用 sqlite3 命令行工具

```bash
# 连接到数据库
sqlite3 data/beebotos.db

# 查看所有表
.tables

# 查看系统设置
SELECT * FROM system_settings;

# 查看默认 Agent
SELECT * FROM agents;

# 退出
.quit
```

### 检查数据库文件

```bash
# 检查数据库文件是否存在
ls -lh data/beebotos.db

# 检查文件大小
du -h data/beebotos.db
```

## 常见问题

### Q: 数据库文件权限问题

```bash
# 确保目录存在且有写入权限
mkdir -p data
chmod 755 data

# 如果是已有数据库，确保可写
chmod 664 data/beebotos.db
```

### Q: 迁移失败

```bash
# 删除数据库文件后重试
rm data/beebotos.db
# 然后重新启动应用
```

### Q: 如何查看 SQL 日志

在 `config/beebotos.toml` 中启用调试日志：

```toml
[logging]
level = "debug"
```

## 备份与恢复

### 备份

SQLite 数据库备份非常简单，直接复制文件即可：

```bash
# 复制数据库文件
cp data/beebotos.db /backup/beebotos_backup_$(date +%Y%m%d).db

# 或使用 sqlite3 的备份命令
sqlite3 data/beebotos.db ".backup /backup/beebotos_backup.db"
```

### 恢复

```bash
# 从备份恢复
cp /backup/beebotos_backup.db data/beebotos.db

# 或使用 sqlite3 恢复
sqlite3 data/beebotos.db ".restore /backup/beebotos_backup.db"
```

## 高级用法

### 使用 sqlite3 进行维护

```bash
# 分析并优化数据库
sqlite3 data/beebotos.db "ANALYZE;"

# 清理碎片
sqlite3 data/beebotos.db "VACUUM;"

# 检查完整性
sqlite3 data/beebotos.db "PRAGMA integrity_check;"
```

### 导出数据

```bash
# 导出为 SQL
sqlite3 data/beebotos.db ".dump" > beebotos_export.sql

# 导出特定表
sqlite3 data/beebotos.db ".dump agents" > agents_export.sql
```

### 导入数据

```bash
# 从 SQL 文件导入
sqlite3 data/beebotos.db < beebotos_export.sql
```
