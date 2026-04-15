//! WeChat (微信/企业微信) Message Content Parser
//!
//! Provides parsing and handling of various WeChat message content types
//! including text, images, voice, video, location, links, and mini programs.

use serde::{Deserialize, Serialize};

use crate::communication::channel::content::{
    ContentType as UnifiedContentType, MediaContent, PlatformContent,
};
use crate::error::{AgentError, Result};

/// WeChat message content types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum WeChatContent {
    /// Plain text message
    #[serde(rename = "text")]
    Text(WeChatTextContent),
    /// Image message
    #[serde(rename = "image")]
    Image(WeChatImageContent),
    /// Voice message
    #[serde(rename = "voice")]
    Voice(WeChatVoiceContent),
    /// Video message
    #[serde(rename = "video")]
    Video(WeChatVideoContent),
    /// Short video (moment video)
    #[serde(rename = "shortvideo")]
    ShortVideo(WeChatVideoContent),
    /// Location message
    #[serde(rename = "location")]
    Location(WeChatLocationContent),
    /// Link/message sharing
    #[serde(rename = "link")]
    Link(WeChatLinkContent),
    /// Event message (subscribe, unsubscribe, etc.)
    #[serde(rename = "event")]
    Event(WeChatEventContent),
    /// Mini program card
    #[serde(rename = "miniprogrampage")]
    MiniProgram(WeChatMiniProgramContent),
    /// File message
    #[serde(rename = "file")]
    File(WeChatFileContent),
    /// Music message
    #[serde(rename = "music")]
    Music(WeChatMusicContent),
    /// News/article message
    #[serde(rename = "news")]
    News(WeChatNewsContent),
    /// Markdown message (企业微信)
    #[serde(rename = "markdown")]
    Markdown(WeChatMarkdownContent),
}

/// Text content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatTextContent {
    /// The text content
    pub content: String,
}

/// Image content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatImageContent {
    /// Image URL (temporary, valid for 3 days)
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>,
    /// Media ID
    #[serde(rename = "mediaId")]
    pub media_id: String,
    /// Image format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Voice content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatVoiceContent {
    /// Media ID
    #[serde(rename = "mediaId")]
    pub media_id: String,
    /// Voice format (amr, speex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Recognition result (if voice recognition is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recognition: Option<String>,
    /// Duration in milliseconds (企业微信)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
}

/// Video content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatVideoContent {
    /// Media ID
    #[serde(rename = "mediaId")]
    pub media_id: String,
    /// Video title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Video description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Thumbnail media ID
    #[serde(rename = "thumbMediaId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_media_id: Option<String>,
}

/// Location content structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WeChatLocationContent {
    /// Latitude
    #[serde(rename = "location_X")]
    pub location_x: f64,
    /// Longitude
    #[serde(rename = "location_Y")]
    pub location_y: f64,
    /// Scale (map zoom level)
    pub scale: i32,
    /// Location label/description
    pub label: String,
    /// POI name (if available)
    #[serde(rename = "poiname")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poi_name: Option<String>,
}

/// Link content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatLinkContent {
    /// Link title
    pub title: String,
    /// Link description
    pub description: String,
    /// Link URL
    pub url: String,
    /// Thumbnail URL
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>,
}

/// Event content structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeChatEventContent {
    /// Event type
    pub event: String,
    /// Event key (for menu clicks, QR code scans)
    #[serde(rename = "eventKey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_key: Option<String>,
    /// Ticket (for QR code scans)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,
    /// Latitude (for location reporting)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Longitude (for location reporting)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// Precision (for location reporting)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<f64>,
    /// Change type (for contact change events in 企业微信)
    #[serde(rename = "changeType")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_type: Option<String>,
    /// User ID (for contact change events)
    #[serde(rename = "userId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Department ID (for contact change events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
}

/// Mini program content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatMiniProgramContent {
    /// Mini program title
    pub title: String,
    /// Mini program appid
    #[serde(rename = "appid")]
    pub app_id: String,
    /// Mini program page path
    #[serde(rename = "pagepath")]
    pub page_path: String,
    /// Mini program thumbnail URL
    #[serde(rename = "thumbUrl")]
    pub thumb_url: Option<String>,
    /// Mini program thumbnail media ID
    #[serde(rename = "thumbMediaId")]
    pub thumb_media_id: String,
}

