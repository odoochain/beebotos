//! Model Input Converter Module
//!
//! Provides functionality for converting internal message formats to various
//! LLM provider formats including OpenAI, Anthropic, and Gemini.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::media::formatter::{
    AttachmentContent, FormattedMessage, MessageAttachment, MessageRole,
};

/// LLM Message types for universal representation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum LLMMessage {
    /// System message
    System {
        /// Message content
        content: String,
        /// Optional name
        name: Option<String>,
    },
    /// User message
    User {
        /// Message content
        content: Vec<LLMContent>,
        /// Optional name
        name: Option<String>,
    },
    /// Assistant message
    Assistant {
        /// Text content
        content: String,
        /// Tool calls if any
        tool_calls: Option<Vec<ToolCall>>,
        /// Reasoning content (for reasoning models)
        reasoning_content: Option<String>,
    },
    /// Tool response message
    Tool {
        /// Tool call ID
        tool_call_id: String,
        /// Tool response content
        content: String,
        /// Tool name
        name: Option<String>,
    },
}

/// LLM Content types for multimodal messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LLMContent {
    /// Text content
    Text {
        /// Text content
        text: String,
    },
    /// Image URL content
    Image {
        /// Image URL
        url: String,
        /// Detail level (low, high, auto)
        detail: Option<String>,
    },
    /// Base64 encoded image
    ImageBase64 {
        /// MIME type
        mime_type: String,
        /// Base64 encoded data
        data: String,
        /// Detail level
        detail: Option<String>,
    },
    /// Resource reference
    Resource {
        /// Resource URL or identifier
        resource: String,
        /// Resource MIME type
        mime_type: String,
    },
}

/// Tool call representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool type
    pub tool_type: String,
    /// Function call details
    pub function: FunctionCall,
}

/// Function call details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function name
    pub name: String,
    /// Function arguments as JSON string
    pub arguments: String,
}

/// Model input converter configuration
#[derive(Debug, Clone)]
pub struct ConverterConfig {
    /// Whether to include metadata in output
    pub include_metadata: bool,
    /// Default image detail level
    pub default_image_detail: String,
    /// Maximum content length
    pub max_content_length: usize,
}

impl Default for ConverterConfig {
    fn default() -> Self {
        Self {
            include_metadata: false,
            default_image_detail: "auto".to_string(),
            max_content_length: 100_000,
        }
    }
}

/// Model input converter for format transformations
#[derive(Debug, Clone)]
pub struct ModelInputConverter {
    config: ConverterConfig,
}

