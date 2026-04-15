//! Message Formatter Module
//!
//! Provides functionality for formatting messages with various media
//! attachments into formats suitable for LLM consumption. Supports multimodal
//! messages with text, images, documents, audio, and video content.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{AgentError, Result};
use crate::media::attachment::{AttachmentType, ParsedAttachment};

/// A formatted message ready for LLM consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedMessage {
    /// Message text content
    pub text: String,
    /// Optional message ID
    pub id: Option<String>,
    /// Message role (user, assistant, system)
    pub role: MessageRole,
    /// Attachments included in the message
    pub attachments: Vec<MessageAttachment>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Timestamp when the message was formatted
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Message role enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Tool message
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(f, "system"),
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::Tool => write!(f, "tool"),
        }
    }
}

/// Message attachment with content data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    /// Unique identifier for the attachment
    pub id: String,
    /// Attachment type
    pub attachment_type: AttachmentType,
    /// File name
    pub file_name: Option<String>,
    /// MIME type
    pub mime_type: String,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Attachment content (base64 encoded for binary data)
    pub content: AttachmentContent,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Attachment content variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AttachmentContent {
    /// Text content
    Text { content: String },
    /// Base64 encoded binary content
    Base64 { data: String },
    /// URL reference
    Url { url: String },
    /// File path reference
    Path { path: String },
}

/// Message formatter for building multimodal messages
#[derive(Debug, Clone)]
pub struct MessageFormatter {
    /// Maximum total message size in bytes
    max_message_size: usize,
    /// Maximum attachments per message
    max_attachments: usize,
    /// Whether to include attachment metadata
    include_metadata: bool,
}

impl Default for MessageFormatter {
    fn default() -> Self {
        Self {
            max_message_size: 10 * 1024 * 1024, // 10MB
            max_attachments: 32,
            include_metadata: true,
        }
    }
}

