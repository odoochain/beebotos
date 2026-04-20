# BeeBotOS Web — Gateway 交互式配置向导 Web 化设计文档

> 将 `apps/gateway/src/config_wizard.rs` 的 CLI TUI 向导能力转化为 Web 前端配置向导的方案  
> 版本：v1.0  
> 生成时间：2026-04-19

---

## 一、设计背景

### 1.1 现状分析

`apps/gateway` 提供了强大的**命令行交互式配置向导**（`config_wizard.rs`），支持：

- 12 个配置分类的引导式配置（Server、Database、JWT、Models、Channels、Blockchain、Security、Logging、Metrics、TLS）
- 配置变更追踪与自动备份
- 配置预览（保存前确认）
- 多格式导出（`.env` / `docker-compose` / `Kubernetes ConfigMap`）
- 从 `.env` 文件导入配置
- 多主题 TUI 界面

**触发方式：**
```bash
./beebotos-gateway --wizard
./beebotos-gateway --interactive
./beebotos-gateway --configure
```

### 1.2 问题与机会

**当前痛点：**
- 配置向导仅在服务器本地通过 CLI 使用，远程管理不便
- Web 前端 `SettingsPage` 完全没有配置向导能力
- 新用户部署时需要在服务器上手动运行 CLI，无法通过 Web UI 完成初始化
- 多环境（dev/staging/prod）配置管理缺乏可视化工具

**Web 化价值：**
- 降低部署门槛：新用户可通过浏览器完成 Gateway 初始化配置
- 远程管理：管理员无需 SSH 登录服务器即可查看和调整配置
- 配置版本化：支持配置历史对比、回滚（结合 Gateway 的自动备份机制）
- 团队协作：配置可导出分享，支持环境变量批量导入

---

## 二、设计原则

1. **与 CLI 向导功能对等**：Web 向导覆盖 CLI 的全部 12 个配置分类
2. **与 Gateway 配置结构对齐**：表单字段与 `BeeBotOSConfig` 结构体一一对应
3. **敏感信息保护**：API Key、JWT Secret、钱包助记词等字段加密传输、脱敏展示
4. **配置即代码**：最终输出为可下载的 `beebotos.toml`，支持多种部署格式导出
5. **渐进式披露**：基础配置优先展示，高级配置可折叠

---

## 三、页面定位与路由

```
主路由：/setup            （首次部署引导，无认证要求或仅 basic auth）
子路由：/settings/wizard   （已有系统下的重新配置，需 admin 权限）

入口场景：
  1. 全新部署：访问 Web UI 时检测到 Gateway 未配置，自动重定向到 /setup
  2. 管理员入口：Settings 页面增加 "Configuration Wizard" 卡片
  3. 快捷操作：顶部导航下拉菜单增加 "Reconfigure Gateway"
```

---

## 四、向导流程设计

### 4.1 步骤总览（10 步，与 CLI 对齐）

```
Step 1:  Welcome          → 欢迎页 + 配置模式选择（全新 / 导入 / 模板）
Step 2:  Server           → HTTP/gRPC 服务器、CORS、超时
Step 3:  Database         → SQLite/Postgres、连接池、迁移
Step 4:  JWT & Security   → Secret 生成、过期时间、速率限制
Step 5:  LLM Models       → Provider 选择、API Key、模型参数
Step 6:  Channels         → 通信频道启用 + 各平台参数
Step 7:  Blockchain       → 链配置、合约地址、钱包（可选）
Step 8:  Logging & Obs    → 日志级别、Prometheus、OpenTelemetry
Step 9:  Review           → 配置预览、变更对比、校验
Step 10: Deploy           → 导出 TOML / 生成命令 / 部署指引
```

### 4.2 每一步的详细设计

---

#### Step 1: Welcome（欢迎页）

**目的：** 确定配置模式，降低用户认知负担

