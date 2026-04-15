//! Message Templates System
//!
//! Provides templating capabilities for messages across all channels.
//! Supports variable substitution with platform-specific formatting.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::info;

use super::{ContentType, PlatformType};
use crate::communication::{Message, MessageType};
use crate::error::{AgentError, Result};

/// Message template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: Option<String>,
    /// Platform this template is for (optional - generic if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<PlatformType>,
    /// Content type
    pub content_type: ContentType,
    /// Template content (simple variable substitution: {{variable}})
    pub template: String,
    /// Default variables
    #[serde(default)]
    pub default_variables: HashMap<String, String>,
    /// Required variables
    #[serde(default)]
    pub required_variables: Vec<String>,
    /// Platform-specific variants
    #[serde(default)]
    pub platform_variants: HashMap<PlatformType, String>,
}

impl MessageTemplate {
    /// Create a new template
    pub fn new(id: &str, name: &str, template: &str, content_type: ContentType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            platform: None,
            content_type,
            template: template.to_string(),
            default_variables: HashMap::new(),
            required_variables: Vec::new(),
            platform_variants: HashMap::new(),
        }
    }

    /// Set platform
    pub fn for_platform(mut self, platform: PlatformType) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Add default variable
    pub fn with_default(mut self, key: &str, value: &str) -> Self {
        self.default_variables
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Add required variable
    pub fn requires(mut self, key: &str) -> Self {
        self.required_variables.push(key.to_string());
        self
    }

    /// Add platform variant
    pub fn with_variant(mut self, platform: PlatformType, template: &str) -> Self {
        self.platform_variants
            .insert(platform, template.to_string());
        self
    }

    /// Validate variables
    pub fn validate_variables(&self, variables: &HashMap<String, String>) -> Result<()> {
        for required in &self.required_variables {
            if !variables.contains_key(required) && !self.default_variables.contains_key(required) {
                return Err(AgentError::configuration(format!(
                    "Missing required variable: {}",
                    required
                ))
                .into());
            }
        }
        Ok(())
    }

    /// Get template for platform
    fn get_template_for_platform(&self, platform: Option<PlatformType>) -> &str {
        if let Some(p) = platform {
            if let Some(variant) = self.platform_variants.get(&p) {
                return variant;
            }
        }
        &self.template
    }
}

/// Simple template engine using basic variable substitution
#[derive(Clone)]
pub struct TemplateEngine {
    templates: HashMap<String, MessageTemplate>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template
    pub fn register_template(&mut self, template: MessageTemplate) -> Result<()> {
        let template_id = template.id.clone();
        self.templates.insert(template_id.clone(), template);
        info!("Registered template: {}", template_id);
        Ok(())
    }

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&MessageTemplate> {
        self.templates.get(id)
    }

    /// Simple variable substitution
    fn substitute_variables(template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Remove any remaining placeholders
        result = result.replace("{{", "").replace("}}", "");

        result
    }

    /// Render a template
    pub fn render(
        &self,
        template_id: &str,
        variables: &HashMap<String, String>,
        platform: Option<PlatformType>,
    ) -> Result<String> {
        let template = self.templates.get(template_id).ok_or_else(|| {
            AgentError::configuration(format!("Template not found: {}", template_id))
        })?;

        // Validate variables
        template.validate_variables(variables)?;

        // Merge with defaults
        let mut context = template.default_variables.clone();
        context.extend(variables.clone());

        // Get template for platform
        let template_str = template.get_template_for_platform(platform);

        // Render
        let result = Self::substitute_variables(template_str, &context);

        Ok(result)
    }

