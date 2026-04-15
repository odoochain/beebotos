//! DingTalk Message Content Parser
//!
//! Provides parsing and handling of various DingTalk message content types
//! including text, markdown, action cards, images, files, voice, and links.

use serde::{Deserialize, Serialize};

use crate::communication::channel::content::{
    ContentType as UnifiedContentType, MediaContent, PlatformContent,
};
use crate::error::{AgentError, Result};

/// DingTalk message content types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "msg_type", content = "content")]
pub enum DingTalkContent {
    /// Plain text message
    #[serde(rename = "text")]
    Text(DingTalkTextContent),
    /// Markdown message
    #[serde(rename = "markdown")]
    Markdown(DingTalkMarkdownContent),
    /// Action card message
    #[serde(rename = "action_card")]
    ActionCard(DingTalkActionCardContent),
    /// Image message
    #[serde(rename = "image")]
    Image(DingTalkImageContent),
    /// Voice message
    #[serde(rename = "voice")]
    Voice(DingTalkVoiceContent),
    /// File message
    #[serde(rename = "file")]
    File(DingTalkFileContent),
    /// Link message
    #[serde(rename = "link")]
    Link(DingTalkLinkContent),
    /// Rich text message (feed card)
    #[serde(rename = "feedCard")]
    FeedCard(DingTalkFeedCardContent),
}

/// Text content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DingTalkTextContent {
    /// The text content
    pub content: String,
}

/// Markdown content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DingTalkMarkdownContent {
    /// Title
    pub title: String,
    /// Markdown text
    pub text: String,
}

/// Action card content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DingTalkActionCardContent {
    /// Title
    pub title: String,
    /// Markdown content
    pub markdown: String,
    /// Single button title (for single button card)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "singleTitle")]
    pub single_title: Option<String>,
    /// Single button URL (for single button card)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "singleURL")]
    pub single_url: Option<String>,
    /// Button orientation (0 for vertical, 1 for horizontal)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "btnOrientation")]
    pub btn_orientation: Option<String>,
    /// Multiple buttons (for multi-button card)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btns: Option<Vec<DingTalkActionCardButton>>,
}

/// Action card button
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DingTalkActionCardButton {
    /// Button title
    pub title: String,
    /// Button action URL
    #[serde(rename = "actionURL")]
    pub action_url: String,
}

/// Image content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DingTalkImageContent {
    /// Image URL (for outgoing messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>,
    /// Download code (for incoming messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "downloadCode")]
    pub download_code: Option<String>,
}

/// Voice content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DingTalkVoiceContent {
    /// Download code for the voice file
    #[serde(rename = "downloadCode")]
    pub download_code: String,
    /// Voice duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    /// Voice recognition text (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recognition: Option<String>,
}

/// File content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DingTalkFileContent {
    /// Download code for the file
    #[serde(rename = "downloadCode")]
    pub download_code: String,
    /// File name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "fileSize")]
    pub file_size: Option<i64>,
}

/// Link content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DingTalkLinkContent {
    /// Link title
    pub title: String,
    /// Link text/description
    pub text: String,
    /// Picture URL
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>,
    /// Message URL
    #[serde(rename = "messageUrl")]
    pub message_url: String,
}

/// Feed card content structure (rich text with multiple links)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DingTalkFeedCardContent {
    /// List of feed card links
    pub links: Vec<DingTalkFeedCardLink>,
}

/// Feed card link item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DingTalkFeedCardLink {
    /// Title
    pub title: String,
    /// Message URL
    #[serde(rename = "messageURL")]
    pub message_url: String,
    /// Picture URL
    #[serde(rename = "picURL")]
    pub pic_url: Option<String>,
}

/// DingTalk content parser for parsing message content from JSON
#[derive(Debug, Clone, Default)]
pub struct DingTalkContentParser;

impl DingTalkContentParser {
    /// Create a new content parser
    pub fn new() -> Self {
        Self
    }