impl ModelInputConverter {
    /// Create a new converter with default config
    pub fn new() -> Self {
        Self {
            config: ConverterConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ConverterConfig) -> Self {
        Self { config }
    }

    /// Convert internal messages to LLM format
    ///
    /// # Arguments
    /// * `messages` - List of formatted messages to convert
    ///
    /// # Returns
    /// List of LLM messages in universal format
    pub fn convert_messages(&self, messages: &[FormattedMessage]) -> Result<Vec<LLMMessage>> {
        messages
            .iter()
            .map(|msg| self.convert_single_message(msg))
            .collect()
    }

    /// Convert a single formatted message
    fn convert_single_message(&self, message: &FormattedMessage) -> Result<LLMMessage> {
        // Build content from text and attachments
        let mut content_parts = vec![];

        // Add text content
        if !message.text.is_empty() {
            content_parts.push(LLMContent::Text {
                text: message.text.clone(),
            });
        }

        // Add attachment content
        for attachment in &message.attachments {
            let content = self.convert_attachment(attachment)?;
            if let Some(c) = content {
                content_parts.push(c);
            }
        }

        // Create message based on role
        match message.role {
            MessageRole::System => Ok(LLMMessage::System {
                content: message.text.clone(),
                name: None,
            }),
            MessageRole::User => Ok(LLMMessage::User {
                content: content_parts,
                name: None,
            }),
            MessageRole::Assistant => Ok(LLMMessage::Assistant {
                content: message.text.clone(),
                tool_calls: None,
                reasoning_content: None,
            }),
            MessageRole::Tool => Ok(LLMMessage::Tool {
                tool_call_id: message.id.clone().unwrap_or_default(),
                content: message.text.clone(),
                name: None,
            }),
        }
    }

    /// Convert attachment to LLM content
    fn convert_attachment(&self, attachment: &MessageAttachment) -> Result<Option<LLMContent>> {
        match &attachment.content {
            AttachmentContent::Text { content } => Ok(Some(LLMContent::Text {
                text: content.clone(),
            })),
            AttachmentContent::Base64 { data } => {
                if self.is_image_mime(&attachment.mime_type) {
                    Ok(Some(LLMContent::ImageBase64 {
                        mime_type: attachment.mime_type.clone(),
                        data: data.clone(),
                        detail: Some(self.config.default_image_detail.clone()),
                    }))
                } else {
                    Ok(Some(LLMContent::Resource {
                        resource: format!(
                            "data:{};base64,{}...",
                            attachment.mime_type,
                            &data[..20.min(data.len())]
                        ),
                        mime_type: attachment.mime_type.clone(),
                    }))
                }
            }
            AttachmentContent::Url { url } => {
                if self.is_image_mime(&attachment.mime_type) {
                    Ok(Some(LLMContent::Image {
                        url: url.clone(),
                        detail: Some(self.config.default_image_detail.clone()),
                    }))
                } else {
                    Ok(Some(LLMContent::Resource {
                        resource: url.clone(),
                        mime_type: attachment.mime_type.clone(),
                    }))
                }
            }
            AttachmentContent::Path { path } => {
                // Path references need to be loaded first
                tracing::debug!("Path attachment not yet loaded: {}", path);
                Ok(None)
            }
        }
    }

    /// Convert messages to OpenAI format
    ///
    /// # Arguments
    /// * `messages` - List of LLM messages to convert
    ///
    /// # Returns
    /// JSON value in OpenAI format
    pub fn to_openai_format(&self, messages: &[LLMMessage]) -> serde_json::Value {
        let openai_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| match msg {
                LLMMessage::System { content, name } => {
                    let mut obj = serde_json::json!({
                        "role": "system",
                        "content": content
                    });
                    if let Some(n) = name {
                        obj["name"] = serde_json::json!(n);
                    }
                    obj
                }
                LLMMessage::User { content, name } => {
                    let content_parts: Vec<serde_json::Value> = content
                        .iter()
                        .map(|c| match c {
                            LLMContent::Text { text } => serde_json::json!({
                                "type": "text",
                                "text": text
                            }),
                            LLMContent::Image { url, detail } => serde_json::json!({
                                "type": "image_url",
                                "image_url": {
                                    "url": url,
                                    "detail": detail.as_deref().unwrap_or("auto")
                                }
                            }),
                            LLMContent::ImageBase64 { mime_type, data, detail } => {
                                serde_json::json!({
                                    "type": "image_url",
                                    "image_url": {
                                        "url": format!("data:{};base64, {}", mime_type, data),
                                        "detail": detail.as_deref().unwrap_or("auto")
                                    }
                                })
                            }
                            LLMContent::Resource { resource, .. } => serde_json::json!({
                                "type": "text",
                                "text": format!("Resource: {}", resource)
                            }),
                        })
                        .collect();

                    let mut obj = if content_parts.len() == 1 && content_parts[0].get("type") == Some(&serde_json::json!("text")) {
                        serde_json::json!({
                            "role": "user",
                            "content": content_parts[0].get("text").unwrap_or(&serde_json::json!(""))
                        })
                    } else {
                        serde_json::json!({
                            "role": "user",
                            "content": content_parts
                        })
                    };
                    if let Some(n) = name {
                        obj["name"] = serde_json::json!(n);
                    }
                    obj
                }
                LLMMessage::Assistant { content, tool_calls, reasoning_content } => {
                    let mut obj = serde_json::json!({
                        "role": "assistant",
                        "content": content
                    });
                    if let Some(calls) = tool_calls {
                        obj["tool_calls"] = serde_json::json!(calls.iter().map(|tc| serde_json::json!({
                            "id": tc.id,
                            "type": tc.tool_type,
                            "function": {
                                "name": tc.function.name,
                                "arguments": tc.function.arguments
                            }
                        })).collect::<Vec<_>>());
                    }
                    if let Some(reasoning) = reasoning_content {
                        obj["reasoning_content"] = serde_json::json!(reasoning);
                    }
                    obj
                }
                LLMMessage::Tool { tool_call_id, content, name } => {
                    let mut obj = serde_json::json!({
                        "role": "tool",
                        "tool_call_id": tool_call_id,
                        "content": content
                    });
                    if let Some(n) = name {
                        obj["name"] = serde_json::json!(n);
                    }
                    obj
                }
            })
            .collect();

        serde_json::json!(openai_messages)
    }