    /// Create a message from template
    pub fn create_message(
        &self,
        template_id: &str,
        variables: &HashMap<String, String>,
        platform: PlatformType,
    ) -> Result<Message> {
        let template = self.templates.get(template_id).ok_or_else(|| {
            AgentError::configuration(format!("Template not found: {}", template_id))
        })?;

        let content = self.render(template_id, variables, Some(platform))?;

        let message_type = match template.content_type {
            ContentType::Text => MessageType::Text,
            ContentType::Image => MessageType::Image,
            ContentType::File => MessageType::File,
            ContentType::Audio => MessageType::Voice,
            ContentType::Video => MessageType::Video,
            ContentType::Sticker => MessageType::Sticker,
            _ => MessageType::Text,
        };

        Ok(Message {
            id: uuid::Uuid::new_v4(),
            thread_id: uuid::Uuid::new_v4(),
            platform,
            message_type,
            content,
            metadata: variables.clone(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Load templates from directory
    pub fn load_from_directory(&mut self, path: &str) -> Result<usize> {
        let mut count = 0;

        for entry in std::fs::read_dir(path)
            .map_err(|e| AgentError::configuration(format!("Failed to read directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| AgentError::configuration(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    AgentError::configuration(format!("Failed to read file: {}", e))
                })?;

                let template: MessageTemplate = serde_json::from_str(&content).map_err(|e| {
                    AgentError::configuration(format!("Failed to parse template: {}", e))
                })?;

                self.register_template(template)?;
                count += 1;
            }
        }

        info!("Loaded {} templates from {}", count, path);
        Ok(count)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in templates
pub mod built_in {
    use super::*;

    /// Welcome message template
    pub fn welcome() -> MessageTemplate {
        MessageTemplate::new(
            "welcome",
            "Welcome Message",
            "Hello {{name}}! Welcome to {{service}}.",
            ContentType::Text,
        )
        .with_description("Welcome message for new users")
        .requires("name")
        .with_default("service", "BeeBotOS")
        .with_variant(PlatformType::Discord, "🎉 Welcome {{name}} to {{service}}!")
        .with_variant(PlatformType::Slack, "*Welcome {{name}}!*")
    }

    /// Help message template
    pub fn help() -> MessageTemplate {
        MessageTemplate::new(
            "help",
            "Help Message",
            "*Available Commands*\n\n{{commands}}\n\nFor more info: {{support_url}}",
            ContentType::Text,
        )
        .with_description("Help message listing available commands")
    }

    /// Error message template
    pub fn error() -> MessageTemplate {
        MessageTemplate::new(
            "error",
            "Error Message",
            "Error: {{message}}\n\n{{#if show_help}}Type /help for assistance.{{/if}}",
            ContentType::Text,
        )
        .with_description("Error message template")
        .requires("message")
        .with_default("show_help", "true")
    }

    /// Status update template
    pub fn status_update() -> MessageTemplate {
        MessageTemplate::new(
            "status_update",
            "Status Update",
            "{{title}}\n\n{{items}}\n\nUpdated: {{timestamp}}",
            ContentType::Text,
        )
        .with_description("Status update message")
        .requires("title")
    }

    /// Notification template
    pub fn notification() -> MessageTemplate {
        MessageTemplate::new(
            "notification",
            "Notification",
            "{{title}}\n\n{{message}}\n\n{{action_url}}",
            ContentType::Text,
        )
        .with_description("General notification template")
        .requires("title")
        .requires("message")
    }

    /// Get all built-in templates
    pub fn all() -> Vec<MessageTemplate> {
        vec![welcome(), help(), error(), status_update(), notification()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let mut engine = TemplateEngine::new();
        engine.register_template(built_in::welcome()).unwrap();

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("is_new".to_string(), "true".to_string());

        let result = engine.render("welcome", &vars, None).unwrap();
        assert!(result.contains("Hello Alice!"));
        assert!(result.contains("Welcome to BeeBotOS"));
    }

    #[test]
    fn test_platform_variant() {
        let mut engine = TemplateEngine::new();
        engine.register_template(built_in::welcome()).unwrap();

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Bob".to_string());

        let discord_result = engine
            .render("welcome", &vars, Some(PlatformType::Discord))
            .unwrap();
        assert!(discord_result.contains("🎉"));

        let slack_result = engine
            .render("welcome", &vars, Some(PlatformType::Slack))
            .unwrap();
        assert!(slack_result.contains("*Welcome"));
    }

    #[test]
    fn test_variable_substitution() {
        let template = "Hello {{name}}, your code is {{code}}";
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("code".to_string(), "12345".to_string());

        let result = TemplateEngine::substitute_variables(template, &vars);
        assert_eq!(result, "Hello Alice, your code is 12345");
    }
}
