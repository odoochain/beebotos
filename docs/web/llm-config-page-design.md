# BeeBotOS Web — LLM 大模型配置页面设计文档

> 基于 `apps/gateway` 接口定义的 Web 端 LLM 全局配置页面方案  
> 版本：v1.0  
> 生成时间：2026-04-19

---

## 一、设计背景

### 1.1 现状问题

当前 `apps/web` 前端仅在**创建 Agent 时**支持选择模型 Provider 和 Model Name，属于**Agent 级别**的模型绑定。系统缺乏以下能力：

- 无法查看 Gateway 当前加载的全局 LLM 配置
- 无法监控各 Provider 的健康状态和性能指标
- 无法查看 Token 消耗和延迟趋势
- 无法了解 Fallback Chain 的生效情况

### 1.2 Gateway 能力支撑

Gateway 已提供以下运行时只读接口：

| 接口 | 路径 | 能力 |
|------|------|------|
| LLM Metrics | `GET /api/v1/llm/metrics` | 请求量、Token 消耗、P50/P95/P99 延迟、Provider 健康状态 |
| LLM Health | `GET /api/v1/llm/health` | Provider 健康汇总、连续失败次数 |
| Agent 创建 | `POST /api/v1/agents` | 支持 `model_provider` + `model_name` 字段 |

> **约束：** Gateway 的 LLM 全局配置为静态 TOML 驱动，**不支持运行时修改**。因此前端配置页面以**只读监控 + Agent 级选择辅助**为主。

---

## 二、页面定位

```
路由建议：/settings/llm  （作为 Settings 的子页面）
或独立路由：/llm-config

入口位置：
  1. Settings 页面新增 "LLM Configuration" 卡片入口
  2. 顶部导航或侧边栏增加 "AI Models" 快捷入口
  3. Agent 创建弹窗中模型选择区域增加 "查看系统模型配置" 链接
```

---

## 三、页面结构设计

### 3.1 整体布局

```
+-------------------------------------------------------------+
|  LLM Configuration                                    [?]   |
+-------------------------------------------------------------+
|                                                             |
|  +-------------------+  +-------------------+              |
|  | Default Provider  |  | Fallback Chain    |              |
|  |   [Kimi]          |  |  kimi → openai    |              |
|  +-------------------+  +-------------------+              |
|                                                             |
|  +-----------------------------------------------------+   |
|  | Provider Health Status                               |   |
|  | +--------+ +--------+ +--------+ +--------+          |   |
|  | |  Kimi  | | OpenAI | | DeepS  | | Ollama |          |   |
|  | |   🟢   | |   🟢   | |   🟡   | |   🔴   |          |   |
|  | | 12ms   | | 89ms   | | 156ms  | | N/A    |          |   |
|  | +--------+ +--------+ +--------+ +--------+          |   |
|  +-----------------------------------------------------+   |
|                                                             |
|  +------------------------+  +------------------------+    |
|  | Request Statistics     |  | Token Consumption      |    |
|  | [Line Chart: 7d]       |  | [Pie Chart]            |    |
|  | Total: 12,453          |  | Input: 2.1M            |    |
|  | Success: 98.2%         |  | Output: 890K           |    |
|  +------------------------+  +------------------------+    |
|                                                             |
|  +-----------------------------------------------------+   |
|  | Latency Percentiles (ms)                             |   |
|  | P50: 45  |  P95: 180  |  P99: 420  |  Avg: 67        |   |
|  +-----------------------------------------------------+   |
|                                                             |
|  +-----------------------------------------------------+   |
|  | Provider Details                                     |   |
|  | Kimi                                                 |   |
|  |   Model: moonshot-v1-8k  |  Temperature: 0.7          |   |
|  |   Base URL: https://api.moonshot.cn                  |   |
|  |   API Key: sk-****...**** (masked)                   |   |
|  |   Context Window: 8192  |  Max Tokens: 4096           |   |
|  +-----------------------------------------------------+   |
|                                                             |
+-------------------------------------------------------------+
```

