//! Signal Message Content Parser
//!
//! Provides parsing and handling of various Signal message content types
//! including text, media, documents, locations, contacts, reactions, and typing
//! indicators.

use serde::{Deserialize, Serialize};

use crate::communication::channel::content::{
    ContentType as UnifiedContentType, MediaContent, PlatformContent,
};
use crate::error::{AgentError, Result};

/// Signal message content types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum SignalContent {
    /// Plain text message
    #[serde(rename = "text")]
    Text(SignalTextContent),
    /// Image message
    #[serde(rename = "image")]
    Image(SignalImageContent),
    /// Video message
    #[serde(rename = "video")]
    Video(SignalVideoContent),
    /// Audio message
    #[serde(rename = "audio")]
    Audio(SignalAudioContent),
    /// Voice message
    #[serde(rename = "voice")]
    Voice(SignalVoiceContent),
    /// Document/file message
    #[serde(rename = "file")]
    File(SignalFileContent),
    /// Location message
    #[serde(rename = "location")]
    Location(SignalLocationContent),
    /// Contact message
    #[serde(rename = "contact")]
    Contact(SignalContactContent),
    /// Sticker message
    #[serde(rename = "sticker")]
    Sticker(SignalStickerContent),
    /// Reaction message
    #[serde(rename = "reaction")]
    Reaction(SignalReactionContent),
    /// Typing indicator
    #[serde(rename = "typing")]
    Typing(SignalTypingContent),
    /// Remote delete (message deletion)
    #[serde(rename = "remote_delete")]
    RemoteDelete(SignalRemoteDeleteContent),
    /// Group info update
    #[serde(rename = "group_info")]
    GroupInfo(SignalGroupInfoContent),
}

/// Text content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalTextContent {
    /// The text content
    pub body: String,
    /// Whether this message is a reply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Box<SignalQuoteContent>>,
    /// Mentions in the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<SignalMention>>,
    /// Expiration timer (disappearing messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_seconds: Option<i32>,
}

/// Quote content (reply to message)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalQuoteContent {
    /// Original message ID (timestamp)
    pub id: i64,
    /// Original message author
    pub author: String,
    /// Original message text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Original message attachments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<SignalAttachmentInfo>>,
}

/// Mention in message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalMention {
    /// Display name
    pub name: String,
    /// Phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Start position in text
    pub start: i32,
    /// Length of mention
    pub length: i32,
}

/// Image content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalImageContent {
    /// Attachment ID
    pub attachment_id: String,
    /// Local file path (after download)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Image caption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Image width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// Image height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    /// Whether image is a view-once message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_once: Option<bool>,
}

/// Video content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalVideoContent {
    /// Attachment ID
    pub attachment_id: String,
    /// Local file path (after download)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Video caption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Video width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    /// Video height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    /// Whether video is a view-once message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_once: Option<bool>,
}

/// Audio content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalAudioContent {
    /// Attachment ID
    pub attachment_id: String,
    /// Local file path (after download)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
}

/// Voice content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalVoiceContent {
    /// Attachment ID
    pub attachment_id: String,
    /// Local file path (after download)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
}

/// File content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalFileContent {
    /// Attachment ID
    pub attachment_id: String,
    /// Local file path (after download)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// File name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// File size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    /// File caption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

/// Location content structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SignalLocationContent {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Location name (if shared from a place)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Location address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// Contact content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalContactContent {
    /// Contact name
    pub name: String,
    /// Phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    /// vCard data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcard: Option<String>,
}

/// Sticker content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalStickerContent {
    /// Sticker ID
    pub sticker_id: String,
    /// Pack ID
    pub pack_id: String,
    /// Associated emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
}

/// Reaction content structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalReactionContent {
    /// Emoji reaction
    pub emoji: String,
    /// Target message author
    pub target_author: String,
    /// Target message timestamp
    pub target_timestamp: i64,
    /// Whether this is a removal
    pub is_remove: bool,
}

/// Typing indicator content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalTypingContent {
    /// Action: "started" or "stopped"
    pub action: String,
    /// Group ID (if in group)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
}

/// Remote delete content (message deletion)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalRemoteDeleteContent {
    /// Target message timestamp
    pub target_timestamp: i64,
}

/// Group info content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalGroupInfoContent {
    /// Group ID
    pub group_id: String,
    /// Group name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    /// Revision number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<i32>,
    /// Update type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_type: Option<String>,
}