/// File content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatFileContent {
    /// File title/name
    pub title: String,
    /// File description
    pub description: String,
    /// File name
    #[serde(rename = "fileName")]
    pub file_name: String,
    /// File key (for download)
    #[serde(rename = "fileKey")]
    pub file_key: Option<String>,
    /// File MD5
    #[serde(rename = "fileMd5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_md5: Option<String>,
    /// File total length
    #[serde(rename = "fileTotalLen")]
    pub file_total_len: i64,
}

/// Music content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatMusicContent {
    /// Music title
    pub title: String,
    /// Music description
    pub description: String,
    /// Music URL
    #[serde(rename = "musicUrl")]
    pub music_url: String,
    /// High quality music URL
    #[serde(rename = "hqMusicUrl")]
    pub hq_music_url: Option<String>,
    /// Thumbnail media ID
    #[serde(rename = "thumbMediaId")]
    pub thumb_media_id: String,
}

/// News/article content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatNewsContent {
    /// Article count
    #[serde(rename = "articleCount")]
    pub article_count: i32,
    /// Articles
    pub articles: Vec<WeChatArticle>,
}

/// WeChat article
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeChatArticle {
    /// Article title
    pub title: String,
    /// Article description
    pub description: String,
    /// Article URL
    pub url: String,
    /// Thumbnail URL
    #[serde(rename = "picUrl")]
    pub pic_url: String,
}

/// Markdown content structure (企业微信)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WeChatMarkdownContent {
    /// Markdown content
    pub content: String,
}

/// WeChat content parser
#[derive(Debug, Clone, Default)]
pub struct WeChatContentParser;

impl WeChatContentParser {
    /// Create a new content parser
    pub fn new() -> Self {
        Self
    }