**内容：**
```
┌─────────────────────────────────────────────┐
│  🐝 BeeBotOS Gateway Setup                  │
│                                             │
│  Choose your configuration mode:            │
│                                             │
│  [🆕 Start Fresh]                           │
│    Create a new configuration from scratch  │
│                                             │
│  [📥 Import Existing]                       │
│    Upload an existing .env or .toml file    │
│                                             │
│  [📋 Use Template]                          │
│    Start from a preset configuration        │
│      • Minimal (local only)                 │
│      • Standard (with LLM + Channels)       │
│      • Enterprise (full stack)              │
│                                             │
└─────────────────────────────────────────────┘
```

**模板预设（与 CLI 对齐）：**

| 模板 | 默认启用 | 说明 |
|------|---------|------|
| **Minimal** | SQLite、单 Provider（Kimi）、WebChat | 本地快速体验 |
| **Standard** | SQLite、3 Providers、5 Channels、Prometheus | 标准生产配置 |
| **Enterprise** | Postgres、全 Providers、全 Channels、TLS、mTLS、OTLP | 企业级完整配置 |

---

#### Step 2: Server Configuration

**对应 Gateway 结构：** `ServerConfig`

**表单字段：**

| 字段 | 类型 | 默认值 | 校验规则 |
|------|------|--------|---------|
| Host | text | `0.0.0.0` | IP 地址格式 |
| HTTP Port | number | `8000` | 1-65535 |
| gRPC Port | number | `50051` | 1-65535 |
| CORS Origins | text[] | `["*"]` | URL 格式 |
| Request Timeout | number | `30` | 秒，≥1 |
| Max Body Size | number | `10` | MB，≥1 |

**高级选项（折叠）：**
- TLS Enabled (toggle)
- TLS Cert Path (file picker)
- TLS Key Path (file picker)
- mTLS Enabled (toggle)

---

#### Step 3: Database Configuration

**对应 Gateway 结构：** `DatabaseConfig`

**表单字段：**

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| Database Type | select | `sqlite` | SQLite / PostgreSQL |
| SQLite Path | text | `data/beebotos.db` | 文件路径 |
| Postgres URL | text | - | `postgres://user:pass@host/db` |
| Max Connections | number | `10` | ≥1 |
| Min Connections | number | `2` | ≥1 |
| Connect Timeout | number | `30` | 秒 |
| Auto Migrate | toggle | `true` | 启动时自动运行迁移 |

**测试连接按钮：**
```rust
<button on:click=move |_| {
    // 调用 Gateway 新增端点：POST /api/v1/admin/db/test-connection
    // 或前端本地校验 URL 格式
}>
    "Test Connection"
</button>
```

---

#### Step 4: JWT & Security

**对应 Gateway 结构：** `JwtConfig` + `SecurityConfig` + `RateLimitConfig`

**表单字段：**

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| JWT Secret | password | 自动生成 | ≥32 字符，提供 "Regenerate" 按钮 |
| Token Expiry | number | `3600` | 秒 |
| Refresh Token Expiry | number | `604800` | 秒（7天） |
| Rate Limit Enabled | toggle | `true` | |
| QPS Limit | number | `100` | |
| Burst Limit | number | `200` | |
| Webhook IP Whitelist | text[] | `[]` | IP/CIDR 格式 |

**JWT Secret 生成器：**
```rust
fn generate_jwt_secret() -> String {
    // 使用 web crypto API 生成 256-bit random
    // 格式：Base64 或 Hex
}
```

---

#### Step 5: LLM Models Configuration

**对应 Gateway 结构：** `ModelsConfig` + `HashMap<String, ModelProviderConfig>`

**这是最关键的一步，与 LLM 配置页面设计文档联动。**

**表单结构：**