/// Attachment info for parsing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalAttachmentInfo {
    pub id: String,
    pub content_type: String,
    pub filename: String,
    pub size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_note: Option<bool>,
}

/// Signal content parser
#[derive(Debug, Clone, Default)]
pub struct SignalContentParser;

impl SignalContentParser {
    /// Create a new content parser
    pub fn new() -> Self {
        Self
    }

    /// Parse content from JSON value
    pub fn parse(content_type: &str, content: serde_json::Value) -> Result<SignalContent> {
        match content_type {
            "text" => {
                let text_content: SignalTextContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse text content: {}", e))
                    })?;
                Ok(SignalContent::Text(text_content))
            }
            "image" => {
                let image_content: SignalImageContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse image content: {}", e))
                    })?;
                Ok(SignalContent::Image(image_content))
            }
            "video" => {
                let video_content: SignalVideoContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse video content: {}", e))
                    })?;
                Ok(SignalContent::Video(video_content))
            }
            "audio" => {
                let audio_content: SignalAudioContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse audio content: {}", e))
                    })?;
                Ok(SignalContent::Audio(audio_content))
            }
            "voice" => {
                let voice_content: SignalVoiceContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse voice content: {}", e))
                    })?;
                Ok(SignalContent::Voice(voice_content))
            }
            "file" | "document" => {
                let file_content: SignalFileContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse file content: {}", e))
                    })?;
                Ok(SignalContent::File(file_content))
            }
            "location" => {
                let location_content: SignalLocationContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse location content: {}", e))
                    })?;
                Ok(SignalContent::Location(location_content))
            }
            "contact" => {
                let contact_content: SignalContactContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse contact content: {}", e))
                    })?;
                Ok(SignalContent::Contact(contact_content))
            }
            "sticker" => {
                let sticker_content: SignalStickerContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse sticker content: {}", e))
                    })?;
                Ok(SignalContent::Sticker(sticker_content))
            }
            "reaction" => {
                let reaction_content: SignalReactionContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse reaction content: {}", e))
                    })?;
                Ok(SignalContent::Reaction(reaction_content))
            }
            "typing" => {
                let typing_content: SignalTypingContent =
                    serde_json::from_value(content).map_err(|e| {
                        AgentError::platform(format!("Failed to parse typing content: {}", e))
                    })?;
                Ok(SignalContent::Typing(typing_content))
            }
            "remote_delete" => {
                let delete_content: SignalRemoteDeleteContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!(
                            "Failed to parse remote delete content: {}",
                            e
                        ))
                    })?;
                Ok(SignalContent::RemoteDelete(delete_content))
            }
            "group_info" => {
                let group_content: SignalGroupInfoContent = serde_json::from_value(content)
                    .map_err(|e| {
                        AgentError::platform(format!("Failed to parse group info content: {}", e))
                    })?;
                Ok(SignalContent::GroupInfo(group_content))
            }
            _ => Err(AgentError::platform(format!(
                "Unknown content type: {}",
                content_type
            ))),
        }
    }

    /// Parse content from JSON string
    pub fn parse_str(content_type: &str, content_json: &str) -> Result<SignalContent> {
        let content: serde_json::Value = serde_json::from_str(content_json)
            .map_err(|e| AgentError::platform(format!("Invalid JSON: {}", e)))?;
        Self::parse(content_type, content)
    }

    /// Extract plain text from any content type
    pub fn extract_text(content: &SignalContent) -> String {
        match content {
            SignalContent::Text(text) => text.body.clone(),
            SignalContent::Image(image) => {
                format!("[Image] {}", image.caption.as_deref().unwrap_or(""))
            }
            SignalContent::Video(video) => {
                format!("[Video] {}", video.caption.as_deref().unwrap_or(""))
            }
            SignalContent::Audio(_) => "[Audio]".to_string(),
            SignalContent::Voice(voice) => {
                format!(
                    "[Voice message, {}s]",
                    voice.duration.map(|d| d.to_string()).unwrap_or_default()
                )
            }
            SignalContent::File(file) => {
                format!(
                    "[File: {}] {}",
                    file.filename.as_deref().unwrap_or("unnamed"),
                    file.caption.as_deref().unwrap_or("")
                )
            }
            SignalContent::Location(loc) => {
                format!("[Location: {}, {}]", loc.latitude, loc.longitude)
            }
            SignalContent::Contact(contact) => {
                format!("[Contact: {}]", contact.name)
            }
            SignalContent::Sticker(sticker) => {
                format!("[Sticker: {}]", sticker.emoji.as_deref().unwrap_or(""))
            }
            SignalContent::Reaction(reaction) => {
                format!(
                    "[Reaction: {} to message from {}]",
                    reaction.emoji, reaction.target_author
                )
            }
            SignalContent::Typing(typing) => {
                format!("[Typing: {}]", typing.action)
            }
            SignalContent::RemoteDelete(_) => "[Message deleted]".to_string(),
            SignalContent::GroupInfo(group) => {
                format!(
                    "[Group: {}]",
                    group.group_name.as_deref().unwrap_or(&group.group_id)
                )
            }
        }
    }

    /// Get content type string
    pub fn get_content_type(content: &SignalContent) -> &'static str {
        match content {
            SignalContent::Text(_) => "text",
            SignalContent::Image(_) => "image",
            SignalContent::Video(_) => "video",
            SignalContent::Audio(_) => "audio",
            SignalContent::Voice(_) => "voice",
            SignalContent::File(_) => "file",
            SignalContent::Location(_) => "location",
            SignalContent::Contact(_) => "contact",
            SignalContent::Sticker(_) => "sticker",
            SignalContent::Reaction(_) => "reaction",
            SignalContent::Typing(_) => "typing",
            SignalContent::RemoteDelete(_) => "remote_delete",
            SignalContent::GroupInfo(_) => "group_info",
        }
    }

    /// Create text content
    pub fn create_text(body: impl Into<String>) -> SignalContent {
        SignalContent::Text(SignalTextContent {
            body: body.into(),
            quote: None,
            mentions: None,
            expires_in_seconds: None,
        })
    }

    /// Create image content
    pub fn create_image(
        attachment_id: impl Into<String>,
        caption: Option<String>,
    ) -> SignalContent {
        SignalContent::Image(SignalImageContent {
            attachment_id: attachment_id.into(),
            file_path: None,
            caption,
            mime_type: None,
            width: None,
            height: None,
            file_size: None,
            view_once: None,
        })
    }

    /// Create video content
    pub fn create_video(
        attachment_id: impl Into<String>,
        caption: Option<String>,
    ) -> SignalContent {
        SignalContent::Video(SignalVideoContent {
            attachment_id: attachment_id.into(),
            file_path: None,
            caption,
            mime_type: None,
            width: None,
            height: None,
            duration: None,
            file_size: None,
            view_once: None,
        })
    }

    /// Create file content
    pub fn create_file(
        attachment_id: impl Into<String>,
        filename: Option<String>,
        caption: Option<String>,
    ) -> SignalContent {
        SignalContent::File(SignalFileContent {
            attachment_id: attachment_id.into(),
            file_path: None,
            filename,
            mime_type: None,
            file_size: None,
            caption,
        })
    }

    /// Create voice content
    pub fn create_voice(attachment_id: impl Into<String>) -> SignalContent {
        SignalContent::Voice(SignalVoiceContent {
            attachment_id: attachment_id.into(),
            file_path: None,
            mime_type: None,
            duration: None,
            file_size: None,
        })
    }

    /// Create location content
    pub fn create_location(latitude: f64, longitude: f64, name: Option<String>) -> SignalContent {
        SignalContent::Location(SignalLocationContent {
            latitude,
            longitude,
            name,
            address: None,
        })
    }

    /// Create reaction content
    pub fn create_reaction(
        emoji: impl Into<String>,
        target_author: impl Into<String>,
        target_timestamp: i64,
    ) -> SignalContent {
        SignalContent::Reaction(SignalReactionContent {
            emoji: emoji.into(),
            target_author: target_author.into(),
            target_timestamp,
            is_remove: false,
        })
    }

    /// Create typing content
    pub fn create_typing(action: impl Into<String>, group_id: Option<String>) -> SignalContent {
        SignalContent::Typing(SignalTypingContent {
            action: action.into(),
            group_id,
        })
    }

    /// Serialize content to JSON string
    pub fn to_json(content: &SignalContent) -> Result<String> {
        serde_json::to_string(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)))
    }

    /// Serialize content to JSON value
    pub fn to_json_value(content: &SignalContent) -> Result<serde_json::Value> {
        serde_json::to_value(content)
            .map_err(|e| AgentError::platform(format!("Failed to serialize content: {}", e)))
    }

    /// Check if content has media
    pub fn has_media(content: &SignalContent) -> bool {
        matches!(
            content,
            SignalContent::Image(_)
                | SignalContent::Video(_)
                | SignalContent::Audio(_)
                | SignalContent::Voice(_)
                | SignalContent::File(_)
                | SignalContent::Sticker(_)
        )
    }

    /// Get attachment ID if available
    pub fn get_attachment_id(content: &SignalContent) -> Option<&str> {
        match content {
            SignalContent::Image(img) => Some(&img.attachment_id),
            SignalContent::Video(vid) => Some(&vid.attachment_id),
            SignalContent::Audio(aud) => Some(&aud.attachment_id),
            SignalContent::Voice(voice) => Some(&voice.attachment_id),
            SignalContent::File(file) => Some(&file.attachment_id),
            _ => None,
        }
    }

    /// Check if content is a reaction
    pub fn is_reaction(content: &SignalContent) -> bool {
        matches!(content, SignalContent::Reaction(_))
    }

    /// Check if content is a typing indicator
    pub fn is_typing(content: &SignalContent) -> bool {
        matches!(content, SignalContent::Typing(_))
    }

    /// Check if content is a remote delete
    pub fn is_remote_delete(content: &SignalContent) -> bool {
        matches!(content, SignalContent::RemoteDelete(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_content() {
        let json = serde_json::json!({
            "body": "Hello, Signal!"
        });
        let content = SignalContentParser::parse("text", json).unwrap();
        assert!(matches!(content, SignalContent::Text(_)));
        assert_eq!(
            SignalContentParser::extract_text(&content),
            "Hello, Signal!"
        );
    }

    #[test]
    fn test_parse_image_content() {
        let json = serde_json::json!({
            "attachment_id": "abc123",
            "caption": "My photo"
        });
        let content = SignalContentParser::parse("image", json).unwrap();
        assert!(matches!(content, SignalContent::Image(_)));
        let text = SignalContentParser::extract_text(&content);
        assert!(text.contains("[Image]"));
        assert!(text.contains("My photo"));
    }

    #[test]
    fn test_parse_location_content() {
        let json = serde_json::json!({
            "latitude": 37.7749,
            "longitude": -122.4194,
            "name": "San Francisco"
        });
        let content = SignalContentParser::parse("location", json).unwrap();
        assert!(matches!(content, SignalContent::Location(_)));
        let text = SignalContentParser::extract_text(&content);
        assert!(text.contains("37.7749"));
        assert!(text.contains("-122.4194"));
    }

    #[test]
    fn test_parse_reaction_content() {
        let json = serde_json::json!({
            "emoji": "👍",
            "target_author": "+1234567890",
            "target_timestamp": 1234567890,
            "is_remove": false
        });
        let content = SignalContentParser::parse("reaction", json).unwrap();
        assert!(matches!(content, SignalContent::Reaction(_)));
        assert!(SignalContentParser::is_reaction(&content));
        let text = SignalContentParser::extract_text(&content);
        assert!(text.contains("👍"));
    }

    #[test]
    fn test_parse_typing_content() {
        let json = serde_json::json!({
            "action": "started"
        });
        let content = SignalContentParser::parse("typing", json).unwrap();
        assert!(matches!(content, SignalContent::Typing(_)));
        assert!(SignalContentParser::is_typing(&content));
    }

    #[test]
    fn test_create_text() {
        let content = SignalContentParser::create_text("Test message");
        assert!(
            matches!(content, SignalContent::Text(SignalTextContent { body, .. }) if body == "Test message")
        );
    }

    #[test]
    fn test_create_location() {
        let content =
            SignalContentParser::create_location(37.7749, -122.4194, Some("SF".to_string()));
        assert!(matches!(content, SignalContent::Location(_)));
        if let SignalContent::Location(loc) = content {
            assert_eq!(loc.latitude, 37.7749);
            assert_eq!(loc.longitude, -122.4194);
            assert_eq!(loc.name, Some("SF".to_string()));
        }
    }

    #[test]
    fn test_create_reaction() {
        let content = SignalContentParser::create_reaction("❤️", "+1234567890", 1234567890);
        assert!(matches!(content, SignalContent::Reaction(_)));
        if let SignalContent::Reaction(reaction) = content {
            assert_eq!(reaction.emoji, "❤️");
            assert_eq!(reaction.target_author, "+1234567890");
            assert_eq!(reaction.target_timestamp, 1234567890);
        }
    }

    #[test]
    fn test_get_content_type() {
        let text = SignalContent::Text(SignalTextContent::default());
        assert_eq!(SignalContentParser::get_content_type(&text), "text");

        let image = SignalContent::Image(SignalImageContent::default());
        assert_eq!(SignalContentParser::get_content_type(&image), "image");

        let voice = SignalContent::Voice(SignalVoiceContent::default());
        assert_eq!(SignalContentParser::get_content_type(&voice), "voice");

        let reaction = SignalContentParser::create_reaction("👍", "+123", 123);
        assert_eq!(SignalContentParser::get_content_type(&reaction), "reaction");
    }

    #[test]
    fn test_has_media() {
        let text = SignalContent::Text(SignalTextContent::default());
        assert!(!SignalContentParser::has_media(&text));

        let image = SignalContent::Image(SignalImageContent::default());
        assert!(SignalContentParser::has_media(&image));

        let voice = SignalContent::Voice(SignalVoiceContent::default());
        assert!(SignalContentParser::has_media(&voice));

        let reaction = SignalContentParser::create_reaction("👍", "+123", 123);
        assert!(!SignalContentParser::has_media(&reaction));
    }

    #[test]
    fn test_get_attachment_id() {
        let image = SignalContent::Image(SignalImageContent {
            attachment_id: "abc123".to_string(),
            ..Default::default()
        });
        assert_eq!(
            SignalContentParser::get_attachment_id(&image),
            Some("abc123")
        );

        let text = SignalContent::Text(SignalTextContent::default());
        assert_eq!(SignalContentParser::get_attachment_id(&text), None);
    }
}

// =============================================================================
// 🟢 P0 FIX: PlatformContent trait implementation for unified content framework
// =============================================================================

impl PlatformContent for SignalContent {
    fn content_type(&self) -> UnifiedContentType {
        match self {
            SignalContent::Text(_) => UnifiedContentType::Text,
            SignalContent::Image(_) => UnifiedContentType::Image,
            SignalContent::Video(_) => UnifiedContentType::Video,
            SignalContent::Audio(_) => UnifiedContentType::Audio,
            SignalContent::Voice(_) => UnifiedContentType::Audio,
            SignalContent::File(_) => UnifiedContentType::File,
            SignalContent::Location(_) => UnifiedContentType::Location,
            SignalContent::Contact(_) => UnifiedContentType::Contact,
            SignalContent::Sticker(_) => UnifiedContentType::Sticker,
            SignalContent::Reaction(_) => UnifiedContentType::System,
            SignalContent::Typing(_) => UnifiedContentType::System,
            SignalContent::RemoteDelete(_) => UnifiedContentType::System,
            SignalContent::GroupInfo(_) => UnifiedContentType::System,
        }
    }

    fn extract_text(&self) -> String {
        SignalContentParser::extract_text(self)
    }
}

impl SignalImageContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.file_path.clone().unwrap_or_default(),
            width: self.width,
            height: self.height,
            size: self.file_size,
            mime_type: self.mime_type.clone(),
            filename: None,
            caption: self.caption.clone(),
            thumbnail: None,
            duration: None,
        }
    }
}