impl MessageFormatter {
    /// Create a new message formatter with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum message size
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_message_size = max_size;
        self
    }

    /// Set maximum attachments
    pub fn with_max_attachments(mut self, max_attachments: usize) -> Self {
        self.max_attachments = max_attachments;
        self
    }

    /// Set whether to include metadata
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Build a multimodal message from text and attachments
    ///
    /// # Arguments
    /// * `text` - The text content of the message
    /// * `attachments` - List of parsed attachments to include
    /// * `role` - The message role
    ///
    /// # Returns
    /// A formatted message or an error if validation fails
    pub async fn build_multimodal_message(
        &self,
        text: impl Into<String>,
        attachments: Vec<ParsedAttachment>,
        role: MessageRole,
    ) -> Result<FormattedMessage> {
        let text = text.into();

        // Validate attachment count
        if attachments.len() > self.max_attachments {
            return Err(AgentError::InvalidConfig(format!(
                "Too many attachments: {} (max: {})",
                attachments.len(),
                self.max_attachments
            )));
        }

        // Convert attachments
        let message_attachments: Vec<MessageAttachment> = attachments
            .into_iter()
            .map(|att| self.convert_to_message_attachment(att))
            .collect();

        // Calculate total size
        let total_size = self.calculate_total_size(&text, &message_attachments);
        if total_size > self.max_message_size {
            return Err(AgentError::InvalidConfig(format!(
                "Message too large: {} bytes (max: {})",
                total_size, self.max_message_size
            )));
        }

        let mut metadata = HashMap::new();
        if self.include_metadata {
            metadata.insert(
                "attachment_count".to_string(),
                message_attachments.len().to_string(),
            );
            metadata.insert("total_size".to_string(), total_size.to_string());
        }

        Ok(FormattedMessage {
            text,
            id: Some(uuid::Uuid::new_v4().to_string()),
            role,
            attachments: message_attachments,
            metadata,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Format a message for LLM input
    ///
    /// Converts the formatted message into a format suitable for LLM APIs.
    /// Different LLM providers may have different format requirements.
    ///
    /// # Arguments
    /// * `message` - The formatted message to convert
    /// * `format` - The target format type
    ///
    /// # Returns
    /// JSON value representing the formatted message
    pub fn format_for_llm(
        &self,
        message: &FormattedMessage,
        format: LLMFormat,
    ) -> serde_json::Value {
        match format {
            LLMFormat::OpenAI => self.to_openai_format(message),
            LLMFormat::Anthropic => self.to_anthropic_format(message),
            LLMFormat::Gemini => self.to_gemini_format(message),
            LLMFormat::Generic => self.to_generic_format(message),
        }
    }

    /// Convert parsed attachment to message attachment
    fn convert_to_message_attachment(&self, attachment: ParsedAttachment) -> MessageAttachment {
        let content = if let Some(url) = attachment.url {
            AttachmentContent::Url { url }
        } else if let Some(file_key) = attachment.file_key {
            AttachmentContent::Path { path: file_key }
        } else {
            AttachmentContent::Text {
                content: String::new(),
            }
        };

        MessageAttachment {
            id: attachment.id,
            attachment_type: attachment.attachment_type,
            file_name: attachment.file_name,
            mime_type: attachment
                .mime_type
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            file_size: attachment.file_size,
            content,
            metadata: attachment.metadata,
        }
    }

    /// Calculate total message size
    fn calculate_total_size(&self, text: &str, attachments: &[MessageAttachment]) -> usize {
        let text_size = text.len();
        let attachments_size: usize = attachments
            .iter()
            .map(|att| match &att.content {
                AttachmentContent::Text { content } => content.len(),
                AttachmentContent::Base64 { data } => data.len(),
                AttachmentContent::Url { url } => url.len(),
                AttachmentContent::Path { path } => path.len(),
            })
            .sum();
        text_size + attachments_size
    }

    /// Convert to OpenAI format
    fn to_openai_format(&self, message: &FormattedMessage) -> serde_json::Value {
        let mut content_parts = vec![];

        // Add text content
        if !message.text.is_empty() {
            content_parts.push(serde_json::json!({
                "type": "text",
                "text": message.text
            }));
        }

        // Add image attachments
        for attachment in &message.attachments {
            if self.is_image_type(&attachment.attachment_type) {
                let image_url = match &attachment.content {
                    AttachmentContent::Url { url } => url.clone(),
                    AttachmentContent::Base64 { data } => {
                        format!("data:{};base64, {}", attachment.mime_type, data)
                    }
                    _ => continue,
                };

                content_parts.push(serde_json::json!({
                    "type": "image_url",
                    "image_url": {
                        "url": image_url,
                        "detail": "auto"
                    }
                }));
            }
        }

        serde_json::json!({
            "role": message.role.to_string(),
            "content": content_parts
        })
    }

    /// Convert to Anthropic format
    fn to_anthropic_format(&self, message: &FormattedMessage) -> serde_json::Value {
        let mut content_blocks = vec![];

        // Add text content
        if !message.text.is_empty() {
            content_blocks.push(serde_json::json!({
                "type": "text",
                "text": message.text
            }));
        }

        // Add image attachments
        for attachment in &message.attachments {
            if self.is_image_type(&attachment.attachment_type) {
                if let AttachmentContent::Base64 { data } = &attachment.content {
                    content_blocks.push(serde_json::json!({
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": attachment.mime_type,
                            "data": data
                        }
                    }));
                }
            }
        }

        // Add document attachments
        for attachment in &message.attachments {
            if self.is_document_type(&attachment.attachment_type) {
                if let AttachmentContent::Base64 { data } = &attachment.content {
                    content_blocks.push(serde_json::json!({
                        "type": "document",
                        "source": {
                            "type": "base64",
                            "media_type": attachment.mime_type,
                            "data": data
                        }
                    }));
                }
            }
        }

        serde_json::json!({
            "role": message.role.to_string(),
            "content": content_blocks
        })
    }

    /// Convert to Gemini format
    fn to_gemini_format(&self, message: &FormattedMessage) -> serde_json::Value {
        let mut parts = vec![];

        // Add text content
        if !message.text.is_empty() {
            parts.push(serde_json::json!({
                "text": message.text
            }));
        }

        // Add file data attachments
        for attachment in &message.attachments {
            match &attachment.content {
                AttachmentContent::Base64 { data } => {
                    parts.push(serde_json::json!({
                        "inline_data": {
                            "mime_type": attachment.mime_type,
                            "data": data
                        }
                    }));
                }
                AttachmentContent::Url { url } => {
                    parts.push(serde_json::json!({
                        "file_data": {
                            "mime_type": &attachment.mime_type,
                            "file_uri": url
                        }
                    }));
                }
                _ => {}
            }
        }

        serde_json::json!({
            "role": match message.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "model",
                _ => "user",
            },
            "parts": parts
        })
    }

    /// Convert to generic format
    fn to_generic_format(&self, message: &FormattedMessage) -> serde_json::Value {
        serde_json::json!({
            "role": message.role.to_string(),
            "text": message.text,
            "attachments": message.attachments.iter().map(|att| {
                serde_json::json!({
                    "id": att.id,
                    "type": format!("{:?}", att.attachment_type),
                    "mime_type": att.mime_type,
                    "file_name": att.file_name,
                    "content": match &att.content {
                        AttachmentContent::Text { content } => {
                            serde_json::json!({"type": "text", "content": content})
                        }
                        AttachmentContent::Base64 { data } => {
                            serde_json::json!({"type": "base64", "data": data})
                        }
                        AttachmentContent::Url { url } => {
                            serde_json::json!({"type": "url", "url": url})
                        }
                        AttachmentContent::Path { path } => {
                            serde_json::json!({"type": "path", "path": path})
                        }
                    }
                })
            }).collect::<Vec<_>>(),
            "metadata": message.metadata,
            "timestamp": message.timestamp.to_rfc3339()
        })
    }

    /// Check if attachment type is an image
    fn is_image_type(&self, attachment_type: &AttachmentType) -> bool {
        matches!(
            attachment_type,
            AttachmentType::Image | AttachmentType::EmbeddedImage | AttachmentType::Sticker
        )
    }

    /// Check if attachment type is a document
    fn is_document_type(&self, attachment_type: &AttachmentType) -> bool {
        matches!(
            attachment_type,
            AttachmentType::Document | AttachmentType::File
        )
    }
}