### 3.2 模块详细设计

#### 模块 A：全局配置概览（GlobalConfigOverview）

**数据需求：**
- 默认 Provider（来自 Gateway 启动配置，需 Gateway 新增只读接口或从 Agent 创建推断）
- Fallback Chain（同上）

**UI 组件：**
```rust
#[component]
fn GlobalConfigOverview(
    default_provider: Signal<String>,
    fallback_chain: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <div class="config-overview-grid">
            <Card title="Default Provider">
                <ProviderBadge name=default_provider />
            </Card>
            <Card title="Fallback Chain">
                <div class="fallback-chain">
                    {fallback_chain.get().into_iter().map(|name| view! {
                        <span class="fallback-item">{name}</span>
                        <span class="fallback-arrow">"→"</span>
                    }).collect::<Vec<_>>()}
                    <span class="fallback-end">"∞"</span>
                </div>
            </Card>
        </div>
    }
}
```

> **后端依赖：** 需要 Gateway 暴露当前生效的 `default_provider` 和 `fallback_chain`。建议新增 `GET /api/v1/llm/config` 只读端点。

---

#### 模块 B：Provider 健康卡片（ProviderHealthCards）

**数据需求：** `GET /api/v1/llm/health` + `GET /api/v1/llm/metrics`

**状态定义：**
| 状态 | 图标 | 条件 |
|------|------|------|
| Healthy | 🟢 | `healthy == true && consecutive_failures == 0` |
| Degraded | 🟡 | `healthy == true && consecutive_failures > 0` |
| Unhealthy | 🔴 | `healthy == false` |
| Unknown | ⚪ | 无数据 |

**UI 组件：**
```rust
#[component]
fn ProviderHealthCards(providers: Signal<Vec<ProviderHealth>>) -> impl IntoView {
    view! {
        <div class="provider-health-grid">
            {providers.get().into_iter().map(|p| view! {
                <div class={format!("provider-card status-{}", p.status)}>
                    <div class="provider-header">
                        <span class="provider-name">{p.name}</span>
                        <span class="provider-status-icon">{p.status_icon}</span>
                    </div>
                    <div class="provider-metrics">
                        <span class="latency">{format!("{} ms", p.avg_latency_ms)}</span>
                        <span class="success-rate">{format!("{:.1}%", p.success_rate)}</span>
                    </div>
                    {if p.consecutive_failures > 0 {
                        view! {
                            <span class="failure-badge">
                                {format!("{} consecutive failures", p.consecutive_failures)}
                            </span>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    }}
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}
```

---

#### 模块 C：请求统计图表（RequestStatisticsChart）

**数据需求：** `GET /api/v1/llm/metrics`

**展示内容：**
- 近 7 天/24 小时请求量趋势（折线图）
- 成功率趋势
- 总请求数、成功请求数、失败请求数

**前端实现：** 由于 WASM 环境限制，建议使用纯 CSS/SVG 实现的轻量级图表，或引入 `plotters` WASM 版本。

---

#### 模块 D：Token 消耗面板（TokenConsumptionPanel）

**数据需求：** `GET /api/v1/llm/metrics`

**展示内容：**
```
Total Tokens: 3,245,678
├─ Input Tokens:  2,100,000  ████████████████ 65%
└─ Output Tokens:   890,000  ████████ 27%
    (Other: 255,678)

按 Provider 分布：
Kimi    ████████████████████  65%
OpenAI  ████████              25%
Ollama  ██                     8%
Others  █                      2%
```

---

#### 模块 E：延迟分位值（LatencyPercentiles）

**数据需求：** `GET /api/v1/llm/metrics`

**展示内容：**
| 指标 | 数值 | 颜色 |
|------|------|------|
| P50 | 45 ms | 🟢 |
| P95 | 180 ms | 🟡 |
| P99 | 420 ms | 🟡 |
| Average | 67 ms | 🟢 |

---

#### 模块 F：Provider 详情（ProviderDetails）