    /// Convert messages to Anthropic format
    ///
    /// # Arguments
    /// * `messages` - List of LLM messages to convert
    /// * `system_prompt` - Optional system prompt (Anthropic separates this)
    ///
    /// # Returns
    /// JSON value in Anthropic format
    pub fn to_anthropic_format(
        &self,
        messages: &[LLMMessage],
        system_prompt: Option<String>,
    ) -> serde_json::Value {
        let mut anthropic_messages: Vec<serde_json::Value> = Vec::new();

        for msg in messages {
            match msg {
                LLMMessage::System { .. } => {
                    // System messages are handled separately in Anthropic format
                    continue;
                }
                LLMMessage::User { content, .. } => {
                    let content_blocks: Vec<serde_json::Value> = content
                        .iter()
                        .map(|c| match c {
                            LLMContent::Text { text } => serde_json::json!({
                                "type": "text",
                                "text": text
                            }),
                            LLMContent::Image { url, .. } => serde_json::json!({
                                "type": "image",
                                "source": {
                                    "type": "url",
                                    "url": url
                                }
                            }),
                            LLMContent::ImageBase64 {
                                mime_type, data, ..
                            } => {
                                serde_json::json!({
                                    "type": "image",
                                    "source": {
                                        "type": "base64",
                                        "media_type": mime_type,
                                        "data": data
                                    }
                                })
                            }
                            LLMContent::Resource {
                                resource,
                                mime_type,
                            } => {
                                if mime_type.starts_with("application/pdf") {
                                    serde_json::json!({
                                        "type": "document",
                                        "source": {
                                            "type": "url",
                                            "url": resource
                                        }
                                    })
                                } else {
                                    serde_json::json!({
                                        "type": "text",
                                        "text": format!("Resource: {}", resource)
                                    })
                                }
                            }
                        })
                        .collect();

                    anthropic_messages.push(serde_json::json!({
                        "role": "user",
                        "content": content_blocks
                    }));
                }
                LLMMessage::Assistant { content, .. } => {
                    anthropic_messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": content
                    }));
                }
                LLMMessage::Tool { content, .. } => {
                    // Tool responses are user messages in Anthropic format
                    anthropic_messages.push(serde_json::json!({
                        "role": "user",
                        "content": content
                    }));
                }
            }
        }

        let mut result = serde_json::json!({
            "messages": anthropic_messages
        });

        // Add system prompt if provided or found
        let system = system_prompt.or_else(|| {
            messages.iter().find_map(|m| match m {
                LLMMessage::System { content, .. } => Some(content.clone()),
                _ => None,
            })
        });

        if let Some(s) = system {
            result["system"] = serde_json::json!(s);
        }

        result
    }

    /// Convert messages to Gemini format
    ///
    /// # Arguments
    /// * `messages` - List of LLM messages to convert
    /// * `system_instruction` - Optional system instruction
    ///
    /// # Returns
    /// JSON value in Gemini format
    pub fn to_gemini_format(
        &self,
        messages: &[LLMMessage],
        system_instruction: Option<String>,
    ) -> serde_json::Value {
        let mut contents: Vec<serde_json::Value> = Vec::new();

        for msg in messages {
            match msg {
                LLMMessage::System { .. } => {
                    // System instructions are handled separately in Gemini
                    continue;
                }
                LLMMessage::User { content, .. } => {
                    let parts: Vec<serde_json::Value> = content
                        .iter()
                        .map(|c| match c {
                            LLMContent::Text { text } => serde_json::json!({
                                "text": text
                            }),
                            LLMContent::Image { url, .. } => serde_json::json!({
                                "file_data": {
                                    "mime_type": "image/*",
                                    "file_uri": url
                                }
                            }),
                            LLMContent::ImageBase64 {
                                mime_type, data, ..
                            } => {
                                serde_json::json!({
                                    "inline_data": {
                                        "mime_type": mime_type,
                                        "data": data
                                    }
                                })
                            }
                            LLMContent::Resource {
                                resource,
                                mime_type,
                            } => {
                                if resource.starts_with("http") {
                                    serde_json::json!({
                                        "file_data": {
                                            "mime_type": mime_type,
                                            "file_uri": resource
                                        }
                                    })
                                } else {
                                    serde_json::json!({
                                        "text": format!("Resource: {}", resource)
                                    })
                                }
                            }
                        })
                        .collect();

                    contents.push(serde_json::json!({
                        "role": "user",
                        "parts": parts
                    }));
                }
                LLMMessage::Assistant { content, .. } => {
                    contents.push(serde_json::json!({
                        "role": "model",
                        "parts": [{"text": content}]
                    }));
                }
                LLMMessage::Tool { content, .. } => {
                    // Tool responses are user messages in Gemini format
                    contents.push(serde_json::json!({
                        "role": "user",
                        "parts": [{"text": content}]
                    }));
                }
            }
        }

        let mut result = serde_json::json!({
            "contents": contents
        });

        // Add system instruction if provided or found
        let system = system_instruction.or_else(|| {
            messages.iter().find_map(|m| match m {
                LLMMessage::System { content, .. } => Some(content.clone()),
                _ => None,
            })
        });

        if let Some(s) = system {
            result["system_instruction"] = serde_json::json!({
                "parts": [{"text": s}]
            });
        }

        result
    }

    /// Check if MIME type is an image
    fn is_image_mime(&self, mime_type: &str) -> bool {
        mime_type.starts_with("image/")
    }
}

