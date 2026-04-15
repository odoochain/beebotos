# BeeBotOS-Web 工作原理与代码逻辑分析文档

## 目录
1. [架构概述](#一架构概述)
2. [核心模块详解](#二核心模块详解)
3. [数据流与状态管理](#三数据流与状态管理)
4. [API 层设计](#四api-层设计)
5. [组件系统](#五组件系统)
6. [与其他模块的关系](#六与其他模块的关系)
7. [函数接口关系图](#七函数接口关系图)
8. [安全机制](#八安全机制)
9. [性能优化策略](#九性能优化策略)

---

## 一、架构概述

### 1.1 技术栈

BeeBotOS-Web 是基于 **Leptos** 框架构建的 **WebAssembly (WASM)** 前端应用：

| 技术 | 用途 | 版本 |
|------|------|------|
| Leptos | Rust 响应式前端框架 | 0.8.6 |
| WASM | 浏览器执行环境 | wasm32-unknown-unknown |
| leptos_router | 客户端路由 | 0.8.6 |
| gloo-net | HTTP 客户端 | 0.5 |
| gloo-storage | 本地存储 | 0.3 |
| chrono | 日期时间处理 | 0.4 |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │   Pages     │ │ Components  │ │      Guards         │    │
│  │  (pages/)   │ │(components/)│ │   (guard.rs)        │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                     State Layer                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │  AuthState  │ │  AgentState │ │  NotificationState  │    │
│  │   (auth)    │ │   (agent)   │ │  (notification)     │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
│  ┌─────────────┐ ┌─────────────┐                            │
│  │   DaoState  │ │   AppState  │                            │
│  │    (dao)    │ │    (app)    │                            │
│  └─────────────┘ └─────────────┘                            │
├─────────────────────────────────────────────────────────────┤
│                      API Layer                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │ ApiClient   │ │  Services   │ │   Error Handling    │    │
│  │(client.rs)  │ │(services.rs)│ │    (error/)         │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                    Utility Layer                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │ Validation  │ │    Theme    │ │      Security       │    │
│  │(validation) │ │   (theme)   │ │    (security)       │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 应用入口流程

```rust
// main.rs - 应用入口
fn main() {
    leptos::mount::mount_to_body(App);  // 挂载到 body
}

// lib.rs - 应用根组件
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();      // 1. 提供 Meta 上下文
    provide_app_state();         // 2. 提供全局状态
    provide_theme();             // 3. 提供主题上下文
    
    view! {
        <ContentSecurityPolicy />   // CSP 安全头
        <GlobalErrorHandler>        // 全局错误边界
            <Router>                // 路由系统
                <Nav />             // 导航组件
                <main>
                    <Routes fallback=|| view! { <NotFound/> }>
                        // 路由配置...
                    </Routes>
                </main>
                <Footer />
            </Router>
        </GlobalErrorHandler>
    }
}
```

---

## 二、核心模块详解

### 2.1 状态管理模块 (state/)

#### 2.1.1 架构设计

采用**领域驱动**的状态拆分策略，避免单一状态对象导致的过度重渲染：

```
┌──────────────────────────────────────────────────────────────┐
│                        AppState                               │
│                     (组合根状态)                               │
├──────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │
│  │AuthState │  │AgentState│  │ DaoState │  │Notification  │  │
│  │          │  │          │  │          │  │    State     │  │
│  │ • user   │  │ • agents │  │• proposals│  │ • notifications│ │
│  │ • token  │  │ • filters│  │ • votes  │  │ • unread_count│  │
│  │ • roles  │  │ • pagination│ │• summary │  │              │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

#### 2.1.2 AuthState - 认证状态

```rust
/// 认证状态结构
pub struct AuthState {
    pub user: RwSignal<Option<User>>,           // 用户信息
    pub token: RwSignal<Option<String>>,        // 访问令牌
    pub refresh_token: RwSignal<Option<String>>, // 刷新令牌
    pub token_expires_at: RwSignal<Option<i64>>, // 过期时间
    pub is_loading: RwSignal<bool>,             // 加载状态
    pub error: RwSignal<Option<AuthError>>,     // 错误状态
}

/// 核心方法
impl AuthState {
    // 检查认证状态
    pub fn is_authenticated(&self) -> bool;
    
    // 检查 Token 是否过期
    pub fn is_token_expired(&self) -> bool;
    
    // RBAC 权限检查
    pub fn has_role(&self, role: &Role) -> bool;
    pub fn has_permission(&self, permission: &Permission) -> bool;
    
    // 登录/登出
    pub fn set_authenticated(&self, user: User, token: String, ...);
    pub fn logout(&self);
    
    // Token 自动刷新
    pub fn should_refresh_token(&self) -> bool;
    async fn perform_token_refresh(&self);
}
```

**Token 刷新机制：**

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   用户登录   │────▶│ set_authenticated │────▶│ start_token_    │
└─────────────┘     └──────────────────┘     │ refresh_interval │
                                             └────────┬────────┘
                                                      │
                                                      ▼
                              ┌───────────────────────────────────┐
                              │  Interval (每 60 秒检查一次)       │
                              │  • 检查 token 是否即将过期         │
                              │  • 如果是，调用 refresh API       │
                              └───────────────────────────────────┘
```

#### 2.1.3 AgentState - Agent 管理状态

```rust
pub struct AgentState {
    pub agents: RwSignal<Vec<AgentInfo>>,      // Agent 列表
    pub filters: RwSignal<AgentFilters>,        // 过滤器
    pub pagination: RwSignal<AgentPagination>,  // 分页状态
    pub sort_by: RwSignal<(AgentSortBy, SortOrder)>, // 排序
    pub selected_agent: RwSignal<Option<String>>, // 选中的 Agent
    pub is_list_loading: RwSignal<bool>,        // 列表加载状态
    pub is_detail_loading: RwSignal<bool>,      // 详情加载状态
}
```

#### 2.1.4 状态提供与消费

```rust
// 在应用初始化时提供状态
pub fn provide_app_state() {
    // 提供各个领域状态
    provide_auth_state();
    provide_agent_state();
    provide_dao_state();
    provide_notification_state();
    
    // 提供组合状态
    provide_context(AppState::new());
    
    // 设置在线状态监控
    setup_online_status_monitoring();
}

// 在组件中使用状态
#[component]
fn MyComponent() -> impl IntoView {
    let app_state = use_app_state();      // 使用组合状态
    let auth = use_auth_state();          // 或直接使用领域状态
    let agents = use_agent_state();
    
    // 读取状态
    let is_logged_in = auth.is_authenticated();
    
    // 更新状态
    auth.user.set(Some(new_user));
}
```

### 2.2 API 客户端模块 (api/)

#### 2.2.1 ApiClient 架构

```rust
/// API 客户端结构
pub struct ApiClient {
    config: ClientConfig,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,      // 请求缓存
    in_flight: Arc<RwLock<HashMap<String, InFlightRequest>>>, // 请求去重
    interceptors: Vec<Box<dyn RequestInterceptor>>,
}

/// 核心功能
impl ApiClient {
    // HTTP 方法
    pub async fn get<T>(&self, path: &str) -> Result<T, ApiError>;
    pub async fn post<T, B>(&self, path: &str, body: &B) -> Result<T, ApiError>;
    pub async fn put<T, B>(&self, path: &str, body: &B) -> Result<T, ApiError>;
    pub async fn delete<T>(&self, path: &str) -> Result<T, ApiError>;
    
    // 缓存管理
    pub fn invalidate_cache(&self, key: &str);
    
    // Token 刷新
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenRefreshResponse, ApiError>;
}
```

#### 2.2.2 请求处理流程

```
┌──────────────┐
│   发起请求    │
└──────┬───────┘
       │
       ▼
┌────────────────────────────────────────────┐
│           请求拦截器链                      │
│  • 添加认证头 (Authorization: Bearer xxx)   │
│  • 添加 CSRF Token (POST/PUT/DELETE)        │
│  • 添加 Origin 头                          │
└────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│           缓存检查                          │
│  • 检查是否有有效缓存                        │
│  • 有：直接返回缓存数据                      │
│  • 无：继续发送请求                          │
└────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│           请求去重检查                       │
│  • 检查相同请求是否正在进行                   │
│  • 是：等待该请求完成                        │
│  • 否：发送新请求                            │
└────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│           发送 HTTP 请求                     │
│  • 使用 gloo-net 发送请求                    │
│  • 应用超时控制                              │
└────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│           重试机制 (如失败)                  │
│  • 指数退避重试                              │
│  • 最大重试次数：3 次                        │
└────────────────────────────────────────────┘
       │
       ▼
┌────────────────────────────────────────────┐
│           响应处理                           │
│  • 解析 JSON 响应                            │
│  • 缓存响应数据                              │
│  • 返回结果                                  │
└────────────────────────────────────────────┘
```

#### 2.2.3 Service 层封装

```rust
/// Agent 服务封装
pub struct AgentService {
    client: ApiClient,
}

impl AgentService {
    pub async fn list(&self) -> Result<Vec<AgentInfo>, ApiError> {
        self.client.get("/agents").await
    }
    
    pub async fn get(&self, id: &str) -> Result<AgentInfo, ApiError> {
        self.client.get(&format!("/agents/{}", id)).await
    }
    
    pub async fn create(&self, req: CreateAgentRequest) -> Result<AgentInfo, ApiError> {
        self.client.post("/agents", &req).await
    }
    
    pub async fn start(&self, id: &str) -> Result<(), ApiError> {
        self.client.post(&format!("/agents/{}/start", id), &json!({})).await
    }
    
    pub async fn stop(&self, id: &str) -> Result<(), ApiError> {
        self.client.post(&format!("/agents/{}/stop", id), &json!({})).await
    }
}
```

### 2.3 路由守卫系统 (components/guard.rs)

#### 2.3.1 守卫组件层次

```
┌───────────────────────────────────────────────────────────────┐
│                        AuthGuard                               │
│                   (基础认证检查)                                │
│  • 检查是否已登录                                              │
│  • 未登录：保存当前路径，重定向到登录页                          │
└───────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐
│   RoleGuard     │  │ PermissionGuard │  │   CombinedGuard     │
│  (角色检查)      │  │   (权限检查)     │  │   (组合检查)        │
│ • Admin         │  │ • AgentCreate   │  │  认证 + 角色 + 权限   │
│ • Operator      │  │ • DaoVote       │  │                     │
│ • Member        │  │ • TreasuryView  │  │                     │
└─────────────────┘  └─────────────────┘  └─────────────────────┘
```

#### 2.3.2 守卫实现原理

```rust
#[component]
pub fn AuthGuard(children: ChildrenFn) -> impl IntoView {
    let auth = use_auth_state();
    let navigate = use_navigate();
    
    // 使用 Memo 缓存认证状态检查
    let is_authenticated = Memo::new(move |_| auth.is_authenticated());
    
    // Effect 用于副作用（重定向）
    Effect::new(move |_| {
        if !auth.is_authenticated() {
            // 保存当前路径用于登录后重定向
            save_redirect_path();
            navigate("/login", Default::default());
        }
    });
    
    // 条件渲染
    move || {
        if is_authenticated.get() {
            children.clone()().into_any()
        } else {
            view! { <Redirecting message="Checking authentication..." /> }.into_any()
        }
    }
}
```

---

## 三、数据流与状态管理

### 3.1 数据流架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                           数据流图                                   │
└─────────────────────────────────────────────────────────────────────┘

   用户操作
      │
      ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   页面组件   │────▶│  状态更新   │────▶│   UI 重渲染  │
│  (Pages)    │     │  (State)    │     │  (Leptos)   │
└─────────────┘     └─────────────┘     └─────────────┘
      │
      ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   API 服务   │────▶│  后端 API   │────▶│  区块链/    │
│ (Services)  │     │  (Gateway)  │     │  存储层     │
└─────────────┘     └─────────────┘     └─────────────┘
      │
      ▼
┌─────────────┐
│  缓存更新    │
│  (Cache)    │
└─────────────┘
```

### 3.2 页面数据获取模式

```rust
// 以 Agents 页面为例
#[component]
pub fn AgentsPage() -> impl IntoView {
    let app_state = use_app_state();
    let pagination = RwSignal::new(PaginationState::new(PAGE_SIZE));
    
    // 使用 RwSignal 存储数据（CSR 模式）
    let agents_data: RwSignal<Option<(Vec<AgentInfo>, usize)>> = RwSignal::new(None);
    let is_loading = RwSignal::new(false);
    
    // 数据获取函数
    let fetch_agents = move || {
        is_loading.set(true);
        let client = app_state.api_client();
        let page = pagination.get().current_page;
        
        spawn_local(async move {
            match fetch_agents_paginated(&client, page, PAGE_SIZE).await {
                Ok((data, total)) => {
                    pagination.update(|p| p.set_total(total));
                    agents_data.set(Some((data, total)));
                    is_loading.set(false);
                }
                Err(e) => {
                    agents_error.set(Some(e));
                    is_loading.set(false);
                }
            }
        });
    };
    
    // 初始加载
    fetch_agents();
    
    view! {
        // 根据状态条件渲染
        {move || {
            if is_loading.get() {
                view! { <AgentsLoading/> }.into_any()
            } else if let Some((data, _)) = agents_data.get() {
                view! { <AgentsList agents=data .../> }.into_any()
            } else {
                view! { <AgentsError .../> }.into_any()
            }
        }}
    }
}
```

### 3.3 状态更新流程

```rust
// 1. 用户触发操作
<button on:click=move |_| {
    let agent_id = agent_id.clone();
    let on_status_change = on_status_change.clone();
    is_starting.set(true);
    
    // 2. 异步调用 API
    spawn_local(async move {
        let service = app_state.agent_service();
        match service.start(&agent_id).await {
            Ok(_) => {
                // 3. 成功：更新通知状态
                app_state.notify(
                    NotificationType::Success,
                    "Agent Started",
                    format!("Agent {} has been started", agent_id)
                );
                // 4. 触发刷新回调
                if let Some(ref cb) = on_status_change {
                    cb();
                }
            }
            Err(e) => {
                // 5. 失败：显示错误通知
                app_state.notify(
                    NotificationType::Error,
                    "Start Failed",
                    format!("Failed to start agent: {}", e)
                );
            }
        }
        is_starting.set(false);
    });
}>
```

---

## 四、API 层设计

### 4.1 API 端点映射

| 功能模块 | HTTP 方法 | 端点 | 对应 Service 方法 |
|----------|-----------|------|-------------------|
| **Agent** | | | `AgentService` |
| 列表 | GET | `/api/v1/agents` | `list()` |
| 详情 | GET | `/api/v1/agents/:id` | `get(id)` |
| 创建 | POST | `/api/v1/agents` | `create(req)` |
| 更新 | PUT | `/api/v1/agents/:id` | `update(id, req)` |
| 删除 | DELETE | `/api/v1/agents/:id` | `delete(id)` |
| 启动 | POST | `/api/v1/agents/:id/start` | `start(id)` |
| 停止 | POST | `/api/v1/agents/:id/stop` | `stop(id)` |
| **DAO** | | | `DaoService` |
| 摘要 | GET | `/api/v1/dao/summary` | `get_summary()` |
| 提案列表 | GET | `/api/v1/dao/proposals` | `list_proposals()` |
| 投票 | POST | `/api/v1/dao/proposals/:id/vote` | `vote(id, ...)` |
| **Skills** | | | `SkillService` |
| 列表 | GET | `/api/v1/skills` | `list()` |
| 安装 | POST | `/api/v1/skills/:id/install` | `install(id)` |
| **Treasury** | | | `TreasuryService` |
| 信息 | GET | `/api/v1/treasury` | `get_info()` |
| **Auth** | | | `AuthService` |
| 登录 | POST | `/api/v1/auth/login` | `login(...)` |
| 登出 | POST | `/api/v1/auth/logout` | `logout()` |
| 刷新 | POST | `/api/v1/auth/refresh` | `refresh_token(...)` |

### 4.2 错误处理机制

```rust
/// API 错误类型
#[derive(Clone, Debug)]
pub enum ApiError {
    Network(String),        // 网络错误 (1000-1999)
    ServerError(u16),       // 服务端错误 (2000-2999)
    Unauthorized,           // 认证错误 (3000-3999)
    ClientError(u16),       // 客户端错误 (4000-4999)
    Serialization(String),  // 序列化错误 (5000-5999)
    Timeout,                // 超时
}

impl ApiError {
    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(self, 
            ApiError::Network(_) | 
            ApiError::ServerError(500..=599) |
            ApiError::Timeout
        )
    }
}
```

---

## 五、组件系统

### 5.1 组件层次结构

```
App (根组件)
├── ContentSecurityPolicy (安全头)
├── GlobalErrorHandler (错误边界)
├── Router (路由)
│   ├── Nav (导航栏)
│   ├── Routes
│   │   ├── Home (首页)
│   │   │   ├── DashboardStats
│   │   │   ├── FeatureCard
│   │   │   └── QuickActionCard
│   │   ├── AuthGuard
│   │   │   ├── AgentsPage
│   │   │   │   ├── AgentsList
│   │   │   │   │   └── AgentCard
│   │   │   │   ├── Pagination
│   │   │   │   └── CreateAgentModal
│   │   │   ├── AgentDetail
│   │   │   ├── DaoPage
│   │   │   ├── TreasuryPage
│   │   │   ├── SkillsPage
│   │   │   └── SettingsPage
│   │   └── NotFound
│   └── Footer
```

### 5.2 关键组件实现

#### 5.2.1 AgentCard 组件

```rust
#[component]
fn AgentCard(
    #[prop(into)] agent: AgentInfo,
    on_delete: impl Fn(String) + 'static,
    #[prop(optional)] on_status_change: Option<impl Fn() + Clone + 'static>,
) -> impl IntoView {
    // 状态信号
    let app_state = use_app_state();
    let is_starting = RwSignal::new(false);
    let is_stopping = RwSignal::new(false);
    
    // 根据状态计算样式
    let status_class = match agent.status {
        AgentStatus::Running => "status-running",
        AgentStatus::Stopped | AgentStatus::Idle => "status-idle",
        AgentStatus::Error => "status-error",
        AgentStatus::Pending => "status-pending",
    };
    
    view! {
        <div class="card agent-card">
            <div class="agent-header">
                <h3>{agent.name}</h3>
                <span class=format!("status-badge {}", status_class)>
                    {format!("{:?}", agent.status)}
                </span>
            </div>
            // ... 其他 UI 元素
        </div>
    }
}
```

#### 5.2.2 分页组件

```rust
#[component]
pub fn Pagination(
    state: RwSignal<PaginationState>,
    on_change: impl Fn(usize) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let total_pages = Memo::new(move |_| state.get().total_pages());
    let current_page = Memo::new(move |_| state.get().current_page);
    
    view! {
        <div class="pagination">
            <button 
                disabled=move || current_page.get() == 0
                on:click=move |_| {
                    state.update(|s| s.prev_page());
                    on_change(current_page.get());
                }
            >
                "Previous"
            </button>
            
            // 页码显示...
            
            <button 
                disabled=move || current_page.get() >= total_pages.get() - 1
                on:click=move |_| {
                    state.update(|s| s.next_page());
                    on_change(current_page.get());
                }
            >
                "Next"
            </button>
        </div>
    }
}
```

---

## 六、与其他模块的关系

### 6.1 与后端服务的关系

```
┌────────────────────────────────────────────────────────────────────┐
│                         BeeBotOS 系统架构                           │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│   ┌──────────────────┐         HTTP/WebSocket         ┌──────────┐ │
│   │   beebotos-web   │ ◀────────────────────────────▶ │  Gateway │ │
│   │   (WASM 前端)     │                               │  网关层   │ │
│   └──────────────────┘                               └────┬─────┘ │
│                                                            │       │
│                                                            ▼       │
│                                                    ┌──────────────┐ │
│                                                    │   Kernel     │ │
│                                                    │   核心层      │ │
│                                                    └──────┬───────┘ │
│                                                           │        │
│                              ┌────────────────────────────┼────┐   │
│                              │                            │    │   │
│                              ▼                            ▼    ▼   │
│                        ┌──────────┐                ┌──────────────┐ │
│                        │  Brain   │                │    Chain     │ │
│                        │ AI 模块  │                │  区块链集成   │ │
│                        └──────────┘                └──────────────┘ │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### 6.2 API 调用链路

```
beebotos-web
    │
    ├── AgentService.list() ───────────────┐
    │   └── GET /api/v1/agents             │
    │       │                              │
    │       ▼                              │
    │   beebotos-gateway                   │
    │       │                              │
    │       └──▶ beebotos-kernel           │
    │               │                      │
    │               └──▶ AgentManager      │
    │                       │              │
    │                       └──▶ 数据库/存储 │
    │                                      │
    ├── DaoService.vote() ─────────────────┤
    │   └── POST /api/v1/dao/proposals/... │
    │       │                              │
    │       ▼                              │
    │   beebotos-gateway                   │
    │       │                              │
    │       └──▶ beebotos-dao              │
    │               │                      │
    │               └──▶ 智能合约调用       │
    │                       │              │
    │                       └──▶ 区块链网络 │
    │                                      │
    └── TreasuryService.get_info() ────────┘
        └── GET /api/v1/treasury
            │
            ▼
        beebotos-gateway
            │
            └──▶ beebotos-chain
                    │
                    └──▶ 链上数据查询
```

### 6.3 数据模型映射

| Web 模型 | 后端对应 | 说明 |
|----------|----------|------|
| `AgentInfo` | `Agent` | Agent 基本信息 |
| `AgentStatus` | `AgentStatus` | 运行状态枚举 |
| `ProposalInfo` | `Proposal` | DAO 提案信息 |
| `TreasuryInfo` | `Treasury` | 金库资产信息 |
| `UserInfo` | `User` | 用户信息 |

---

## 七、函数接口关系图

### 7.1 核心函数调用链

```
main()
│
└── mount_to_body(App)
    │
    └── App()
        │
        ├── provide_app_state()
        │   ├── provide_auth_state()
        │   ├── provide_agent_state()
        │   ├── provide_dao_state()
        │   ├── provide_notification_state()
        │   └── setup_online_status_monitoring()
        │
        ├── provide_theme()
        │
        └── Router
            │
            ├── Nav()
            │   └── NavAuthSection()
            │       └── LogoutButton()
            │
            ├── Routes
            │   ├── Home()
            │   │   └── fetch_dashboard_stats()
            │   │       ├── AgentService.list()
            │   │       └── DaoService.get_summary()
            │   │
            │   ├── AuthGuard → AgentsPage()
            │   │   ├── fetch_agents_paginated()
            │   │   ├── AgentsList()
            │   │   │   └── AgentCard()
            │   │   └── CreateAgentModal()
            │   │
            │   ├── AuthGuard → AgentDetail()
            │   ├── AuthGuard → DaoPage()
            │   └── ...
            │
            └── Footer()
```

### 7.2 API Service 函数关系

```
ApiClient
│
├── get(path) ───────────────┐
├── post(path, body) ────────┼──▶ RequestBuilder
├── put(path, body) ─────────┤      │
├── delete(path) ────────────┤      ├── build()
│                            │      │       │
└── refresh_token() ─────────┘      │       ▼
                                    │   gloo_net::http::Request
                                    │       │
                                    ▼       ▼
                                send() ─────┘
                                    │
                                    ▼
                            ApiResponse
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
            AgentService     DaoService      TreasuryService
                    │               │               │
                    ▼               ▼               ▼
            - list()          - get_summary()   - get_info()
            - get()           - list_proposals()
            - create()        - vote()
            - start()         - create_proposal()
            - stop()
```

### 7.3 状态管理函数关系

```
┌────────────────────────────────────────────────────────────────┐
│                        AppState                                 │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  auth: AuthState ───────────┐                                  │
│    ├── is_authenticated()   │                                  │
│    ├── has_role()           │                                  │
│    ├── has_permission()     │                                  │
│    ├── login()              │                                  │
│    └── logout()             │                                  │
│                             │                                  │
│  agent: AgentState ─────────┼──┐                               │
│    ├── set_agents()         │  │                               │
│    ├── set_filters()        │  │                               │
│    └── set_pagination()     │  │                               │
│                             │  │                               │
│  dao: DaoState ─────────────┼──┼──┐                            │
│    ├── set_proposals()      │  │  │                            │
│    └── set_votes()          │  │  │                            │
│                             │  │  │                            │
│  notification: NotificationState ─┼──┐                         │
│    ├── add()                │  │  │  │                         │
│    ├── remove()             │  │  │  │                         │
│    └── clear()              │  │  │  │                         │
│                             ▼  ▼  ▼  ▼                         │
│                           RwSignal<T>                          │
│                                                                 │
│  api_client: ApiClient ────────────────────┐                   │
│    ├── agent_service() ──▶ AgentService   │                   │
│    ├── dao_service() ────▶ DaoService     │                   │
│    └── ...                                 │                   │
│                                            ▼                   │
│                                        HTTP API                │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

## 八、安全机制

### 8.1 认证安全

```rust
/// Token 存储策略
/// 
/// ┌─────────────────────────────────────────────────────────────┐
/// │  存储位置         │  access_token  │  refresh_token         │
/// ├─────────────────────────────────────────────────────────────┤
/// │  Memory (RwSignal)│      ✓         │      ✓                │
/// │  localStorage     │      ✓         │      ✗ (安全考虑)      │
/// └─────────────────────────────────────────────────────────────┘
///
/// 安全措施：
/// 1. refresh_token 仅保存在内存中，防止 XSS 攻击窃取
/// 2. Token 过期前自动刷新（5 分钟阈值）
/// 3. 刷新失败强制重新登录
```

### 8.2 CSRF 防护

```rust
/// 自动为状态改变请求添加 CSRF Token
fn is_state_changing_method(method: &str) -> bool {
    matches!(method, "POST" | "PUT" | "PATCH" | "DELETE")
}

/// 请求头
/// X-CSRF-Token: <csrf_token>
/// Origin: <window_origin>
/// X-Requested-With: XMLHttpRequest
```

### 8.3 输入消毒

```rust
// utils/security.rs
pub fn escape_html(input: &str) -> String {
    // 转义 HTML 特殊字符
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn sanitize_url(url: &str) -> Option<String> {
    // 过滤危险协议
    if url.starts_with("javascript:") || url.starts_with("data:") {
        return None;
    }
    Some(url.to_string())
}
```

---

## 九、性能优化策略

### 9.1 状态拆分优化

```rust
// 优化前：单一状态对象（导致所有组件重渲染）
pub struct AppStateOld {
    pub user: RwSignal<Option<User>>,
    pub agents: RwSignal<Vec<AgentInfo>>,
    pub proposals: RwSignal<Vec<ProposalInfo>>,
    // ... 所有状态
}

// 优化后：领域拆分（组件只订阅需要的领域）
pub struct AppState {
    pub auth: AuthState,           // 组件只需认证状态时使用
    pub agent: AgentState,         // Agent 相关组件使用
    pub dao: DaoState,             // DAO 页面组件使用
    pub notification: NotificationState,
}
```

### 9.2 请求缓存策略

```rust
impl ApiClient {
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let cache_key = format!("GET:{}", path);
        
        // 1. 检查缓存
        if let Some(cached) = self.get_cached(&cache_key) {
            return Ok(cached);
        }
        
        // 2. 检查是否有进行中的相同请求（去重）
        if self.is_in_flight(&cache_key) {
            return self.wait_for_in_flight(&cache_key).await;
        }
        
        // 3. 发送请求
        let result = self.send_request(path).await?;
        
        // 4. 缓存结果
        self.set_cache(cache_key, result.clone());
        
        Ok(result)
    }
}
```

### 9.3 组件渲染优化

```rust
// 使用 Memo 缓存计算结果
let filtered_agents = Memo::new(move |_| {
    let agents = agents.get();
    let filters = filters.get();
    apply_filters(agents, filters)  // 只有 agents 或 filters 变化时才重新计算
});

// 使用 LocalResource 替代 Resource（CSR 模式）
let data = LocalResource::new(|| async { fetch_data().await });
// 注意：LocalResource 不需要 Send，适合 WASM 单线程环境
```

---

## 附录：模块依赖图

```
beebotos-web
│
├── leptos (v0.8.6)
│   ├── leptos::prelude::*     (核心信号、组件)
│   ├── leptos::task::*        (异步任务)
│   └── leptos::mount::*       (应用挂载)
│
├── leptos_router (v0.8.6)
│   ├── Router, Routes, Route  (路由组件)
│   ├── A                      (链接组件)
│   └── use_navigate           (导航 hook)
│
├── leptos_meta (v0.8.6)
│   ├── Title                  (页面标题)
│   └── provide_meta_context   (Meta 上下文)
│
├── gloo-net (v0.5)
│   └── http::Request          (HTTP 请求)
│
├── gloo-storage (v0.3)
│   ├── LocalStorage           (本地存储)
│   └── SessionStorage         (会话存储)
│
├── gloo-timers (v0.3)
│   ├── callback::Interval     (定时器)
│   └── future::TimeoutFuture  (超时 Future)
│
├── wasm-bindgen (v0.2)
│   └── JsCast                 (JS 类型转换)
│
├── web-sys (v0.3)
│   ├── console                (控制台日志)
│   ├── window, document       (DOM API)
│   └── Crypto                 (加密 API)
│
└── chrono (v0.4)
    └── DateTime, Utc          (日期时间处理)
```

---

*文档版本: 1.0*  
*最后更新: 2026-03-22*  
*维护者: BeeBotOS Dev Team*