**数据需求：**
- Gateway 静态配置（需新增只读接口）
- 敏感字段（API Key）脱敏展示

**展示字段：**
| 字段 | 来源 | 展示方式 |
|------|------|---------|
| Provider Name | metrics | 文本 |
| Model | config | 文本 |
| Base URL | config | 文本（可点击跳转） |
| API Key | config | `sk-****...****` 脱敏 |
| Temperature | config | 文本 |
| Max Tokens | config | 文本 |
| Context Window | config | 文本 |
| Request Timeout | config | 文本 |

---

## 四、前端数据模型

### 4.1 新增 Service

```rust
// apps/web/src/api/services.rs

/// LLM Configuration Service
#[derive(Clone)]
pub struct LlmConfigService {
    client: ApiClient,
}

impl LlmConfigService {
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    /// Get LLM metrics (request stats, latency, token usage, provider health)
    pub async fn get_metrics(&self) -> Result<LlmMetrics, ApiError> {
        self.client.get("/llm/metrics").await
    }

    /// Get LLM health summary
    pub async fn get_health(&self) -> Result<LlmHealth, ApiError> {
        self.client.get("/llm/health").await
    }

    /// Get current global LLM configuration (requires Gateway to expose)
    pub async fn get_global_config(&self) -> Result<LlmGlobalConfig, ApiError> {
        self.client.get("/llm/config").await
    }
}
```

### 4.2 新增数据模型

```rust
// LLM Metrics Response (aligns with Gateway handler)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmMetrics {
    pub summary: LlmSummary,
    pub tokens: TokenStats,
    pub latency: LatencyStats,
    pub providers: Vec<ProviderMetric>,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmSummary {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate_percent: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenStats {
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LatencyStats {
    pub average_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderMetric {
    pub name: String,
    pub healthy: bool,
    pub consecutive_failures: u32,
    pub avg_latency_ms: Option<u64>,
    pub success_rate: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmHealth {
    pub status: String,
    pub providers: ProviderHealthSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderHealthSummary {
    pub total: usize,
    pub healthy: usize,
    pub details: Vec<ProviderHealthDetail>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderHealthDetail {
    pub name: String,
    pub healthy: bool,
    pub consecutive_failures: u32,
}

/// Global LLM Configuration (requires Gateway endpoint)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmGlobalConfig {
    pub default_provider: String,
    pub fallback_chain: Vec<String>,
    pub cost_optimization: bool,
    pub max_tokens: u32,
    pub system_prompt: String,
    pub request_timeout: u64,
    pub providers: Vec<ProviderConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_key_masked: String,  // e.g. "sk-****...****"
    pub model: String,
    pub base_url: String,
    pub temperature: f32,
    pub context_window: Option<u32>,
}
```

### 4.3 AppState 扩展

```rust
// apps/web/src/state/app.rs

impl AppState {
    pub fn llm_config_service(&self) -> LlmConfigService {
        LlmConfigService::new(self.api_client())
    }
}
```

---

## 五、Gateway 侧依赖

### 5.1 已存在的端点（无需修改）

- `GET /api/v1/llm/metrics`
- `GET /api/v1/llm/health`

### 5.2 建议新增的端点

```rust
// apps/gateway/src/handlers/http/llm_config.rs (新增文件)

/// Get current global LLM configuration (read-only, sensitive fields masked)
pub async fn get_llm_global_config(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<LlmGlobalConfigResponse>, GatewayError> {
    require_any_role(&user, &["user", "admin"])?;
    
    let config = &state.config.models;
    
    let providers = config.providers.iter().map(|(name, provider)| {
        ProviderConfigResponse {
            name: name.clone(),
            api_key_masked: mask_api_key(&provider.api_key),
            model: provider.model.clone(),
            base_url: provider.base_url.clone(),
            temperature: provider.temperature,
            context_window: provider.context_window,
        }
    }).collect();
    
    Ok(Json(LlmGlobalConfigResponse {
        default_provider: config.default_provider.clone(),
        fallback_chain: config.fallback_chain.clone(),
        cost_optimization: config.cost_optimization,
        max_tokens: config.max_tokens,
        system_prompt: config.system_prompt.clone(),
        request_timeout: config.request_timeout,
        providers,
    }))
}

fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        "****".to_string()
    } else {
        format!("{}****{}", &key[..4], &key[key.len()-4..])
    }
}
```