    /// Parse content from JSON value
    ///
    /// # Arguments
    /// * `msg_type` - The message type (text, markdown, action_card, image,
    ///   etc.)
    /// * `content` - The JSON content to parse
    ///
    /// # Returns
    /// Parsed DingTalkContent enum variant
    pub fn parse(msg_type: &str, content: serde_json::Value) -> Result<DingTalkContent> {
        match msg_type {
            "text" => {
                let text_content: DingTalkTextContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse text content: {}", e))
                    })?;
                Ok(DingTalkContent::Text(text_content))
            }
            "markdown" => {
                let markdown_content: DingTalkMarkdownContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse markdown content: {}", e))
                    })?;
                Ok(DingTalkContent::Markdown(markdown_content))
            }
            "action_card" => {
                let action_card_content: DingTalkActionCardContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse action_card content: {}", e))
                    })?;
                Ok(DingTalkContent::ActionCard(action_card_content))
            }
            "image" => {
                let image_content: DingTalkImageContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse image content: {}", e))
                    })?;
                Ok(DingTalkContent::Image(image_content))
            }
            "voice" => {
                let voice_content: DingTalkVoiceContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse voice content: {}", e))
                    })?;
                Ok(DingTalkContent::Voice(voice_content))
            }
            "file" => {
                let file_content: DingTalkFileContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse file content: {}", e))
                    })?;
                Ok(DingTalkContent::File(file_content))
            }
            "link" => {
                let link_content: DingTalkLinkContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse link content: {}", e))
                    })?;
                Ok(DingTalkContent::Link(link_content))
            }
            "feedCard" => {
                let feed_card_content: DingTalkFeedCardContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse feedCard content: {}", e))
                    })?;
                Ok(DingTalkContent::FeedCard(feed_card_content))
            }
            _ => Err(AgentError::platform(format!("Unknown message type: {}", msg_type)).into()),
        }
    }

    /// Parse content from JSON string
    ///
    /// # Arguments
    /// * `msg_type` - The message type
    /// * `content_json` - JSON string to parse
    ///
    /// # Returns
    /// Parsed DingTalkContent enum variant
    pub fn parse_str(msg_type: &str, content_json: &str) -> Result<DingTalkContent> {
        let content: serde_json::Value = serde_json::from_str(content_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON: {}", e)))?;
        Self::parse(msg_type, content)
    }

    /// Parse from webhook payload JSON
    ///
    /// # Arguments
    /// * `payload_json` - Full webhook payload JSON string
    ///
    /// # Returns
    /// Parsed DingTalkContent enum variant
    pub fn parse_from_payload(payload_json: &str) -> Result<DingTalkContent> {
        let payload: serde_json::Value = serde_json::from_str(payload_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON payload: {}", e)))?;

        let msg_type = payload
            .get("msgtype")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        // Extract content based on message type
        let content = match msg_type {
            "text" => payload.get("text").cloned(),
            "markdown" => payload.get("markdown").cloned(),
            "action_card" => payload.get("actionCard").cloned(),
            "image" => payload.get("image").cloned(),
            "voice" => payload.get("voice").cloned(),
            "file" => payload.get("file").cloned(),
            "link" => payload.get("link").cloned(),
            "feedCard" => payload.get("feedCard").cloned(),
            _ => None,
        };

        let content = content.unwrap_or_else(|| serde_json::json!({}));
        Self::parse(msg_type, content)
    }

    /// Extract plain text from any content type
    ///
    /// # Arguments
    /// * `content` - The DingTalkContent to extract text from
    ///
    /// # Returns
    /// Extracted plain text string
    pub fn extract_text(content: &DingTalkContent) -> String {
        match content {
            DingTalkContent::Text(text) => text.content.clone(),
            DingTalkContent::Markdown(markdown) => {
                format!("{}\n{}", markdown.title, markdown.text)
            }
            DingTalkContent::ActionCard(card) => {
                format!("{}\n{}", card.title, card.markdown)
            }
            DingTalkContent::Image(image) => {
                if let Some(url) = &image.pic_url {
                    format!("[Image: {}]", url)
                } else if let Some(code) = &image.download_code {
                    format!("[Image with download code: {}]", code)
                } else {
                    "[Image]".to_string()
                }
            }
            DingTalkContent::Voice(voice) => {
                if let Some(recognition) = &voice.recognition {
                    format!("[Voice: {}]", recognition)
                } else {
                    "[Voice message]".to_string()
                }
            }
            DingTalkContent::File(file) => {
                format!("[File: {}]", file.file_name.as_deref().unwrap_or("unnamed"))
            }
            DingTalkContent::Link(link) => {
                format!("[Link: {} - {}]", link.title, link.message_url)
            }
            DingTalkContent::FeedCard(feed_card) => {
                let links: Vec<String> = feed_card
                    .links
                    .iter()
                    .map(|l| format!("- {}: {}", l.title, l.message_url))
                    .collect();
                format!("[Feed Card]\n{}", links.join("\n"))
            }
        }
    }

    /// Get message type from content
    ///
    /// # Arguments
    /// * `content` - The DingTalkContent
    ///
    /// # Returns
    /// Message type string
    pub fn get_msg_type(content: &DingTalkContent) -> &'static str {
        match content {
            DingTalkContent::Text(_) => "text",
            DingTalkContent::Markdown(_) => "markdown",
            DingTalkContent::ActionCard(_) => "action_card",
            DingTalkContent::Image(_) => "image",
            DingTalkContent::Voice(_) => "voice",
            DingTalkContent::File(_) => "file",
            DingTalkContent::Link(_) => "link",
            DingTalkContent::FeedCard(_) => "feedCard",
        }
    }

    /// Create text content
    ///
    /// # Arguments
    /// * `text` - The text content
    ///
    /// # Returns
    /// DingTalkContent::Text variant
    pub fn create_text(text: impl Into<String>) -> DingTalkContent {
        DingTalkContent::Text(DingTalkTextContent {
            content: text.into(),
        })
    }

    /// Create markdown content
    ///
    /// # Arguments
    /// * `title` - The title
    /// * `text` - The markdown text
    ///
    /// # Returns
    /// DingTalkContent::Markdown variant
    pub fn create_markdown(title: impl Into<String>, text: impl Into<String>) -> DingTalkContent {
        DingTalkContent::Markdown(DingTalkMarkdownContent {
            title: title.into(),
            text: text.into(),
        })
    }

    /// Create action card content with single button
    ///
    /// # Arguments
    /// * `title` - Card title
    /// * `markdown` - Card content in markdown
    /// * `single_title` - Button title
    /// * `single_url` - Button URL
    ///
    /// # Returns
    /// DingTalkContent::ActionCard variant
    pub fn create_action_card(
        title: impl Into<String>,
        markdown: impl Into<String>,
        single_title: impl Into<String>,
        single_url: impl Into<String>,
    ) -> DingTalkContent {
        DingTalkContent::ActionCard(DingTalkActionCardContent {
            title: title.into(),
            markdown: markdown.into(),
            single_title: Some(single_title.into()),
            single_url: Some(single_url.into()),
            btn_orientation: None,
            btns: None,
        })
    }

    /// Create action card content with multiple buttons
    ///
    /// # Arguments
    /// * `title` - Card title
    /// * `markdown` - Card content in markdown
    /// * `buttons` - List of buttons
    ///
    /// # Returns
    /// DingTalkContent::ActionCard variant
    pub fn create_action_card_with_buttons(
        title: impl Into<String>,
        markdown: impl Into<String>,
        buttons: Vec<DingTalkActionCardButton>,
    ) -> DingTalkContent {
        DingTalkContent::ActionCard(DingTalkActionCardContent {
            title: title.into(),
            markdown: markdown.into(),
            single_title: None,
            single_url: None,
            btn_orientation: Some("0".to_string()),
            btns: Some(buttons),
        })
    }

    /// Create image content
    ///
    /// # Arguments
    /// * `pic_url` - Image URL
    ///
    /// # Returns
    /// DingTalkContent::Image variant
    pub fn create_image(pic_url: impl Into<String>) -> DingTalkContent {
        DingTalkContent::Image(DingTalkImageContent {
            pic_url: Some(pic_url.into()),
            download_code: None,
        })
    }

    /// Create link content
    ///
    /// # Arguments
    /// * `title` - Link title
    /// * `text` - Link text
    /// * `message_url` - Message URL
    ///
    /// # Returns
    /// DingTalkContent::Link variant
    pub fn create_link(
        title: impl Into<String>,
        text: impl Into<String>,
        message_url: impl Into<String>,
    ) -> DingTalkContent {
        DingTalkContent::Link(DingTalkLinkContent {
            title: title.into(),
            text: text.into(),
            pic_url: None,
            message_url: message_url.into(),
        })
    }

    /// Serialize content to JSON string
    ///
    /// # Arguments
    /// * `content` - The DingTalkContent to serialize
    ///
    /// # Returns
    /// JSON string representation
    pub fn to_json(content: &DingTalkContent) -> Result<String> {
        serde_json::to_string(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Serialize content to JSON value
    ///
    /// # Arguments
    /// * `content` - The DingTalkContent to serialize
    ///
    /// # Returns
    /// JSON value representation
    pub fn to_json_value(content: &DingTalkContent) -> Result<serde_json::Value> {
        serde_json::to_value(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Check if content type supports buttons/actions
    pub fn has_actions(content: &DingTalkContent) -> bool {
        matches!(
            content,
            DingTalkContent::ActionCard(_) | DingTalkContent::Link(_)
        )
    }

    /// Get download code for media content (image, voice, file)
    ///
    /// # Arguments
    /// * `content` - The DingTalkContent
    ///
    /// # Returns
    /// Optional download code
    pub fn get_download_code(content: &DingTalkContent) -> Option<String> {
        match content {
            DingTalkContent::Image(image) => image.download_code.clone(),
            DingTalkContent::Voice(voice) => Some(voice.download_code.clone()),
            DingTalkContent::File(file) => Some(file.download_code.clone()),
            _ => None,
        }
    }

    /// Get file name for file content
    ///
    /// # Arguments
    /// * `content` - The DingTalkContent
    ///
    /// # Returns
    /// Optional file name
    pub fn get_file_name(content: &DingTalkContent) -> Option<String> {
        match content {
            DingTalkContent::File(file) => file.file_name.clone(),
            DingTalkContent::Voice(voice) => Some(format!("voice_{}.amr", voice.download_code)),
            DingTalkContent::Image(image) => {
                if let Some(code) = &image.download_code {
                    Some(format!("image_{}.jpg", code))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_content() {
        let json = serde_json::json!({
            "content": "Hello, World!"
        });
        let content = DingTalkContentParser::parse("text", json).unwrap();
        assert!(matches!(content, DingTalkContent::Text(_)));
        assert_eq!(
            DingTalkContentParser::extract_text(&content),
            "Hello, World!"
        );
    }

    #[test]
    fn test_parse_markdown_content() {
        let json = serde_json::json!({
            "title": "Title",
            "text": "## Markdown content"
        });
        let content = DingTalkContentParser::parse("markdown", json).unwrap();
        assert!(matches!(content, DingTalkContent::Markdown(_)));
        let text = DingTalkContentParser::extract_text(&content);
        assert!(text.contains("Title"));
        assert!(text.contains("Markdown content"));
    }

    #[test]
    fn test_parse_image_content() {
        let json = serde_json::json!({
            "downloadCode": "img_12345"
        });
        let content = DingTalkContentParser::parse("image", json).unwrap();
        assert!(matches!(content, DingTalkContent::Image(_)));
        assert!(DingTalkContentParser::extract_text(&content).contains("img_12345"));
    }

    #[test]
    fn test_parse_file_content() {
        let json = serde_json::json!({
            "downloadCode": "file_12345",
            "fileName": "document.pdf"
        });
        let content = DingTalkContentParser::parse("file", json).unwrap();
        assert!(matches!(content, DingTalkContent::File(_)));
        assert!(DingTalkContentParser::extract_text(&content).contains("document.pdf"));
    }

    #[test]
    fn test_parse_action_card_content() {
        let json = serde_json::json!({
            "title": "Card Title",
            "markdown": "Card content",
            "singleTitle": "Click me",
            "singleURL": "https://example.com"
        });
        let content = DingTalkContentParser::parse("action_card", json).unwrap();
        assert!(matches!(content, DingTalkContent::ActionCard(_)));
        assert!(DingTalkContentParser::has_actions(&content));
    }

    #[test]
    fn test_create_content_helpers() {
        let text = DingTalkContentParser::create_text("Test message");
        assert!(
            matches!(text, DingTalkContent::Text(DingTalkTextContent { content }) if content == "Test message")
        );

        let markdown = DingTalkContentParser::create_markdown("Title", "Content");
        assert!(matches!(markdown, DingTalkContent::Markdown(_)));

        let image = DingTalkContentParser::create_image("https://example.com/image.jpg");
        assert!(
            matches!(image, DingTalkContent::Image(DingTalkImageContent { pic_url: Some(url), .. }) if url == "https://example.com/image.jpg")
        );

        let link = DingTalkContentParser::create_link("Title", "Text", "https://example.com");
        assert!(matches!(link, DingTalkContent::Link(_)));
    }

    #[test]
    fn test_get_download_code() {
        let voice = DingTalkContent::Voice(DingTalkVoiceContent {
            download_code: "voice_123".to_string(),
            duration: Some(5000),
            recognition: None,
        });
        assert_eq!(
            DingTalkContentParser::get_download_code(&voice),
            Some("voice_123".to_string())
        );

        let text = DingTalkContent::Text(DingTalkTextContent::default());
        assert_eq!(DingTalkContentParser::get_download_code(&text), None);
    }

    #[test]
    fn test_get_msg_type() {
        let text = DingTalkContent::Text(DingTalkTextContent::default());
        assert_eq!(DingTalkContentParser::get_msg_type(&text), "text");

        let image = DingTalkContent::Image(DingTalkImageContent::default());
        assert_eq!(DingTalkContentParser::get_msg_type(&image), "image");

        let file = DingTalkContent::File(DingTalkFileContent::default());
        assert_eq!(DingTalkContentParser::get_msg_type(&file), "file");
    }

    #[test]
    fn test_unknown_msg_type() {
        let json = serde_json::json!({});
        let result = DingTalkContentParser::parse("unknown", json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_content() {
        let content = DingTalkContentParser::create_text("Test");
        let json = DingTalkContentParser::to_json(&content).unwrap();
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_parse_from_payload() {
        let payload = r#"{
            "msgtype": "text",
            "text": {
                "content": "Hello from payload"
            }
        }"#;
        let content = DingTalkContentParser::parse_from_payload(payload).unwrap();
        assert!(matches!(content, DingTalkContent::Text(_)));
        assert_eq!(
            DingTalkContentParser::extract_text(&content),
            "Hello from payload"
        );
    }
}

// =============================================================================
// 🟢 P0 FIX: PlatformContent trait implementation for unified content framework
// =============================================================================

impl PlatformContent for DingTalkContent {
    fn content_type(&self) -> UnifiedContentType {
        match self {
            DingTalkContent::Text(_) => UnifiedContentType::Text,
            DingTalkContent::Markdown(_) => UnifiedContentType::Rich,
            DingTalkContent::ActionCard(_) => UnifiedContentType::Card,
            DingTalkContent::Image(_) => UnifiedContentType::Image,
            DingTalkContent::Voice(_) => UnifiedContentType::Audio,
            DingTalkContent::File(_) => UnifiedContentType::File,
            DingTalkContent::Link(_) => UnifiedContentType::Rich,
            DingTalkContent::FeedCard(_) => UnifiedContentType::Rich,
        }
    }

    fn extract_text(&self) -> String {
        DingTalkContentParser::extract_text(self)
    }
}

impl DingTalkImageContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.pic_url.clone().unwrap_or_default(),
            mime_type: None,
            filename: None,
            size: None,
            width: None,
            height: None,
            duration: None,
            caption: None,
            thumbnail: self.download_code.clone(),
        }
    }
}

impl DingTalkVoiceContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.download_code.clone(),
            mime_type: Some("audio/amr".to_string()),
            filename: Some(format!("voice_{}.amr", self.download_code)),
            size: None,
            width: None,
            height: None,
            duration: self.duration,
            caption: self.recognition.clone(),
            thumbnail: None,
        }
    }
}

impl DingTalkFileContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.download_code.clone(),
            mime_type: None,
            filename: self.file_name.clone(),
            size: self.file_size,
            width: None,
            height: None,
            duration: None,
            caption: None,
            thumbnail: None,
        }
    }
}
