# MCP (Model Context Protocol) 集成

> **标准化工具调用接口**

---

## 概述

MCP (Model Context Protocol) 是一个开放协议，用于标准化 AI 模型与外部工具、数据和服务的交互。

### 为什么使用 MCP

- **标准化** - 统一的工具调用接口
- **可发现** - 动态工具发现能力
- **安全** - 明确的权限控制
- **可组合** - 工具可以组合使用

---

## 核心概念

### Tool (工具)

```json
{
  "name": "weather_query",
  "description": "查询指定城市的天气信息",
  "parameters": {
    "type": "object",
    "properties": {
      "city": {
        "type": "string",
        "description": "城市名称"
      },
      "date": {
        "type": "string",
        "description": "日期 (YYYY-MM-DD)"
      }
    },
    "required": ["city"]
  },
  "returns": {
    "type": "object",
    "properties": {
      "temperature": {"type": "number"},
      "condition": {"type": "string"}
    }
  }
}
```

### Resource (资源)

```json
{
  "uri": "file:///data/users/profile.json",
  "name": "User Profile",
  "mimeType": "application/json",
  "description": "用户配置文件"
}
```

### Prompt (提示模板)

```json
{
  "name": "code_review",
  "description": "代码审查助手",
  "template": "请审查以下代码：\n\n{{code}}\n\n关注：1.安全性 2.性能 3.可读性"
}
```

---

## BeeBotOS MCP 实现

### 服务端 (Tool Provider)

```rust
use beebotos_mcp::{Server, Tool, ToolHandler};

#[derive(Serialize, Deserialize)]
struct WeatherInput {
    city: String,
    #[serde(default)]
    date: Option<String>,
}

#[derive(Serialize)]
struct WeatherOutput {
    temperature: f64,
    condition: String,
}

struct WeatherTool;

#[async_trait]
impl ToolHandler for WeatherTool {
    async fn call(&self, input: Value) -> Result<Value, ToolError> {
        let input: WeatherInput = serde_json::from_value(input)?;
        
        // 调用天气 API
        let weather = fetch_weather(&input.city, input.date).await?;
        
        Ok(json!(WeatherOutput {
            temperature: weather.temp,
            condition: weather.condition,
        }))
    }
}

fn main() {
    let server = Server::new()
        .tool(Tool {
            name: "weather_query".to_string(),
            description: "查询天气".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "city": {"type": "string"},
                    "date": {"type": "string"}
                }
            }),
            handler: Box::new(WeatherTool),
        })
        .serve("127.0.0.1:8080");
}
```

### 客户端 (Tool Consumer)

```rust
use beebotos_mcp::Client;

async fn use_tools() {
    let client = Client::connect("http://localhost:8080").await?;
    
    // 发现可用工具
    let tools = client.list_tools().await?;
    
    for tool in tools {
        println!("Available tool: {}", tool.name);
    }
    
    // 调用工具
    let result = client.call_tool(
        "weather_query",
        json!({"city": "Beijing"})
    ).await?;
    
    println!("Weather: {}", result);
}
```

---

## Agent 中使用 MCP

### 配置 MCP 技能

```yaml
# agent-with-mcp.yaml
name: "MCPAgent"
description: "使用 MCP 工具的 Agent"

skills:
  - id: "mcp-client"
    version: "1.0.0"
    config:
      servers:
        - name: "weather"
          url: "http://weather-mcp.internal:8080"
          
        - name: "calculator"
          url: "http://calc-mcp.internal:8080"
          
        - name: "database"
          url: "http://db-mcp.internal:8080"
          auth:
            type: "api_key"
            key: "${DB_MCP_KEY}"

mcp:
  tool_selection:
    strategy: "auto"  # auto | manual
    
  max_concurrent_tools: 5
  
  timeout_ms: 30000
```

### 运行时工具调用

```rust
impl Agent {
    async fn handle_user_query(&self, query: &str) -> String {
        // 1. 分析用户意图
        let intent = self.nlu.parse(query).await;
        
        // 2. 选择合适工具
        let tools = self.mcp.discover_tools().await;
        let selected = self.select_tools(&intent, &tools);
        
        // 3. 执行工具调用
        let results = self.mcp.execute_parallel(selected).await;
        
        // 4. 综合结果
        self.synthesize_response(results).await
    }
}
```

---

## 常用 MCP 工具

### 内置工具

| 工具名 | 描述 | 示例 |
|--------|------|------|
| `web_search` | 网络搜索 | 搜索新闻、资料 |
| `browser_navigate` | 浏览器导航 | 访问网页 |
| `file_read` | 文件读取 | 读取文档 |
| `code_execute` | 代码执行 | 运行脚本 |

### 第三方工具

```yaml
# 添加第三方 MCP 服务
mcp_servers:
  - name: "slack"
    url: "https://mcp.slack.com"
    tools:
      - "send_message"
      - "create_channel"
      
  - name: "github"
    url: "https://mcp.github.com"
    tools:
      - "create_issue"
      - "create_pr"
      
  - name: "stripe"
    url: "https://mcp.stripe.com"
    tools:
      - "create_payment"
      - "refund"
```

---

## 安全考虑

### 权限控制

```yaml
mcp:
  security:
    # 工具白名单
    allowed_tools:
      - "weather_query"
      - "calculator"
    
    # 工具黑名单
    blocked_tools:
      - "code_execute"
      - "file_write"
    
    # 需要确认的工具
    confirm_tools:
      - "payment_execute"
      - "email_send"
    
    # 每个工具的最大调用次数
    rate_limits:
      web_search: 100/hour
      api_call: 1000/hour
```

### 输入验证

```rust
impl MCPClient {
    async fn validate_input(&self, tool: &str, input: &Value) -> Result<(), Error> {
        let schema = self.get_tool_schema(tool).await?;
        
        // 验证输入符合 JSON Schema
        schema.validate(input)?;
        
        // 额外安全检查
        if tool == "code_execute" {
            self.validate_code(input)?;
        }
        
        Ok(())
    }
}
```

---

**文档版本**: v1.0.0