**路由注册：**
```rust
// apps/gateway/src/main.rs create_router
.route("/api/v1/llm/config", get(handlers::http::llm_config::get_llm_global_config))
```

---

## 六、与 Agent 创建流程的联动

### 6.1 Agent 创建弹窗增强

在 `CreateAgentModal` 的模型选择区域，增加 "查看系统配置" 链接：

```rust
view! {
    <div class="form-group">
        <label>"Model Provider"</label>
        <select prop:value=model_provider on:change=...>
            <option value="openai">"OpenAI"</option>
            ...
        </select>
        <A href="/settings/llm" attr:class="form-help-link">
            "View system LLM configuration →"
        </A>
    </div>
}
```

### 6.2 模型选择自动补全

基于 `GET /api/v1/llm/config` 返回的 Provider 列表，为 Model Name 输入框提供自动补全：

```rust
// 当选择 Provider 后，自动填充默认 Model Name
let provider_models = Signal::derive(move || {
    match model_provider.get().as_str() {
        "kimi" => "moonshot-v1-8k",
        "openai" => "gpt-4",
        "anthropic" => "claude-3-sonnet",
        "deepseek" => "deepseek-chat",
        "zhipu" => "glm-4",
        "ollama" => "llama2",
        _ => "gpt-4",
    }
});
```

---

## 七、权限控制

| 功能 | 所需角色 | 说明 |
|------|---------|------|
| 查看 LLM 配置 | `user` / `admin` | 所有认证用户可见 |
| 查看 API Key（脱敏） | `user` / `admin` | 始终脱敏展示 |
| 查看完整 API Key | `admin` | 如需支持，需额外加密传输 |

---

## 八、实施清单

### 前端（apps/web）

- [ ] 新增 `LlmConfigService` 及数据模型
- [ ] 新增 `pages/llm_config.rs` 页面组件
- [ ] 新增子组件：`GlobalConfigOverview`、`ProviderHealthCards`、`RequestStatisticsChart`、`TokenConsumptionPanel`、`LatencyPercentiles`、`ProviderDetails`
- [ ] 在 `lib.rs` 路由中注册 `/settings/llm`
- [ ] 在 `SettingsPage` 中增加 LLM 配置入口卡片
- [ ] 在 `CreateAgentModal` 中增加 "查看系统配置" 链接
- [ ] 更新 `state/app.rs` 提供 `llm_config_service()` 工厂方法
- [ ] 更新 `api/mod.rs` 导出新增类型

### 后端（apps/gateway）

- [ ] 新增 `handlers/http/llm_config.rs`
- [ ] 实现 `GET /api/v1/llm/config` 只读端点
- [ ] 实现 API Key 脱敏工具函数
- [ ] 在 `create_router` 中注册新路由
- [ ] 确保 `/api/v1/llm/metrics` 和 `/api/v1/llm/health` 返回的数据结构稳定

---

## 九、附录：与 Gateway 配置体系的兼容性说明

由于 Gateway 的 LLM 配置为**静态 TOML + 环境变量**驱动，本设计遵循以下原则：

1. **只读优先**：前端页面以展示和监控为主，不直接修改全局配置
2. **Agent 级覆盖**：如需更换模型，通过 Agent 的 `model_provider` / `model_name` 字段实现
3. **配置变更引导**：如需修改全局配置，页面提供 "如何修改配置" 的文档链接，引导用户编辑 TOML 并重启 Gateway
4. **未来扩展**：Gateway 后续如支持配置热重载或管理 API，前端可平滑扩展为读写模式