    /// Parse content from JSON value
    pub fn parse(msg_type: &str, content: serde_json::Value) -> Result<WeChatContent> {
        match msg_type {
            "text" => {
                let text_content: WeChatTextContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse text content: {}", e))
                    })?;
                Ok(WeChatContent::Text(text_content))
            }
            "image" => {
                let image_content: WeChatImageContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse image content: {}", e))
                    })?;
                Ok(WeChatContent::Image(image_content))
            }
            "voice" => {
                let voice_content: WeChatVoiceContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse voice content: {}", e))
                    })?;
                Ok(WeChatContent::Voice(voice_content))
            }
            "video" => {
                let video_content: WeChatVideoContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse video content: {}", e))
                    })?;
                Ok(WeChatContent::Video(video_content))
            }
            "shortvideo" => {
                let video_content: WeChatVideoContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse short video content: {}", e))
                    })?;
                Ok(WeChatContent::ShortVideo(video_content))
            }
            "location" => {
                let location_content: WeChatLocationContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse location content: {}", e))
                    })?;
                Ok(WeChatContent::Location(location_content))
            }
            "link" => {
                let link_content: WeChatLinkContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse link content: {}", e))
                    })?;
                Ok(WeChatContent::Link(link_content))
            }
            "event" => {
                let event_content: WeChatEventContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse event content: {}", e))
                    })?;
                Ok(WeChatContent::Event(event_content))
            }
            "miniprogrampage" => {
                let mini_program_content: WeChatMiniProgramContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse mini program content: {}", e))
                    })?;
                Ok(WeChatContent::MiniProgram(mini_program_content))
            }
            "file" => {
                let file_content: WeChatFileContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse file content: {}", e))
                    })?;
                Ok(WeChatContent::File(file_content))
            }
            "music" => {
                let music_content: WeChatMusicContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse music content: {}", e))
                    })?;
                Ok(WeChatContent::Music(music_content))
            }
            "news" => {
                let news_content: WeChatNewsContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse news content: {}", e))
                    })?;
                Ok(WeChatContent::News(news_content))
            }
            "markdown" => {
                let markdown_content: WeChatMarkdownContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse markdown content: {}", e))
                    })?;
                Ok(WeChatContent::Markdown(markdown_content))
            }
            _ => Err(AgentError::platform(format!("Unknown message type: {}", msg_type)).into()),
        }
    }

    /// Parse content from JSON string
    pub fn parse_str(msg_type: &str, content_json: &str) -> Result<WeChatContent> {
        let content: serde_json::Value = serde_json::from_str(content_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON: {}", e)))?;
        Self::parse(msg_type, content)
    }

    /// Parse from webhook payload JSON
    pub fn parse_from_payload(payload_json: &str) -> Result<WeChatContent> {
        let payload: serde_json::Value = serde_json::from_str(payload_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON payload: {}", e)))?;

        let msg_type = payload
            .get("msgtype")
            .and_then(|v| v.as_str())
            .unwrap_or("text");

        // Extract content based on message type
        let content = match msg_type {
            "text" => payload.get("text").cloned(),
            "image" => payload.get("image").cloned(),
            "voice" => payload.get("voice").cloned(),
            "video" => payload.get("video").cloned(),
            "location" => payload.get("location").cloned(),
            "link" => payload.get("link").cloned(),
            "file" => payload.get("file").cloned(),
            "news" => payload.get("news").cloned(),
            "markdown" => payload.get("markdown").cloned(),
            _ => None,
        };

        let content = content.unwrap_or_else(|| serde_json::json!({}));
        Self::parse(msg_type, content)
    }

    /// Extract plain text from any content type
    pub fn extract_text(content: &WeChatContent) -> String {
        match content {
            WeChatContent::Text(text) => text.content.clone(),
            WeChatContent::Image(_) => "[Image]".to_string(),
            WeChatContent::Voice(voice) => {
                if let Some(recognition) = &voice.recognition {
                    format!("[Voice: {}]", recognition)
                } else {
                    "[Voice message]".to_string()
                }
            }
            WeChatContent::Video(video) | WeChatContent::ShortVideo(video) => {
                format!(
                    "[Video: {}]",
                    video.title.as_deref().unwrap_or("(no title)")
                )
            }
            WeChatContent::Location(loc) => {
                format!(
                    "[Location: {} - {}, {}]",
                    loc.label, loc.location_x, loc.location_y
                )
            }
            WeChatContent::Link(link) => {
                format!("[Link: {} - {}]", link.title, link.url)
            }
            WeChatContent::Event(event) => {
                format!("[Event: {}]", event.event)
            }
            WeChatContent::MiniProgram(mp) => {
                format!("[Mini Program: {}]", mp.title)
            }
            WeChatContent::File(file) => {
                format!("[File: {}]", file.file_name)
            }
            WeChatContent::Music(music) => {
                format!("[Music: {}]", music.title)
            }
            WeChatContent::News(news) => {
                let titles: Vec<String> = news.articles.iter().map(|a| a.title.clone()).collect();
                format!("[News: {}]", titles.join(", "))
            }
            WeChatContent::Markdown(markdown) => {
                format!("[Markdown]\n{}", markdown.content)
            }
        }
    }

    /// Get message type string
    pub fn get_msg_type(content: &WeChatContent) -> &'static str {
        match content {
            WeChatContent::Text(_) => "text",
            WeChatContent::Image(_) => "image",
            WeChatContent::Voice(_) => "voice",
            WeChatContent::Video(_) => "video",
            WeChatContent::ShortVideo(_) => "shortvideo",
            WeChatContent::Location(_) => "location",
            WeChatContent::Link(_) => "link",
            WeChatContent::Event(_) => "event",
            WeChatContent::MiniProgram(_) => "miniprogrampage",
            WeChatContent::File(_) => "file",
            WeChatContent::Music(_) => "music",
            WeChatContent::News(_) => "news",
            WeChatContent::Markdown(_) => "markdown",
        }
    }

    /// Create text content
    pub fn create_text(content: impl Into<String>) -> WeChatContent {
        WeChatContent::Text(WeChatTextContent {
            content: content.into(),
        })
    }

    /// Create image content
    pub fn create_image(media_id: impl Into<String>) -> WeChatContent {
        WeChatContent::Image(WeChatImageContent {
            media_id: media_id.into(),
            pic_url: None,
            format: None,
        })
    }

    /// Create news/article content
    pub fn create_news(articles: Vec<WeChatArticle>) -> WeChatContent {
        WeChatContent::News(WeChatNewsContent {
            article_count: articles.len() as i32,
            articles,
        })
    }

    /// Create markdown content (企业微信)
    pub fn create_markdown(content: impl Into<String>) -> WeChatContent {
        WeChatContent::Markdown(WeChatMarkdownContent {
            content: content.into(),
        })
    }

    /// Serialize content to JSON string
    pub fn to_json(content: &WeChatContent) -> Result<String> {
        serde_json::to_string(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Serialize content to JSON value
    pub fn to_json_value(content: &WeChatContent) -> Result<serde_json::Value> {
        serde_json::to_value(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)).into())
    }

    /// Check if content is an event
    pub fn is_event(content: &WeChatContent) -> bool {
        matches!(content, WeChatContent::Event(_))
    }

    /// Get event type (if content is an event)
    pub fn get_event_type(content: &WeChatContent) -> Option<&str> {
        match content {
            WeChatContent::Event(event) => Some(&event.event),
            _ => None,
        }
    }

    /// Get media ID for media content types
    pub fn get_media_id(content: &WeChatContent) -> Option<String> {
        match content {
            WeChatContent::Image(img) => Some(img.media_id.clone()),
            WeChatContent::Voice(voice) => Some(voice.media_id.clone()),
            WeChatContent::Video(video) | WeChatContent::ShortVideo(video) => {
                Some(video.media_id.clone())
            }
            _ => None,
        }
    }

    /// Get download code for file content
    pub fn get_file_key(content: &WeChatContent) -> Option<String> {
        match content {
            WeChatContent::File(file) => file.file_key.clone(),
            _ => None,
        }
    }

    /// Get file name for file content
    pub fn get_file_name(content: &WeChatContent) -> Option<String> {
        match content {
            WeChatContent::File(file) => Some(file.file_name.clone()),
            _ => None,
        }
    }
}