```
Default Provider: [Kimi ▼]
Fallback Chain:   [Kimi] [→] [OpenAI] [→] [+] [-]
System Prompt:    [textarea]
Max Tokens:       [4096 ▼]
Request Timeout:  [60] seconds
Cost Optimization [✓]

+--------------------------------------------------+
| Provider: Kimi                              [🗑] |
| API Key:    [**************************]  [👁]   |
| Model:      [moonshot-v1-8k ▼]                   |
| Base URL:   [https://api.moonshot.cn]            |
| Temperature:[0.7]                                |
| Context:    [8192]                               |
+--------------------------------------------------+
| Provider: OpenAI                            [🗑] |
| ...                                              |
+--------------------------------------------------+

[+ Add Provider]
```

**Provider 预设（一键填充）：**

| Provider | 默认 Model | Base URL |
|----------|-----------|----------|
| Kimi | `moonshot-v1-8k` | `https://api.moonshot.cn` |
| OpenAI | `gpt-4` | `https://api.openai.com/v1` |
| Anthropic | `claude-3-sonnet` | `https://api.anthropic.com` |
| Zhipu | `glm-4` | `https://open.bigmodel.cn/api/paas/v4` |
| DeepSeek | `deepseek-chat` | `https://api.deepseek.com` |
| Ollama | `llama2` | `http://localhost:11434` |

**校验逻辑：**
- API Key 非空（至少 10 字符）
- Base URL 为有效 URL
- Temperature 范围 0.0 - 2.0
- 至少配置一个 Provider

---

#### Step 6: Channels Configuration

**对应 Gateway 结构：** `ChannelsConfig`

**表单结构：**

```
Global Settings:
  [✓] Auto Download Media
  Max File Size: [50] MB
  Context Window: [20] messages
  [✓] Auto Reply
  Default Agent ID: [agent-001 ▼]

Enabled Platforms:
  [✓] WebChat     [Configure ▼]
  [✓] Lark        [Configure ▼]
  [ ] DingTalk    [Configure ▼]
  [✓] Telegram    [Configure ▼]
  [ ] Discord     [Configure ▼]
  [ ] Slack       [Configure ▼]
  [ ] WeChat      [Configure ▼]
  ...

+--------------------------------------------------+
| Lark Configuration                                 |
| App ID:        [________________]                  |
| App Secret:    [________________]                  |
| Encrypt Key:   [________________]                  |
| Verification:  [________________]                  |
+--------------------------------------------------+
```

**平台配置字段（以 Lark 为例）：**

| 字段 | 类型 | 来源 |
|------|------|------|
| App ID | text | `settings["app_id"]` |
| App Secret | password | `settings["app_secret"]` |
| Encrypt Key | password | `settings["encrypt_key"]` |
| Verification Token | text | `settings["verification_token"]` |

**平台配置字段（以 Telegram 为例）：**

| 字段 | 类型 |
|------|------|
| Bot Token | password |
| Webhook Secret | password |
| Allowed Updates | text[] |

---

#### Step 7: Blockchain Configuration（可选）

**对应 Gateway 结构：** `BlockchainConfig`

**表单字段：**

| 字段 | 类型 | 默认值 |
|------|------|--------|
| Enable Blockchain | toggle | `false` |
| Chain ID | number | `1` |
| RPC URL | text | - |
| Contract Addresses | key-value | - |
| Wallet Mnemonic | password | - |

**仅在启用时展开表单。**

---

#### Step 8: Logging & Observability

**对应 Gateway 结构：** `LoggingConfig` + `MetricsConfig` + `TracingConfig`

**表单字段：**

| 字段 | 类型 | 默认值 |
|------|------|--------|
| Log Level | select | `info` |
| Log Format | select | `json` |
| Log File Path | text | `logs/beebotos.log` |
| Log Rotation | select | `daily` |
| Enable Metrics | toggle | `true` |
| Metrics Port | number | `9090` |
| Enable Tracing | toggle | `false` |
| OTLP Endpoint | text | `http://localhost:4317` |
| Trace Sampling Rate | number | `0.1` |

---

#### Step 9: Review & Validation

