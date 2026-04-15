//! MCP Server
//!
//! Server implementation for MCP protocol.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;

use super::types::*;
use super::MCPError;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
    pub capabilities: ServerCapabilities,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "BeeBot MCP Server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: true }),
                resources: Some(ResourcesCapability {
                    subscribe: true,
                    list_changed: true,
                }),
                prompts: Some(PromptsCapability { list_changed: true }),
                logging: Some(Value::Object(Default::default())),
            },
        }
    }
}

/// Tool handler trait
#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync {
    async fn call(
        &self,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<Vec<ToolContent>, MCPError>;
}

/// Resource handler trait
#[async_trait::async_trait]
pub trait ResourceHandler: Send + Sync {
    async fn read(&self, uri: &str) -> Result<ResourceContents, MCPError>;
}

/// MCP Server
pub struct MCPServer {
    config: ServerConfig,
    tools: Arc<RwLock<HashMap<String, (Tool, Arc<dyn ToolHandler>)>>>,
    resources: Arc<RwLock<HashMap<String, (Resource, Arc<dyn ResourceHandler>)>>>,
    prompts: Arc<RwLock<HashMap<String, Prompt>>>,
}

impl MCPServer {
    /// Create new MCP server
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            prompts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a tool
    pub async fn register_tool(
        &self,
        tool: Tool,
        handler: Arc<dyn ToolHandler>,
    ) -> Result<(), MCPError> {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name.clone(), (tool, handler));
        Ok(())
    }

    /// Register a resource
    pub async fn register_resource(
        &self,
        resource: Resource,
        handler: Arc<dyn ResourceHandler>,
    ) -> Result<(), MCPError> {
        let mut resources = self.resources.write().await;
        resources.insert(resource.uri.clone(), (resource, handler));
        Ok(())
    }

    /// Register a prompt
    pub async fn register_prompt(&self, prompt: Prompt) -> Result<(), MCPError> {
        let mut prompts = self.prompts.write().await;
        prompts.insert(prompt.name.clone(), prompt);
        Ok(())
    }

    /// Handle initialize request
    pub async fn handle_initialize(
        &self,
        params: InitializeParams,
    ) -> Result<InitializeResult, MCPError> {
        // Check protocol version compatibility
        if params.protocol_version != super::types::MCPVersion::current().protocol_version {
            tracing::warn!(
                "Protocol version mismatch: client={}, server={}",
                params.protocol_version,
                super::types::MCPVersion::current().protocol_version
            );
        }

        Ok(InitializeResult {
            protocol_version: super::types::MCPVersion::current().protocol_version,
            capabilities: self.config.capabilities.clone(),
            server_info: Implementation {
                name: self.config.name.clone(),
                version: self.config.version.clone(),
            },
        })
    }

    /// Handle tools/list request
    pub async fn handle_list_tools(&self, _cursor: Option<&str>) -> ListToolsResult {
        let tools = self.tools.read().await;
        ListToolsResult {
            tools: tools.values().map(|(t, _)| t.clone()).collect(),
            next_cursor: None,
        }
    }

    /// Handle tools/call request
    pub async fn handle_call_tool(
        &self,
        params: CallToolParams,
    ) -> Result<CallToolResult, MCPError> {
        let tools = self.tools.read().await;

        let (_, handler) = tools
            .get(&params.name)
            .ok_or_else(|| MCPError::ToolNotFound(params.name.clone()))?;

        let content = handler.call(params.arguments).await?;

        Ok(CallToolResult {
            content,
            is_error: false,
        })
    }

    /// Handle resources/list request
    pub async fn handle_list_resources(&self, _cursor: Option<&str>) -> ListResourcesResult {
        let resources = self.resources.read().await;
        ListResourcesResult {
            resources: resources.values().map(|(r, _)| r.clone()).collect(),
            next_cursor: None,
        }
    }

    /// Handle resources/read request
    pub async fn handle_read_resource(
        &self,
        params: ReadResourceParams,
    ) -> Result<ReadResourceResult, MCPError> {
        let resources = self.resources.read().await;

        let (_, handler) = resources
            .get(&params.uri)
            .ok_or_else(|| MCPError::ResourceNotFound(params.uri.clone()))?;

        let content = handler.read(&params.uri).await?;

        Ok(ReadResourceResult {
            contents: vec![content],
        })
    }

    /// Handle prompts/list request
    pub async fn handle_list_prompts(&self, _cursor: Option<&str>) -> ListPromptsResult {
        let prompts = self.prompts.read().await;
        ListPromptsResult {
            prompts: prompts.values().cloned().collect(),
            next_cursor: None,
        }
    }

    /// Handle prompts/get request
    pub async fn handle_get_prompt(
        &self,
        params: GetPromptParams,
    ) -> Result<GetPromptResult, MCPError> {
        let prompts = self.prompts.read().await;

        let prompt = prompts
            .get(&params.name)
            .ok_or_else(|| MCPError::RequestFailed(format!("Prompt not found: {}", params.name)))?;

        // Build prompt messages with arguments substituted
        let messages = vec![PromptMessage {
            role: Role::User,
            content: PromptContent::Text {
                text: format!("Prompt: {} with args {:?}", prompt.name, params.arguments),
            },
        }];

        Ok(GetPromptResult {
            description: prompt.description.clone(),
            messages,
        })
    }

    /// Handle ping request
    pub async fn handle_ping(&self) -> Result<(), MCPError> {
        Ok(())
    }
}

/// Simple tool handler for functions
pub struct FnToolHandler<F>
where
    F: Fn(Option<HashMap<String, Value>>) -> Result<Vec<ToolContent>, MCPError> + Send + Sync,
{
    handler: F,
}

impl<F> FnToolHandler<F>
where
    F: Fn(Option<HashMap<String, Value>>) -> Result<Vec<ToolContent>, MCPError> + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

#[async_trait::async_trait]
impl<F> ToolHandler for FnToolHandler<F>
where
    F: Fn(Option<HashMap<String, Value>>) -> Result<Vec<ToolContent>, MCPError> + Send + Sync,
{
    async fn call(
        &self,
        arguments: Option<HashMap<String, Value>>,
    ) -> Result<Vec<ToolContent>, MCPError> {
        (self.handler)(arguments)
    }
}

/// Simple resource handler for static content
pub struct StaticResourceHandler {
    content: String,
    mime_type: String,
}

impl StaticResourceHandler {
    pub fn new(content: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            mime_type: mime_type.into(),
        }
    }
}

#[async_trait::async_trait]
impl ResourceHandler for StaticResourceHandler {
    async fn read(&self, uri: &str) -> Result<ResourceContents, MCPError> {
        Ok(ResourceContents::Text {
            uri: uri.to_string(),
            mime_type: Some(self.mime_type.clone()),
            text: self.content.clone(),
        })
    }
}