/// WeChat XML message parser (for webhook payloads)
pub struct WeChatXmlParser;

impl WeChatXmlParser {
    /// Parse XML message to content
    ///
    /// Note: This is a placeholder. Actual implementation would use an XML
    /// parser.
    pub fn parse(xml: &str) -> Result<(String, WeChatContent)> {
        // Extract message type from XML
        let msg_type =
            Self::extract_xml_value(xml, "MsgType").unwrap_or_else(|| "text".to_string());

        // Extract content based on message type
        let content = match msg_type.as_str() {
            "text" => {
                let text = Self::extract_xml_value(xml, "Content").unwrap_or_default();
                WeChatContent::Text(WeChatTextContent { content: text })
            }
            "image" => WeChatContent::Image(WeChatImageContent {
                media_id: Self::extract_xml_value(xml, "MediaId").unwrap_or_default(),
                pic_url: Self::extract_xml_value(xml, "PicUrl"),
                format: None,
            }),
            "voice" => WeChatContent::Voice(WeChatVoiceContent {
                media_id: Self::extract_xml_value(xml, "MediaId").unwrap_or_default(),
                format: Self::extract_xml_value(xml, "Format"),
                recognition: Self::extract_xml_value(xml, "Recognition"),
                duration: None,
            }),
            "video" | "shortvideo" => WeChatContent::Video(WeChatVideoContent {
                media_id: Self::extract_xml_value(xml, "MediaId").unwrap_or_default(),
                title: Self::extract_xml_value(xml, "Title"),
                description: Self::extract_xml_value(xml, "Description"),
                thumb_media_id: Self::extract_xml_value(xml, "ThumbMediaId"),
            }),
            "location" => WeChatContent::Location(WeChatLocationContent {
                location_x: Self::extract_xml_value(xml, "Location_X")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
                location_y: Self::extract_xml_value(xml, "Location_Y")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.0),
                scale: Self::extract_xml_value(xml, "Scale")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                label: Self::extract_xml_value(xml, "Label").unwrap_or_default(),
                poi_name: None,
            }),
            "link" => WeChatContent::Link(WeChatLinkContent {
                title: Self::extract_xml_value(xml, "Title").unwrap_or_default(),
                description: Self::extract_xml_value(xml, "Description").unwrap_or_default(),
                url: Self::extract_xml_value(xml, "Url").unwrap_or_default(),
                pic_url: Self::extract_xml_value(xml, "PicUrl"),
            }),
            "event" => WeChatContent::Event(WeChatEventContent {
                event: Self::extract_xml_value(xml, "Event").unwrap_or_default(),
                event_key: Self::extract_xml_value(xml, "EventKey"),
                ticket: Self::extract_xml_value(xml, "Ticket"),
                latitude: Self::extract_xml_value(xml, "Latitude").and_then(|s| s.parse().ok()),
                longitude: Self::extract_xml_value(xml, "Longitude").and_then(|s| s.parse().ok()),
                precision: Self::extract_xml_value(xml, "Precision").and_then(|s| s.parse().ok()),
                change_type: Self::extract_xml_value(xml, "ChangeType"),
                user_id: Self::extract_xml_value(xml, "UserID"),
                department: Self::extract_xml_value(xml, "Department"),
            }),
            "file" => WeChatContent::File(WeChatFileContent {
                title: Self::extract_xml_value(xml, "Title").unwrap_or_default(),
                description: String::new(),
                file_name: Self::extract_xml_value(xml, "FileName").unwrap_or_default(),
                file_key: Self::extract_xml_value(xml, "FileKey"),
                file_md5: Self::extract_xml_value(xml, "FileMd5"),
                file_total_len: Self::extract_xml_value(xml, "FileTotalLen")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            }),
            _ => WeChatContent::Text(WeChatTextContent {
                content: "Unknown message type".to_string(),
            }),
        };