/// LLM format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LLMFormat {
    /// OpenAI format
    OpenAI,
    /// Anthropic format
    Anthropic,
    /// Google Gemini format
    Gemini,
    /// Generic format
    Generic,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::attachment::PlatformSource;

    fn create_test_attachment(attachment_type: AttachmentType) -> ParsedAttachment {
        ParsedAttachment {
            id: uuid::Uuid::new_v4().to_string(),
            attachment_type,
            file_key: Some("test_key".to_string()),
            url: None,
            file_name: Some("test_file.jpg".to_string()),
            file_size: Some(1024),
            mime_type: Some("image/jpeg".to_string()),
            extension: Some("jpg".to_string()),
            source_platform: PlatformSource::Generic,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_build_multimodal_message() {
        let formatter = MessageFormatter::new();
        let attachment = create_test_attachment(AttachmentType::Image);

        let message = formatter
            .build_multimodal_message("Hello with image", vec![attachment], MessageRole::User)
            .await
            .unwrap();

        assert_eq!(message.text, "Hello with image");
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.attachments.len(), 1);
    }

    #[test]
    fn test_format_for_llm_openai() {
        let formatter = MessageFormatter::new();
        let attachment = create_test_attachment(AttachmentType::Image);
        let message_attachment = MessageAttachment {
            id: attachment.id,
            attachment_type: attachment.attachment_type,
            file_name: attachment.file_name,
            mime_type: "image/jpeg".to_string(),
            file_size: attachment.file_size,
            content: AttachmentContent::Url {
                url: "https://example.com/image.jpg".to_string(),
            },
            metadata: HashMap::new(),
        };

        let message = FormattedMessage {
            text: "Check this image".to_string(),
            id: Some(uuid::Uuid::new_v4().to_string()),
            role: MessageRole::User,
            attachments: vec![message_attachment],
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        let formatted = formatter.format_for_llm(&message, LLMFormat::OpenAI);
        assert!(formatted.get("role").is_some());
        assert!(formatted.get("content").is_some());
    }

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::System.to_string(), "system");
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::Tool.to_string(), "tool");
    }
}