**目的：** 在最终确认前展示完整配置，高亮变更和潜在问题

**内容：**

```
┌─────────────────────────────────────────────┐
│  Configuration Review                         │
│                                             │
│  ✅ All required fields filled              │
│  ⚠️  1 warning: Webhook IP whitelist is empty│
│                                             │
│  [View as TOML]  [View as ENV]  [View JSON] │
│                                             │
│  +----------------------------------------+ │
│  | [toml preview]                         | │
│  | system_name = "BeeBotOS"               | │
│  | ...                                    | │
│  +----------------------------------------+ │
│                                             │
│  Changes from current config:               │
│  • default_provider: "openai" → "kimi"      │
│  • Added Provider: DeepSeek                │
│  • Removed Channel: DingTalk               │
│                                             │
│  [⬅ Back]              [Save & Export ▼]   │
└─────────────────────────────────────────────┘
```

**校验项目：**
- [ ] 至少一个 LLM Provider 已配置且 API Key 非空
- [ ] JWT Secret ≥ 32 字符
- [ ] Database URL 非空
- [ ] Server Port 不冲突
- [ ] 启用的 Channel 已填写必要参数

---

#### Step 10: Deploy & Export

**目的：** 将配置应用到 Gateway 并提供多种部署格式

**选项卡设计：**

```
┌─────────────────────────────────────────────┐
│  Deploy Configuration                       │
│                                             │
│  [Download TOML]  [ENV Export]  [Docker] [K8s]│
│                                             │
│  +----------------------------------------+ │
│  | Option 1: Manual Deployment            | │
│  |                                        | │
│  | 1. Download beebotos.toml              | │
│  | 2. Upload to your server               | │
│  | 3. Place in config/ directory          | │
│  | 4. Restart Gateway                     | │
│  |                                        | │
│  | [Download beebotos.toml]               | │
│  +----------------------------------------+ │
│                                             │
│  +----------------------------------------+ │
│  | Option 2: Environment Variables        | │
│  |                                        | │
│  | BEE__SERVER__HOST=0.0.0.0              | │
│  | BEE__SERVER__PORT=8000                 | │
│  | ...                                    | │
│  |                                        | │
│  | [Copy to Clipboard] [Download .env]    | │
│  +----------------------------------------+ │
│                                             │
│  +----------------------------------------+ │
│  | Option 3: Docker Compose               | │
│  |                                        | │
│  | services:                              | │
│  |   gateway:                             | │
│  |     environment:                       | │
│  |       - BEE__SERVER__HOST=0.0.0.0      | │
│  | ...                                    | │
│  |                                        | │
│  | [Download docker-compose.yml]          | │
│  +----------------------------------------+ │
│                                             │
│  +----------------------------------------+ │
│  | Option 4: Kubernetes                   | │
│  |                                        | │
│  | apiVersion: v1                         | │
│  | kind: ConfigMap                        | │
│  | ...                                    | │
│  |                                        | │
│  | [Download configmap.yaml]              | │
│  +----------------------------------------+ │
│                                             │
└─────────────────────────────────────────────┘
```

---

## 五、前端架构设计

### 5.1 页面组件结构

```
pages/
└── setup.rs                    # 主向导页面
    ├── WizardLayout            # 步骤导航 + 内容区框架
    ├── StepWelcome             # Step 1
    ├── StepServer              # Step 2
    ├── StepDatabase            # Step 3
    ├── StepSecurity            # Step 4
    ├── StepLlmModels           # Step 5
    ├── StepChannels            # Step 6
    ├── StepBlockchain          # Step 7
    ├── StepObservability       # Step 8
    ├── StepReview              # Step 9
    └── StepDeploy              # Step 10

components/
└── wizard/
    ├── WizardStepper.rs        # 步骤指示器（顶部进度条）
    ├── WizardNavigation.rs     # 底部导航（Back / Next / Save）
    ├── ConfigPreview.rs        # TOML/ENV/JSON 预览
    ├── ProviderCard.rs         # LLM Provider 卡片
    ├── ChannelConfigForm.rs    # 频道配置表单（动态字段）
    └── SecretInput.rs          # 带生成/显示切换的密码输入
```