        Ok((msg_type, content))
    }

    /// Extract value from XML tag
    fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
        // Try CDATA format first
        let pattern = format!(r"<{}><!\[CDATA\[(.*?)\]\]></{}>", tag, tag);
        let regex = regex::Regex::new(&pattern).ok()?;
        if let Some(captures) = regex.captures(xml) {
            return captures.get(1).map(|m| m.as_str().to_string());
        }

        // Try without CDATA
        let pattern = format!("<{}>(.*?)</{}>", tag, tag);
        let regex = regex::Regex::new(&pattern).ok()?;
        regex.captures(xml)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_content() {
        let json = serde_json::json!({
            "content": "Hello, WeChat!"
        });
        let content = WeChatContentParser::parse("text", json).unwrap();
        assert!(matches!(content, WeChatContent::Text(_)));
        assert_eq!(
            WeChatContentParser::extract_text(&content),
            "Hello, WeChat!"
        );
    }

    #[test]
    fn test_parse_image_content() {
        let json = serde_json::json!({
            "mediaId": "media_id_123",
            "picUrl": "http://example.com/image.jpg"
        });
        let content = WeChatContentParser::parse("image", json).unwrap();
        assert!(matches!(content, WeChatContent::Image(_)));
        assert_eq!(WeChatContentParser::extract_text(&content), "[Image]");
    }

    #[test]
    fn test_parse_location_content() {
        let json = serde_json::json!({
            "location_X": 39.9042,
            "location_Y": 116.4074,
            "scale": 15,
            "label": "Beijing"
        });
        let content = WeChatContentParser::parse("location", json).unwrap();
        assert!(matches!(content, WeChatContent::Location(_)));
        let text = WeChatContentParser::extract_text(&content);
        assert!(text.contains("Beijing"));
        assert!(text.contains("39.9042"));
    }

    #[test]
    fn test_parse_event_content() {
        let json = serde_json::json!({
            "event": "subscribe"
        });
        let content = WeChatContentParser::parse("event", json).unwrap();
        assert!(matches!(content, WeChatContent::Event(_)));
        assert!(WeChatContentParser::is_event(&content));
        assert_eq!(
            WeChatContentParser::get_event_type(&content),
            Some("subscribe")
        );
    }

    #[test]
    fn test_create_text() {
        let content = WeChatContentParser::create_text("Test message");
        assert!(
            matches!(content, WeChatContent::Text(WeChatTextContent { content: c }) if c == "Test message")
        );
    }

    #[test]
    fn test_get_msg_type() {
        let text = WeChatContent::Text(WeChatTextContent::default());
        assert_eq!(WeChatContentParser::get_msg_type(&text), "text");

        let image = WeChatContent::Image(WeChatImageContent::default());
        assert_eq!(WeChatContentParser::get_msg_type(&image), "image");

        let markdown = WeChatContent::Markdown(WeChatMarkdownContent::default());
        assert_eq!(WeChatContentParser::get_msg_type(&markdown), "markdown");
    }

    #[test]
    fn test_parse_from_payload() {
        let payload = r#"{
            "msgtype": "text",
            "text": {
                "content": "Hello from payload"
            }
        }"#;
        let content = WeChatContentParser::parse_from_payload(payload).unwrap();
        assert!(matches!(content, WeChatContent::Text(_)));
        assert_eq!(
            WeChatContentParser::extract_text(&content),
            "Hello from payload"
        );
    }

    #[test]
    fn test_get_media_id() {
        let image = WeChatContent::Image(WeChatImageContent {
            media_id: "media_123".to_string(),
            pic_url: None,
            format: None,
        });
        assert_eq!(
            WeChatContentParser::get_media_id(&image),
            Some("media_123".to_string())
        );

        let text = WeChatContent::Text(WeChatTextContent::default());
        assert_eq!(WeChatContentParser::get_media_id(&text), None);
    }

    #[test]
    fn test_xml_parser() {
        let xml = r#"<xml>
            <ToUserName><![CDATA[corp_id]]></ToUserName>
            <FromUserName><![CDATA[user123]]></FromUserName>
            <MsgType><![CDATA[text]]></MsgType>
            <Content><![CDATA[Hello World]]></Content>
        </xml>"#;

        let (msg_type, content) = WeChatXmlParser::parse(xml).unwrap();
        assert_eq!(msg_type, "text");
        assert!(matches!(content, WeChatContent::Text(text) if text.content == "Hello World"));
    }
}