impl Default for ModelInputConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::media::attachment::AttachmentType;

    fn create_test_formatted_message(role: MessageRole, text: &str) -> FormattedMessage {
        FormattedMessage {
            text: text.to_string(),
            id: Some(uuid::Uuid::new_v4().to_string()),
            role,
            attachments: vec![],
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_convert_messages() {
        let converter = ModelInputConverter::new();
        let messages = vec![
            create_test_formatted_message(MessageRole::System, "You are helpful"),
            create_test_formatted_message(MessageRole::User, "Hello"),
            create_test_formatted_message(MessageRole::Assistant, "Hi!"),
        ];

        let llm_messages = converter.convert_messages(&messages).unwrap();
        assert_eq!(llm_messages.len(), 3);
    }

    #[test]
    fn test_to_openai_format() {
        let converter = ModelInputConverter::new();
        let messages = vec![
            LLMMessage::System {
                content: "You are helpful".to_string(),
                name: None,
            },
            LLMMessage::User {
                content: vec![LLMContent::Text {
                    text: "Hello".to_string(),
                }],
                name: None,
            },
        ];

        let formatted = converter.to_openai_format(&messages);
        let arr = formatted.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["role"], "system");
        assert_eq!(arr[1]["role"], "user");
    }

    #[test]
    fn test_to_anthropic_format() {
        let converter = ModelInputConverter::new();
        let messages = vec![
            LLMMessage::User {
                content: vec![LLMContent::Text {
                    text: "Hello".to_string(),
                }],
                name: None,
            },
            LLMMessage::Assistant {
                content: "Hi there!".to_string(),
                tool_calls: None,
                reasoning_content: None,
            },
        ];

        let formatted = converter.to_anthropic_format(&messages, Some("System prompt".to_string()));
        assert!(formatted.get("messages").is_some());
        assert_eq!(formatted["system"], "System prompt");
    }

    #[test]
    fn test_to_gemini_format() {
        let converter = ModelInputConverter::new();
        let messages = vec![
            LLMMessage::User {
                content: vec![LLMContent::Text {
                    text: "Hello".to_string(),
                }],
                name: None,
            },
            LLMMessage::Assistant {
                content: "Hi!".to_string(),
                tool_calls: None,
                reasoning_content: None,
            },
        ];

        let formatted = converter.to_gemini_format(&messages, None);
        assert!(formatted.get("contents").is_some());
    }

    #[test]
    fn test_image_content_conversion() {
        let converter = ModelInputConverter::new();
        let attachment = MessageAttachment {
            id: "test".to_string(),
            attachment_type: AttachmentType::Image,
            file_name: Some("test.jpg".to_string()),
            mime_type: "image/jpeg".to_string(),
            file_size: Some(1024),
            content: AttachmentContent::Url {
                url: "https://example.com/image.jpg".to_string(),
            },
            metadata: HashMap::new(),
        };

        let content = converter.convert_attachment(&attachment).unwrap();
        assert!(content.is_some());
        match content.unwrap() {
            LLMContent::Image { url, .. } => {
                assert_eq!(url, "https://example.com/image.jpg");
            }
            _ => panic!("Expected Image content"),
        }
    }
}
