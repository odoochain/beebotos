
## beebotos-web 编译和使用指南

**beebotos-web** 是 BeeBotOS 的 **Web 前端应用**，使用 **Leptos** 框架 + **WASM** 技术构建的单页应用（SPA）。

---

### 📦 编译前准备

确保已安装 WASM 工具链：

```bash
# 安装 wasm32 目标
rustup target add wasm32-unknown-unknown

# 安装 wasm-bindgen-cli（用于生成 JS 绑定）
cargo install wasm-bindgen-cli

# 安装 trunk（Leptos 推荐的构建工具，可选）
cargo install trunk
```

---

### 🔨 编译命令

#### 方法 1：使用 Cargo 直接编译（推荐）
```bash
# 编译整个项目（包含 web）
cargo build --release

# 只编译 web 包（仅生成 WASM，需要额外处理）
cargo build --release -p beebotos-web --target wasm32-unknown-unknown
```

#### 方法 2：使用 wasm-pack（推荐用于部署）
```bash
# 进入 web 目录
cd apps/web

# 编译为 WASM 并生成 JS 绑定
wasm-pack build --target web --out-dir pkg

# 或使用 release 模式
wasm-pack build --release --target web --out-dir pkg
```

#### 方法 3：使用 trunk（Leptos 原生支持）
```bash
cd apps/web

# 开发模式（带热重载）
trunk serve

# 生产构建
trunk build --release
```

---

### 🚀 使用方法

#### 1. 开发模式运行
```bash
cd apps/web

# 使用 trunk 启动开发服务器
trunk serve --open
# 或指定端口
trunk serve --port 3000

# 或使用 Python 简单服务器（如果已手动构建）
python3 -m http.server 8080
```

#### 2. 手动部署
```bash
# 1. 编译 WASM
cd apps/web
wasm-pack build --release --target web --out-dir pkg

# 2. 复制资源文件
cp -r style pkg/
cp index.html pkg/

# 3. 使用任意静态文件服务器
cd pkg
python3 -m http.server 8080
# 或
npx serve .
```

#### 3. 访问应用
```
http://localhost:8080
```

---

### 📁 项目结构

```
apps/web/
├── Cargo.toml          # Rust 依赖配置
├── index.html          # HTML 入口
├── src/
│   ├── main.rs         # WASM 入口
│   ├── lib.rs          # App 组件定义
│   ├── api/            # API 客户端
│   ├── components/     # 可复用组件
│   │   ├── nav.rs      # 导航栏
│   │   └── footer.rs   # 页脚
│   ├── pages/          # 页面组件
│   │   ├── home.rs     # 首页
│   │   ├── agents.rs   # Agent 列表
│   │   ├── agent_detail.rs # Agent 详情
│   │   ├── dao.rs      # DAO 治理
│   │   ├── treasury.rs # 金库
│   │   ├── skills.rs   # 技能市场
│   │   ├── settings.rs # 设置
│   │   └── not_found.rs # 404
│   └── state/          # 全局状态管理
├── style/
│   └── main.css        # 样式文件
└── public/             # 静态资源
```

---

### 🌐 页面路由

| 路径 | 页面 | 说明 |
|------|------|------|
| `/` | Home | 首页/欢迎页 |
| `/agents` | AgentsPage | Agent 列表 |
| `/agents/:id` | AgentDetail | Agent 详情 |
| `/dao` | DaoPage | DAO 治理 |
| `/dao/treasury` | TreasuryPage | 金库管理 |
| `/skills` | SkillsPage | 技能市场 |
| `/settings` | SettingsPage | 系统设置 |

---

### 🛠 技术栈

| 技术 | 用途 |
|------|------|
| **Leptos** | Rust 前端框架（类似 React）|
| **WASM** | 编译为浏览器可执行代码 |
| **leptos_router** | 客户端路由 |
| **leptos_meta** | HTML 元标签管理 |
| **gloo-net** | HTTP 请求 |
| **gloo-storage** | 本地存储 |
| **wasm-bindgen** | Rust/JS 互操作 |

---

### ⚙️ 配置说明

**Cargo.toml 关键配置：**
```toml
[package.metadata.leptos]
output-name = "beebotos-web"    # 输出文件名
site-root = "target/site"        # 站点根目录
site-pkg-dir = "pkg"             # WASM 包目录
style-file = "style/main.css"    # 主样式文件
assets-dir = "public"            # 静态资源目录
```

---

### 🔧 常见问题

#### 1. 编译错误：`wasm32-unknown-unknown` 未找到
```bash
rustup target add wasm32-unknown-unknown
```

#### 2. JS 绑定错误
```bash
cargo install wasm-bindgen-cli
```

#### 3. 页面空白（404）
确保 `index.html` 中的路径正确：
```html
<script type="module">
    import init from '/pkg/beebotos_web.js';  <!-- 确认路径 -->
    init();
</script>
```

#### 4. 跨域问题（调用 API）
需要配置代理或确保 API 服务支持 CORS。

---

### 📦 生产部署

```bash
# 完整构建
cd apps/web
wasm-pack build --release --target web --out-dir pkg

# 部署到 Nginx
sudo cp -r pkg/* /var/www/beebotos-web/

# Nginx 配置示例
server {
    listen 80;
    server_name beebotos.local;
    root /var/www/beebotos-web;
    index index.html;
    
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

需要我帮你配置生产部署脚本或解决编译问题吗？