// =============================================================================
// 🟢 P0 FIX: PlatformContent trait implementation for unified content framework
// =============================================================================

impl PlatformContent for WeChatContent {
    fn content_type(&self) -> UnifiedContentType {
        match self {
            WeChatContent::Text(_) => UnifiedContentType::Text,
            WeChatContent::Image(_) => UnifiedContentType::Image,
            WeChatContent::Voice(_) => UnifiedContentType::Audio,
            WeChatContent::Video(_) => UnifiedContentType::Video,
            WeChatContent::ShortVideo(_) => UnifiedContentType::Video,
            WeChatContent::Location(_) => UnifiedContentType::Location,
            WeChatContent::Link(_) => UnifiedContentType::Rich,
            WeChatContent::Event(_) => UnifiedContentType::System,
            WeChatContent::MiniProgram(_) => UnifiedContentType::Card,
            WeChatContent::File(_) => UnifiedContentType::File,
            WeChatContent::Music(_) => UnifiedContentType::Audio,
            WeChatContent::News(_) => UnifiedContentType::Rich,
            WeChatContent::Markdown(_) => UnifiedContentType::Text,
        }
    }

    fn extract_text(&self) -> String {
        WeChatContentParser::extract_text(self)
    }
}

impl WeChatImageContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.pic_url.clone().unwrap_or_default(),
            mime_type: self.format.clone(),
            filename: None,
            size: None,
            width: None,
            height: None,
            duration: None,
            caption: None,
            thumbnail: Some(self.media_id.clone()),
        }
    }
}

impl WeChatVideoContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.media_id.clone(),
            mime_type: None,
            filename: self.title.clone(),
            size: None,
            width: None,
            height: None,
            duration: None,
            caption: self.description.clone(),
            thumbnail: self.thumb_media_id.clone(),
        }
    }
}

impl WeChatFileContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.file_key.clone().unwrap_or_default(),
            mime_type: None,
            filename: Some(self.file_name.clone()),
            size: Some(self.file_total_len),
            width: None,
            height: None,
            duration: None,
            caption: Some(self.description.clone()),
            thumbnail: None,
        }
    }
}