impl SignalVideoContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.file_path.clone().unwrap_or_default(),
            width: self.width,
            height: self.height,
            duration: self.duration,
            mime_type: self.mime_type.clone(),
            filename: None,
            caption: self.caption.clone(),
            size: self.file_size,
            thumbnail: None,
        }
    }
}

impl SignalFileContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.file_path.clone().unwrap_or_default(),
            mime_type: self.mime_type.clone(),
            filename: self.filename.clone(),
            size: self.file_size,
            width: None,
            height: None,
            duration: None,
            caption: self.caption.clone(),
            thumbnail: None,
        }
    }
}

impl SignalAudioContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.file_path.clone().unwrap_or_default(),
            mime_type: self.mime_type.clone(),
            filename: None,
            size: self.file_size,
            duration: self.duration,
            width: None,
            height: None,
            caption: None,
            thumbnail: None,
        }
    }
}

impl SignalVoiceContent {
    /// Convert to unified MediaContent
    pub fn to_media_content(&self) -> MediaContent {
        MediaContent {
            url: self.file_path.clone().unwrap_or_default(),
            mime_type: self.mime_type.clone(),
            filename: None,
            size: self.file_size,
            duration: self.duration,
            width: None,
            height: None,
            caption: None,
            thumbnail: None,
        }
    }
}