### 5.2 状态管理

```rust
// state/wizard.rs

#[derive(Clone, Debug)]
pub struct WizardState {
    pub current_step: usize,           // 1-10
    pub mode: WizardMode,              // Fresh / Import / Template
    pub config: BeeBotOSConfigDraft,   // 正在编辑的配置草稿
    pub original_config: Option<BeeBotOSConfigDraft>, // 用于变更对比
    pub validation_errors: Vec<ValidationError>,
    pub is_submitting: bool,
}

#[derive(Clone, Debug)]
pub enum WizardMode {
    Fresh,
    Import(String),  // imported file content
    Template(String), // template name
}

impl WizardState {
    pub fn can_proceed(&self) -> bool {
        self.validation_errors.is_empty()
    }
    
    pub fn changes(&self) -> Vec<ConfigChange> {
        // 对比 config 和 original_config
    }
    
    pub fn export_toml(&self) -> String {
        // 序列化为 TOML 字符串
    }
    
    pub fn export_env(&self) -> String {
        // 转换为 BEE__* 格式环境变量
    }
}
```

### 5.3 配置草稿结构

```rust
// 与 Gateway BeeBotOSConfig 对齐，但所有字段为可选（用于增量编辑）

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BeeBotOSConfigDraft {
    pub system_name: Option<String>,
    pub version: Option<String>,
    pub server: Option<ServerConfigDraft>,
    pub database: Option<DatabaseConfigDraft>,
    pub jwt: Option<JwtConfigDraft>,
    pub tls: Option<TlsConfigDraft>,
    pub models: Option<ModelsConfigDraft>,
    pub channels: Option<ChannelsConfigDraft>,
    pub logging: Option<LoggingConfigDraft>,
    pub metrics: Option<MetricsConfigDraft>,
    pub tracing: Option<TracingConfigDraft>,
    pub rate_limit: Option<RateLimitConfigDraft>,
    pub security: Option<SecurityConfigDraft>,
    pub blockchain: Option<BlockchainConfigDraft>,
}
```

---

## 六、与 Gateway 的交互设计

### 6.1 前端独立生成配置（推荐短期方案）

由于 Gateway 不提供运行时配置修改 API，前端向导**独立生成配置文件**，由用户手动部署：

```
Web Wizard → 生成 beebotos.toml → 用户下载 → 上传到服务器 → 重启 Gateway
```

**优点：**
- 无需修改 Gateway 架构
- 配置变更可控，避免运行时配置漂移
- 与现有 CLI 向导输出格式一致

**缺点：**
- 无法一键应用，需要用户手动上传和重启

### 6.2 后端辅助接口（可选增强）

为提升体验，Gateway 可提供以下**只读/辅助**接口：

```rust
// GET /api/v1/admin/config/current
// 返回当前生效配置（脱敏），用于 "重新配置" 模式下的预填充

// POST /api/v1/admin/config/validate
// 接收配置 JSON，返回校验结果（字段合法性、依赖关系检查）

// GET /api/v1/admin/config/templates
// 返回预设模板列表（minimal/standard/enterprise）
```

### 6.3 长期方案：配置热重载

未来 Gateway 架构升级时，可支持：

```rust
// POST /api/v1/admin/config/apply
// 接收完整配置，校验后写入 TOML，触发 SIGHUP 热重载
// 返回：是否需要重启、哪些服务将受影响
```

---

## 七、权限与安全

### 7.1 访问控制

