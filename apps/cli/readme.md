
已完成！我创建了 **3 份详细的调试和测试指导文档**：

---

## 📄 创建的文档

### 1. `DEBUGGING_AND_TESTING.md` (完整指南)
**12KB+ 详细文档**，包含：
- 快速开始和环境准备
- **日志调试**：4 个级别 (-v 到 -vvv)、JSON 格式、重定向技巧
- **网络调试**：代理诊断、连接检查、抓包指南
- **测试策略**：单元测试、集成测试、E2E 测试、Mock 测试
- **故障排除**：常见问题快速修复、完整诊断脚本
- **高级技巧**：GDB/LLDB、VS Code 调试、性能分析 (perf/valgrind)
- **CI/CD 配置**：GitHub Actions 示例

### 2. `DEBUG_CHEATSHEET.md` (速查表)
**5KB 快速参考**，包含：
- 日志级别对照表
- 常用调试命令（环境检查、网络、配置）
- 测试命令速查（cargo test 变体）
- 环境变量参考表
- 常见问题快速修复
- 性能调试命令
- 一键诊断脚本

### 3. `TEST_EXAMPLES.md` (测试示例)
**17KB 代码示例**，包含：
- **单元测试示例**：client、error、config、logging 模块
- **集成测试示例**：完整 CLI 测试流程
- **Mock 测试示例**：使用 mockito 模拟 API
- **性能测试示例**：Criterion 基准测试、并发测试
- **最佳实践**：命名规范、Given-When-Then 模式、测试固件

---

## 🚀 快速使用

```bash
# 查看速查表
cat apps/cli/DEBUG_CHEATSHEET.md

# 查看完整调试指南
cat apps/cli/DEBUGGING_AND_TESTING.md

# 查看测试示例
cat apps/cli/TEST_EXAMPLES.md
```

### 常用调试命令

```bash
# 基础调试
cargo run -- -v agent list           # 信息级别
cargo run -- -vv agent create --name test  # 调试级别
cargo run -- -vvv --log-format json agent list 2>&1 | jq  # JSON + 追踪

# 测试
cargo test --lib                     # 单元测试
cargo test --test integration_tests  # 集成测试
cargo test -- --nocapture            # 显示输出

# 诊断脚本
cd apps/cli && bash -c 'source <(grep -A 100 "quick_diagnose.sh" DEBUG_CHEATSHEET.md | tail -n +2)'
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成报告
cd apps/cli && cargo tarpaulin --out Html
open tarpaulin-report.html
```

---

## 📊 关键要点

| 场景 | 命令/方法 |
|-----|----------|
| 查看 API 调用 | `cargo run -- -vv agent list` |
| 检查代理设置 | `HTTP_PROXY=socks5://host:port cargo run -- -vv ...` |
| 测试网络连接 | `cargo run -- -vvv agent list 2>&1 \| grep -i network` |
| 运行单元测试 | `cargo test --lib` |
| 运行集成测试 | `cargo test --test integration_tests` |
| 性能基准测试 | `cargo bench` |
| 代码覆盖率 | `cargo tarpaulin --out Html` |

所有文档都位于 `apps/cli/` 目录下，可以直接在项目中使用！


完成了！我为您创建了 **4 份新的详细技术文档**，总计约 **150KB** 的内容：

## 📚 创建的文档

### 1. [ARCHITECTURE.md](./ARCHITECTURE.md) (~50KB)
**全面的架构设计文档**，包含：
- 整体架构图（4 层架构）
- 10 个核心模块的详细设计（职责、数据结构、流程）
- 数据流分析（命令执行、配置加载、错误处理、网络请求）
- 接口关系定义（依赖图、关键接口）
- 业务逻辑关系（代理生命周期、技能管理、会话流程）
- 扩展指南（添加新命令、新基础设施、修改流程）

### 2. [CODE_LOGIC_ANALYSIS.md](./CODE_LOGIC_ANALYSIS.md) (~27KB)
**代码逻辑深度分析**，包含：
- 核心调用链分析（完整执行流程）
- 关键数据结构关系（实体关系图）
- 状态机分析（Agent/ Task 状态转换）
- 函数调用频次分析
- 并发模型分析（连接池管理）
- 内存管理分析（对象生命周期）
- 关键代码路径（成功/失败场景）
- 性能瓶颈分析
- 安全分析（敏感数据流）
- 扩展点分析

### 3. [MODULE_RELATIONSHIPS.md](./MODULE_RELATIONSHIPS.md) (~35KB)
**模块关系和业务逻辑图**，包含：
- 模块依赖关系图（分层架构）
- 数据流向图（3 种主要流程）
- 业务逻辑关系（Agent/技能/会话/消息生命周期）
- 函数调用关系矩阵
- 模块间接口契约
- 扩展接口设计规范

### 4. [TECHNICAL_DOCUMENTATION_INDEX.md](./TECHNICAL_DOCUMENTATION_INDEX.md) (~6KB)
**技术文档索引**，包含：
- 所有文档的导航和适用读者
- 核心概念速览
- 常用代码片段
- 架构决策记录（ADR）
- 性能基准数据

---

## 📊 文档关系图

```
技术文档套件
    │
    ├── 架构设计
    │   └── ARCHITECTURE.md (50KB)
    │       ├── 整体架构
    │       ├── 模块详解 x10
    │       ├── 数据流分析
    │       └── 扩展指南
    │
    ├── 代码分析
    │   ├── CODE_LOGIC_ANALYSIS.md (27KB)
    │   │   ├── 调用链分析
    │   │   ├── 状态机
    │   │   └── 性能分析
    │   │
    │   └── MODULE_RELATIONSHIPS.md (35KB)
    │       ├── 依赖关系图
    │       ├── 业务逻辑流
    │       └── 接口契约
    │
    ├── 调试测试 (之前创建)
    │   ├── DEBUGGING_AND_TESTING.md (13KB)
    │   ├── DEBUG_CHEATSHEET.md (5KB)
    │   └── TEST_EXAMPLES.md (17KB)
    │
    └── 索引
        └── TECHNICAL_DOCUMENTATION_INDEX.md (6KB)
```

---

## 🎯 使用建议

| 场景 | 推荐文档 |
|------|---------|
| 想了解整体架构 | ARCHITECTURE.md |
| 想深入理解代码逻辑 | CODE_LOGIC_ANALYSIS.md |
| 想知道模块间关系 | MODULE_RELATIONSHIPS.md |
| 遇到 Bug 需要调试 | DEBUGGING_AND_TESTING.md + DEBUG_CHEATSHEET.md |
| 需要添加新功能 | ARCHITECTURE.md 的"扩展指南"章节 |
| 快速查找信息 | TECHNICAL_DOCUMENTATION_INDEX.md |

所有文档都位于 `apps/cli/` 目录下，可以直接在项目中查阅！