| 场景 | 路由 | 权限 | 说明 |
|------|------|------|------|
| 首次部署 | `/setup` | 公开 或 Basic Auth | 无 JWT 服务时的初始化 |
| 重新配置 | `/settings/wizard` | `admin` | 已有系统下的配置变更 |
| 查看配置 | `/settings/config` | `user` / `admin` | 只读查看当前配置 |

### 7.2 敏感信息处理

| 字段 | 前端展示 | 传输 | 存储 |
|------|---------|------|------|
| API Key | `sk-****...****` | HTTPS | 不存储（仅生成文件时存在于内存） |
| JWT Secret | `****` | HTTPS | 不存储 |
| 钱包助记词 | `****` | HTTPS | 不存储 |
| Database URL | 脱敏密码 | HTTPS | 不存储 |

### 7.3 配置预览安全

- TOML/ENV 预览在客户端生成，不发送到服务器
- 导出文件下载通过 Blob URL 实现，不留痕迹

---

## 八、实施清单

### Phase 1: 核心向导（MVP）

- [ ] 创建 `pages/setup.rs` 主页面框架
- [ ] 实现 `WizardStepper` 和 `WizardNavigation` 组件
- [ ] Step 1: Welcome（模式选择 + 模板加载）
- [ ] Step 2: Server（基础表单）
- [ ] Step 3: Database（含类型切换）
- [ ] Step 4: JWT & Security（含 Secret 生成器）
- [ ] Step 5: LLM Models（Provider 动态增删）
- [ ] Step 9: Review（TOML 预览 + 校验）
- [ ] Step 10: Deploy（TOML 下载 + ENV 导出）
- [ ] 注册路由 `/setup` 和 `/settings/wizard`
- [ ] 新增 `WizardState` 到全局状态管理

### Phase 2: 完整覆盖

- [ ] Step 6: Channels（14 个平台配置表单）
- [ ] Step 7: Blockchain（可选模块）
- [ ] Step 8: Logging & Observability
- [ ] Docker Compose 导出
- [ ] Kubernetes ConfigMap 导出
- [ ] 从 `.env` / `.toml` 文件导入
- [ ] 配置变更对比（Diff 视图）

### Phase 3: 增强体验

- [ ] Gateway 新增 `/api/v1/admin/config/current` 只读端点
- [ ] Gateway 新增 `/api/v1/admin/config/validate` 校验端点
- [ ] 前端支持 "重新配置" 时预填充当前配置
- [ ] 配置历史版本管理（结合 Gateway 自动备份）
- [ ] 一键部署按钮（配合 Gateway 热重载）

---

## 九、与现有 CLI 向导的兼容性

本 Web 向导与 `config_wizard.rs` 保持以下兼容：

1. **配置结构一致**：生成的 `beebotos.toml` 与 CLI 向导 100% 兼容
2. **模板一致**：Minimal / Standard / Enterprise 三个预设与 CLI 对齐
3. **导出格式一致**：`.env`、`docker-compose`、`k8s` 导出格式相同
4. **字段校验一致**：复用 Gateway 侧的校验逻辑（建议通过 API 调用）

CLI 向导可在服务器本地无 Web UI 环境下继续使用，两者互为补充。

---

## 十、附录：与 LLM 配置页面的关系

| 功能 | LLM Config Page (`/settings/llm`) | Config Wizard (`/setup` 或 `/settings/wizard`) |
|------|-----------------------------------|-----------------------------------------------|
| **定位** | 运行时监控 + Agent 级模型选择辅助 | 系统初始化 + 全局静态配置生成 |
| **数据来源** | `GET /api/v1/llm/metrics` | 用户输入 + 模板预设 |
| **修改能力** | 只读（展示当前生效配置） | 读写（生成新配置文件） |
| **使用场景** | 日常监控、Agent 创建时参考 | 首次部署、大规模配置变更 |
| **输出** | 无（纯展示） | `beebotos.toml` / `.env` / Docker / K8s |

两个页面互补：**Config Wizard 用于生成配置，LLM Config Page 用于监控配置运行效果。**
